FROM rust:latest

WORKDIR /usr/src/myapp
RUN apt-get update && apt-get install -y libssl-dev
# RUN apk update && apk add openssl-dev alpine-sdk
# RUN apk update && apk add openssl-dev alpine-sdk && cargo install --path .

CMD ["/bin/sh"]