COMPOSE = docker compose -f $(CURDIR)/infra/compose.yaml
PSQL = $(COMPOSE) exec -T postgres psql -U boilerplate -d boilerplate -v ON_ERROR_STOP=1

.PHONY: up down reset db

up:
	$(COMPOSE) up -d --wait

down:
	$(COMPOSE) down

# Deletes every development record and recreates the current schema atomically.
reset: up
	$(PSQL) --single-transaction -c 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;' -f - < $(CURDIR)/infra/schema.sql

db:
	$(COMPOSE) exec postgres psql -U boilerplate -d boilerplate
