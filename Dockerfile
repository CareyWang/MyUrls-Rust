# stage 1
FROM rust:1.68-alpine AS build 

WORKDIR /app

# 安装必要的构建工具和库文件
RUN apk add --no-cache \
    build-base \
    libc-dev

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
FROM alpine:3.17 as runtime 

# 设置工作目录
WORKDIR /app

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
