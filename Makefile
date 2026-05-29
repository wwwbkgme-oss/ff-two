# ── ForgeFabrik DevStudio — Makefile ──────────────────────────────────────
#
# Kurzreferenz aller wichtigen Befehle. Läuft auf Linux und macOS.
# Voraussetzung: Rust-Toolchain (rustup), Docker (für docker-* Targets).
#
# Verwendung:
#   make          — Zeigt diese Hilfe
#   make build    — Release-Build des Binaries
#   make test     — Alle Unit- und Integration-Tests
#   make check    — Typen- und Lint-Prüfung (kein Artifact)
#   make docker-build — Docker-Image bauen
#   make docker-up    — Server via docker-compose starten

.DEFAULT_GOAL := help
.PHONY: help build build-dev test test-verbose check fmt lint clean \
        docker-build docker-up docker-down docker-logs run plugins

BINARY := devstudio
CARGO  := cargo
DOCKER := docker
DC     := docker compose

# Farben (werden ignoriert, wenn kein Terminal vorhanden ist)
BOLD  := $(shell tput bold 2>/dev/null || echo '')
RESET := $(shell tput sgr0 2>/dev/null || echo '')
GREEN := $(shell tput setaf 2 2>/dev/null || echo '')
CYAN  := $(shell tput setaf 6 2>/dev/null || echo '')

# ── Hilfe ─────────────────────────────────────────────────────────────────

help: ## Zeigt diese Hilfe
	@echo "$(BOLD)ForgeFabrik DevStudio — verfügbare Make-Targets$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*## .*$$' $(MAKEFILE_LIST) \
	  | sort \
	  | awk 'BEGIN {FS = ":.*## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'

# ── Build ─────────────────────────────────────────────────────────────────

build: ## Release-Build (optimiert, LTO)
	$(CARGO) build --release --bin $(BINARY)
	@echo "$(GREEN)Binary: target/release/$(BINARY)$(RESET)"

build-dev: ## Entwicklungs-Build (schnell, mit Debug-Symbolen)
	$(CARGO) build --bin $(BINARY)

plugins: ## Alle Plugin-cdylib-Crates bauen
	$(CARGO) build --release \
	  -p world-runtime \
	  -p agents-runtime \
	  -p gm \
	  -p economy
	@echo "$(GREEN)Plugins: target/release/lib*.so (oder .dylib / .dll)$(RESET)"

# ── Tests ─────────────────────────────────────────────────────────────────

test: ## Alle Tests (Unit + Integration)
	$(CARGO) test --workspace

test-verbose: ## Tests mit vollständiger Ausgabe
	$(CARGO) test --workspace -- --nocapture

test-api: ## Nur Integration-Tests für runtime/api
	$(CARGO) test -p api

# ── Code-Qualität ─────────────────────────────────────────────────────────

check: ## Typen-Check ohne Artifact (schnell)
	$(CARGO) check --workspace

fmt: ## Code formatieren (rustfmt)
	$(CARGO) fmt --all

fmt-check: ## Prüft Formatierung ohne Änderungen (CI)
	$(CARGO) fmt --all -- --check

lint: ## Clippy-Linter auf dem gesamten Workspace
	$(CARGO) clippy --workspace --all-targets --all-features \
	  -- -D warnings

audit: ## Bekannte CVEs in Abhängigkeiten prüfen (cargo-audit nötig)
	$(CARGO) audit

# ── Entwicklung ───────────────────────────────────────────────────────────

run: ## Server im Entwicklungsmodus starten (lädt .env)
	$(CARGO) run --bin $(BINARY)

run-release: ## Server im Release-Modus starten
	$(CARGO) run --release --bin $(BINARY)

watch: ## Server bei Quelldatei-Änderungen neu starten (cargo-watch nötig)
	cargo watch -x "run --bin $(BINARY)"

# ── Docker ────────────────────────────────────────────────────────────────

docker-build: ## Docker-Image bauen
	$(DOCKER) build -t forgefabrik/devstudio:dev .

docker-up: .env ## Server via docker-compose starten (Foreground)
	$(DC) up

docker-up-d: .env ## Server via docker-compose starten (Hintergrund)
	$(DC) up -d

docker-down: ## docker-compose stoppen und Container entfernen
	$(DC) down

docker-logs: ## Container-Logs verfolgen
	$(DC) logs -f devstudio

docker-shell: ## Shell im laufenden Container öffnen
	$(DC) exec devstudio /bin/bash

docker-clean: ## Container, Images und Volumes entfernen
	$(DC) down -v --rmi local

# ── Aufräumen ─────────────────────────────────────────────────────────────

clean: ## Cargo-Build-Artifacts löschen
	$(CARGO) clean

# ── Hilfsziele ────────────────────────────────────────────────────────────

# Erzeuge .env aus .env.example, falls noch nicht vorhanden
.env:
	@echo "$(BOLD).env nicht gefunden — kopiere .env.example nach .env$(RESET)"
	cp .env.example .env
	@echo "$(GREEN).env angelegt. Bitte die Werte anpassen.$(RESET)"
