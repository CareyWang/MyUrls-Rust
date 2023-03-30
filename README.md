# MyUrls-Rust 

Your own url shorter service.

Power by: Rust, ChatGPT [GPT-4](https://chat.openai.com/chat?model=gpt-4)

## Usage

### build 
```shell
git clone https://github.com/CareyWang/MyUrls-Rust.git 
cd MyUrls-Rust
cargo update 
cargo build --release

export REDIS_URL=redis://127.0.0.1:6379/
export DOMAIN=http://127.0.0.1:8080
./target/release/myurls-rust
```

### docker 
```shell
docker pull careywong/myurls-rust:latest 
docker run -d --name myurls-rust --restart always \
    -p 8080:8080 \
    -e REDIS_URL=redis://127.0.0.1:6379 \
    -e DOMAIN=http://127.0.0.1:8080 \
    careywong/myurls-rust:latest
```


### docker-compose 
```shell
git clone https://github.com/CareyWang/MyUrls-Rust.git 
cd MyUrls-Rust

docker-compose up -d 
```
