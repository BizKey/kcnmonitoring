FROM rust:1.97.1-alpine3.24 AS builder

RUN apk add --no-cache musl-dev
ENV RUSTFLAGS="-C target-cpu=x86-64-v3"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

RUN rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM alpine:3.24 AS runner

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

RUN adduser -D -u 1000 myuser

COPY --from=builder /app/target/release/kcnmonitoring /app/kcnmonitoring

USER myuser

ENV RUST_LOG=INFO

CMD ["/app/kcnmonitoring"]