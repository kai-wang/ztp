FROM rust:1.56.0 AS builder
WORKDIR /app
COPY . .

ENV SQLX_OFFLINE true
RUN cargo build --release
# Runtime stage
FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
# Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ztp ztp
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./ztp"]