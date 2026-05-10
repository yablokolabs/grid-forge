FROM rust:1.84-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/grid-forge-api /usr/local/bin/grid-forge-api
EXPOSE 8080
CMD ["grid-forge-api"]

FROM debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/grid-forge-worker /usr/local/bin/grid-forge-worker
CMD ["grid-forge-worker"]
