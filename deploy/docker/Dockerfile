FROM rust:1.84-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app && useradd --system --gid app app
COPY --from=builder /app/target/release/grid-forge-api /usr/local/bin/grid-forge-api
USER app
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD ["grid-forge-api", "--health-check"] || exit 1
CMD ["grid-forge-api"]

FROM debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app && useradd --system --gid app app
COPY --from=builder /app/target/release/grid-forge-worker /usr/local/bin/grid-forge-worker
USER app
CMD ["grid-forge-worker"]
