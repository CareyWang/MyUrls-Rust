use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use rand::Rng;
use redis::Commands;
use serde::Deserialize;
use serde::Serialize;
use std::env;

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

async fn shorten_url(req: web::Json<UrlRequest>) -> impl Responder {
    let client = get_redis_client();
    let mut conn = client.get_connection().unwrap();

    // 默认 180 天
    let default_ttl = env::var("DEFAULT_TTL")
        .unwrap_or_else(|_| "15552000".to_string())
        .parse::<usize>()
        .unwrap_or(15552000);

    let mut resp = JsonResponse {
        code: 0,
        msg: "".to_string(),
        data: None,
    };

    let short_url_id = get_ramdon_string();
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

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

async fn redirect(short_url_id: web::Path<String>) -> impl Responder {
    let client = get_redis_client();
    let mut conn = client.get_connection().unwrap();
    info!("try to get {}", short_url_id);

    match conn.get::<_, Option<String>>(short_url_id.clone()) {
        Ok(Some(url)) => {
            info!("redirecting: {} [{}]", short_url_id, url);
            HttpResponse::Found()
                .append_header(("Location", url))
                .finish()
        }
        _ => {
            warn!("not found: {}", short_url_id);
            HttpResponse::NotFound().body("not found or expired")
        }
    }
}

fn get_redis_client() -> redis::Client {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    redis::Client::open(redis_url).unwrap()
}

fn get_ramdon_string() -> String {
    let rng = rand::thread_rng();
    let random_string: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(7)
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
