set dotenv-load := true

fmt:
  cargo fmt --all

check:
  cargo check --workspace --all-targets

test:
  cargo test --workspace

api:
  cargo run -p grid-forge-api

worker:
  cargo run -p grid-forge-worker

compose-up:
  docker-compose up --build
