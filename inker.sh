#!/usr/bin/env bash

set -euo pipefail

export DATABASE_URL="postgres://postgres@localhost:5432/postgres"

COMMAND=${1:-""}
shift || true # Remove first arg, ignore error if no args

case "$COMMAND" in
run)
  echo "Running cargo run..."
  cargo run --bin http
  ;;
watch)
  echo "Watching for changes in crates/http..."
  echo "Note: Install cargo-watch with 'cargo install cargo-watch' if not available"
  cargo watch -w crates/http -w crates/entities -x "run --bin http"
  ;;
generate)
  echo "Generating entities with SeaORM CLI..."
  sea-orm-cli generate entity \
    --database-url "$DATABASE_URL" \
    --output-dir ./crates/entities/src \
    --lib \
    --with-serde both \
    --entity-format dense \
    -v

  echo "Applying additional derives to generated entities..."
  python3 apply_derives.py crates/entities/src/sea_orm_active_enums.rs OauthProvider "ToSchema" "#[serde(rename_all = \"lowercase\")]"
  python3 apply_derives.py crates/entities/src/note.rs Model "ToSchema"

  python3 apply_derives.py crates/entities/src/note.rs Model --field blocks "#[serde(skip)]"
  python3 apply_derives.py crates/entities/src/note.rs Model --field user "#[serde(skip)]"
  ;;
migrate)
  ARG=${1:-""}
  ARG2=${2:-""}
  cd crates/migration
  echo "Running cargo run --bin migration -- $ARG..."
  cargo run --bin migration -- "$ARG" $ARG2
  cd ../..
  ;;
frontend)
  cd frontend
  if [ $# -eq 0 ]; then
    echo "Usage: $0 frontend <npm-script> [args...]"
    echo "Available scripts:"
    npm run --list 2>/dev/null || echo "  dev, build, start, lint, generate-api"
    exit 1
  fi
  echo "Running npm run $*..."
  npm run "$@"
  ;;
*)
  echo "Usage: $0 <command> [args]"
  echo ""
  echo "Commands:"
  echo "  run                    Run the HTTP server"
  echo "  watch                  Watch for changes and restart server"
  echo "  generate               Generate SeaORM entities from database"
  echo "  migrate <up|down|...>  Run database migrations"
  echo "  frontend <script>      Run frontend npm scripts (dev, build, generate-api, etc.)"
  echo ""
  echo "Examples:"
  echo "  $0 watch               # Watch and restart on changes"
  echo "  $0 frontend dev        # Start frontend dev server"
  echo "  $0 frontend generate-api  # Regenerate TypeScript SDK"
  exit 1
  ;;
esac
