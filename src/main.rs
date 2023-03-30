use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use rand::Rng;
use redis::Commands;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use url::Url;


#[derive(Deserialize)]
struct UrlRequest {
    url: String,
}

#[derive(Serialize)]
struct JsonResponse {
    code: i32,
    msg: String,
    data: Option<UrlData>,
}

#[derive(Serialize)]
struct UrlData {
    short_url: String,
    url: String,
}

// 默认服务域名
const DEFAULT_DOMAIN: &str = "http://localhost:8080";
// 默认 redis 地址
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379/";
// 默认过期时间 180 天
const DEFAULT_TTL: usize = 15552000;
// 默认随机字符串长度
const DEFAULT_CHAR_LEN: usize = 8;

// 生成短链接
async fn shorten_url(req: web::Json<UrlRequest>) -> impl Responder {
    let mut resp = JsonResponse {
        code: 0,
        msg: "".to_string(),
        data: None,
    };
    
    // 校验 url 是否合法，不合法直接返回
    let url = req.url.clone();
    if Url::parse(&url).is_err() {
        warn!("invalid url found: {}", url);
        resp.code = 1;
        resp.msg = format!("invalid url found: {}", url);
        return HttpResponse::Ok().json(resp);
    }
    
    let client = get_redis_client();
    let mut conn = client.get_connection().unwrap();

    // 默认 180 天
    let default_ttl = env::var("DEFAULT_TTL")
        .unwrap_or_else(|_| "15552000".to_string())
        .parse::<usize>()
        .unwrap_or(DEFAULT_TTL);

    let short_url_id = get_ramdon_string();
    let domain = env::var("DOMAIN").unwrap_or_else(|_| DEFAULT_DOMAIN.to_string());

    match conn.exists::<String, bool>(short_url_id.clone()) {
        Ok(false) => {
            conn.set_ex::<String, String, ()>(short_url_id.clone(), req.url.clone(), default_ttl)
                .unwrap();
            info!("generated: {}[{}]", short_url_id, req.url);

            resp.data = Some(UrlData {
                short_url: format!("{}/{}", domain, short_url_id),
                url: req.url.clone(),
            });
            return HttpResponse::Ok().json(resp);
        }
        Ok(true) => {
            resp.code = 1001;
            resp.msg = "please try again".to_string();
            warn!("already exists: {}", short_url_id);
            return HttpResponse::Ok().json(resp);
        }
        Err(_) => {
            resp.code = 1002;
            resp.msg = "runtime error, please try again".to_string();
            warn!("redis set error: {} [{}]", short_url_id, req.url);
            return HttpResponse::InternalServerError().json(resp);
        }
    }
}

// 短链接重定向
async fn redirect(short_url_id: web::Path<String>) -> impl Responder {
    let client = get_redis_client();
    let mut conn = client.get_connection().unwrap();
    match conn.get::<_, Option<String>>(short_url_id.clone()) {
        Ok(Some(url)) => {
            if let Ok(parsed_url) = Url::parse(&url) {
                info!("redirect: {} [{}]", short_url_id, url);
                HttpResponse::Found().append_header(("Location", String::from(parsed_url))).finish()
            } else {
                warn!("invalid url found: {} [{}]", short_url_id, url);
                HttpResponse::BadRequest().body(format!("invalid url found: {} [{}]", short_url_id, url))
            }
        }
        _ => {
            warn!("not found: {}", short_url_id);
            HttpResponse::NotFound().body("not found or expired")
        }
    }
}

// 获取 redis 客户端
fn get_redis_client() -> redis::Client {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    redis::Client::open(redis_url).unwrap()
}

// 生成随机字符串
fn get_ramdon_string() -> String {
    let rng = rand::thread_rng();
    let random_string: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(DEFAULT_CHAR_LEN)
        .map(char::from)
        .collect();
    random_string
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 检查 redis 链接
    get_redis_client().get_connection().ok().map_or_else(
        || {
            warn!("redis connect failed");
            std::process::exit(1);
        },
        |_| {
            info!("redis connect success");
        },
    );
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/shorten").route(web::post().to(shorten_url)))
            .service(web::resource("/{short_url_id}").route(web::get().to(redirect)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
