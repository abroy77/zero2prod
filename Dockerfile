# Builder stage
FROM lukemathwalker/cargo-chef:latest-rust-1.77.2 AS chef

WORKDIR app/

RUN apt update && apt install lld clang -y

FROM chef AS planer

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planer /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin zero2prod

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod

COPY configuration configuration

# move static files to the runtime image
COPY --from=builder /app/static static

ENV APP_ENVIRONMENT production

ENTRYPOINT ["./zero2prod"]