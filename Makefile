SHELL := /bin/sh

COMPOSE ?= docker compose
CARGO ?= cargo

.PHONY: up down logs ps test test-llm verify-readonly build explain

up:
	@if [ ! -f .env ]; then cp .env.example .env; fi
	$(COMPOSE) up -d --build

down:
	$(COMPOSE) down --remove-orphans

logs:
	$(COMPOSE) logs -f --tail=200

ps:
	$(COMPOSE) ps

test:
	$(CARGO) test --all --locked

test-llm:
	$(CARGO) test -p aidoc-llm --locked

verify-readonly:
	$(CARGO) test -p aidoc-sandbox --locked

build:
	$(CARGO) build --release -p aidoc-cli --locked

explain:
	$(CARGO) run -p aidoc-cli -- explain
