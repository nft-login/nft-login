FROM rust:1.56.0-slim-buster AS builder
WORKDIR /usr/src/

RUN USER=root cargo new nft-login
WORKDIR /usr/src/nft-login
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs
COPY do-not-use.pem ./do-not-use.pem
COPY src ./src
COPY static ./static
RUN touch src/main.rs
RUN cargo build --release

FROM rust:1.56.0-slim-buster

COPY --from=builder /usr/src/nft-login/target/release/nft-login /bin
USER 1000
CMD [ "nft-login" ]
