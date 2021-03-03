# FROM rust:alpine
FROM debian:buster-slim

# RUN apk update && apk add openssl-dev alpine-sdk && cargo install --path .
# RUN apk update && apk add openssl-dev alpine-sdk
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl-dev \
        ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/myapp
COPY target_docker/debug/manager .

ENTRYPOINT ["/usr/src/myapp/manager"]

# FROM alpine
# 
# RUN mkdir /app
# WORKDIR /app
# COPY --from=builder /usr/src/myapp/target/debug/manager /app/manager
# 
# ENTRYPOINT ["manager"]