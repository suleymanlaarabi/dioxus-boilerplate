COMPOSE = docker compose -f $(CURDIR)/infra/compose.yaml
APP_PORT ?= 8080

.PHONY: up down reset db dev fmt lint

up:
	$(COMPOSE) up -d --wait

down:
	$(COMPOSE) down

# Discards development database records and stored files, then recreates the schema.
reset:
	$(COMPOSE) down --volumes
	$(COMPOSE) up -d --wait

db:
	$(COMPOSE) exec postgres psql -U boilerplate -d boilerplate

# The CLI port and email links use the same development configuration.
dev:
	APP_URL=http://localhost:$(APP_PORT) dx serve --port $(APP_PORT)

fmt:
	cargo fmt --all

lint:
	cargo clippy --no-default-features --features server
	cargo clippy --no-default-features --features web --target wasm32-unknown-unknown
