.PHONY: setup fmt test check clippy api worker compose-up compose-down migrate seed docs

setup:
	cargo fetch

docs:
	@echo "OpenAPI served at http://localhost:8080/docs when apps/api is running"

fmt:
	cargo fmt --all

check:
	cargo check --workspace --all-targets

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

api:
	cargo run -p grid-forge-api

worker:
	cargo run -p grid-forge-worker

compose-up:
	docker-compose up --build

compose-down:
	docker-compose down -v

migrate:
	sqlx migrate run --source migrations

seed:
	psql "$${GRID_FORGE_DATABASE_URL}" -f examples/seed/fictional_utility.sql
