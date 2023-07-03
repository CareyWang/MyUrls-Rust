# build stage
FROM rust:1.68-alpine AS build 
WORKDIR /app

# install build tools and dependencies
RUN apk add --no-cache build-base libc-dev

# copy Cargo.toml and Cargo.lock, and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release

# copy src and build the final executable
RUN rm -rf src && rm Cargo.lock
COPY src ./src
RUN cargo build --release

# runtime stage
FROM alpine:3.17 as runtime 
WORKDIR /app

# copy the executable from the build stage
COPY --from=build /app/target/release/myurls .

# set environment variables
ENV REDIS_URL=redis://127.0.0.1:6379/
ENV DEFAULT_TTL=15552000
ENV DOMAIN=localhost:8080

# expose the port and set the startup command
EXPOSE 8080
CMD ["/app/myurls"]
