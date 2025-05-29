# CARGO-CHEF - build dependencies separately from project to reduce time between builds
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked

# RUN APPLICATION
FROM debian:12-slim AS runner
ARG PORT
WORKDIR /app
COPY --from=builder /app/target/release/curto curto

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser
USER appuser

ENV RUST_LOG=debug
ENV APPLICATION_HOST=0.0.0.0
ENV APPLICATION_PORT=7229

CMD ["./curto"]
