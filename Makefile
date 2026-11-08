.DEFAULT_GOAL := help

DATABASE_URL ?= postgres://postgres:postgres@localhost:5432/postgres

.PHONY: run ## Run the HTTP server
run:
	@echo "Updating API"
	$(MAKE) generate-api
	@echo "Running HTTP server..."
	DATABASE_URL="$(DATABASE_URL)" cargo run --bin http

.PHONY: watch ## Watch for changes and restart the HTTP server
watch:
	@echo "Updating API"
	$(MAKE) generate-api
	@echo "Watching for changes in crates/http and crates/entities..."
	@echo "Note: Install cargo-watch with 'cargo install cargo-watch' if not available"
	DATABASE_URL="$(DATABASE_URL)" cargo watch \
		-w crates/http \
		-w crates/entities \
		-x "run --bin http"

.PHONY: generate ## Generate SeaORM entities from the database
generate:
	@echo "Generating entities with SeaORM CLI..."
	sea-orm-cli generate entity \
		--database-url "$(DATABASE_URL)" \
		--output-dir ./crates/entities/src \
		--lib \
		--with-serde both \
		--entity-format dense \
		-v

	@echo "Applying additional derives to generated entities..."
	python3 apply_derives.py crates/entities/src/sea_orm_active_enums.rs OauthProvider "ToSchema" '#[serde(rename_all = "lowercase")]'

	python3 apply_derives.py crates/entities/src/note.rs Model "ToSchema"
	python3 apply_derives.py crates/entities/src/note.rs Model --field user '#[serde(skip)]'
	python3 apply_derives.py crates/entities/src/note.rs Model --field note_contents '#[serde(skip)]'

	python3 apply_derives.py crates/entities/src/note_content.rs Model --field note '#[serde(skip)]'

.PHONY: migrate ## Run database migrations (usage: make migrate ARGS="up")
migrate:
	@echo "Running migrations..."
	cd crates/migration && \
		DATABASE_URL="$(DATABASE_URL)" cargo run --bin migration -- $(ARGS)

.PHONY: format
format:
	cargo fmt

.PHONY: frontend ## Run frontend npm scripts (usage: make frontend ARGS="dev")
frontend:
	@if [ -z "$(ARGS)" ]; then \
		echo "Usage: make frontend ARGS='<npm-script> [args...]'"; \
		echo "Available scripts:"; \
		npm run --list 2>/dev/null || echo "  dev, build, start, lint, generate-api"; \
		exit 1; \
	fi
	@echo "Running npm $(ARGS)..."
	cd inker-frontend && npm run $(ARGS)

.PHONY: generate-api ## generate-api api bindings for frontend
generate-api:
	cargo run --bin http -- --openapi > ./inker-frontend/openapi.json
	cd inker-frontend && openapi-ts

.PHONY: help ## Display available commands
help:
	@grep -E '^\.PHONY: .*## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ".PHONY: |## "}; \
		{printf "\033[36m%-19s\033[0m %s\n", $$2, $$3}'
