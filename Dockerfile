# stage 1
FROM rust:1.68 AS build 

WORKDIR /app

# 复制 Cargo.toml 和 Cargo.lock 到工作目录
COPY Cargo.toml Cargo.lock ./

# 为了缓存依赖项，创建一个虚拟项目，然后使用 cargo build
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release

# 删除虚拟项目，复制实际的源代码
RUN rm -rf src && rm Cargo.lock
COPY src ./src

# 构建最终的可执行文件
RUN cargo build --release

# stage 2
FROM debian:11-slim as runtime 

# 设置工作目录
WORKDIR /app

# 安装 SSL 证书和运行时依赖
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

# 复制可执行文件到新的镜像
COPY --from=build /app/target/release/myurls .

# 设置环境变量
ENV REDIS_URL=redis://127.0.0.1:6379
ENV DEFAULT_TTL=15552000
ENV DOMAIN=http://localhost:8080

# 设置监听的端口
EXPOSE 8080

# 设置启动命令
CMD ["/app/myurls"]
