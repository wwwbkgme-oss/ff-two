# ── Stage 1: Builder ──────────────────────────────────────────────────────
# Vollständige Rust-Toolchain für den Compile-Schritt.
# Alle C-Abhängigkeiten (OpenSSL, musl) werden durch reine Rust-Alternativen
# ersetzt (rustls, bollard ohne openssl), sodass das finale Image schlank bleibt.
FROM rust:1.87-bookworm AS builder

WORKDIR /build

# Dependency-Cache: Nur die Manifests zuerst kopieren, damit Cargo-Caches
# zwischen Builds erhalten bleiben, wenn sich nur Quellcode ändert.
COPY Cargo.toml Cargo.lock ./
COPY foundation/types/Cargo.toml      foundation/types/Cargo.toml
COPY foundation/events/Cargo.toml     foundation/events/Cargo.toml
COPY foundation/errors/Cargo.toml     foundation/errors/Cargo.toml
COPY domain/world/Cargo.toml          domain/world/Cargo.toml
COPY domain/agents/Cargo.toml         domain/agents/Cargo.toml
COPY domain/security/Cargo.toml       domain/security/Cargo.toml
COPY domain/deployment/Cargo.toml     domain/deployment/Cargo.toml
COPY runtime/config/Cargo.toml        runtime/config/Cargo.toml
COPY runtime/store/Cargo.toml         runtime/store/Cargo.toml
COPY runtime/queue/Cargo.toml         runtime/queue/Cargo.toml
COPY runtime/sandbox/Cargo.toml       runtime/sandbox/Cargo.toml
COPY runtime/api/Cargo.toml           runtime/api/Cargo.toml
COPY runtime/server/Cargo.toml        runtime/server/Cargo.toml
COPY plugins/plugin-world/Cargo.toml   plugins/plugin-world/Cargo.toml
COPY plugins/plugin-agents/Cargo.toml  plugins/plugin-agents/Cargo.toml
COPY plugins/plugin-gm/Cargo.toml      plugins/plugin-gm/Cargo.toml
COPY plugins/plugin-economy/Cargo.toml plugins/plugin-economy/Cargo.toml

# Platzhalter-Quellcode für Caching der Abhängigkeiten
RUN find . -name Cargo.toml -not -path "./Cargo.toml" | while read f; do \
    dir=$(dirname "$f"); \
    mkdir -p "$dir/src"; \
    # Plugin-Crates brauchen lib.rs (keine main.rs) \
    if echo "$dir" | grep -q "plugin"; then \
        echo "#![allow(unused)] fn _dummy() {}" > "$dir/src/lib.rs"; \
    elif echo "$dir" | grep -q "server"; then \
        echo "fn main() {}" > "$dir/src/main.rs"; \
    else \
        echo "#![allow(unused)]" > "$dir/src/lib.rs"; \
    fi \
done && cargo build --release --bin devstudio 2>/dev/null; true

# Echten Quellcode kopieren und neu bauen
COPY . .
# Timestamps erneuern, damit Cargo den Cache invalidiert
RUN touch foundation/types/src/lib.rs \
         foundation/events/src/lib.rs \
         foundation/errors/src/lib.rs \
         domain/world/src/lib.rs \
         domain/agents/src/lib.rs \
         domain/security/src/lib.rs \
         domain/deployment/src/lib.rs \
         runtime/config/src/lib.rs \
         runtime/store/src/lib.rs \
         runtime/queue/src/lib.rs \
         runtime/sandbox/src/lib.rs \
         runtime/api/src/lib.rs \
         runtime/server/src/main.rs

RUN cargo build --release --bin devstudio

# ── Stage 2: Runtime ──────────────────────────────────────────────────────
# Minimales Debian-Image — nur das Binary und glibc.
FROM debian:bookworm-slim AS runtime

# CA-Zertifikate für ausgehende TLS-Verbindungen (Anthropic API, etc.)
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Nicht-Root-User für sichere Ausführung
RUN useradd --no-create-home --shell /bin/false devstudio
USER devstudio

WORKDIR /app

# Binary aus dem Builder-Stage kopieren
COPY --from=builder /build/target/release/devstudio /app/devstudio

# Sandbox-Verzeichnis anlegen (Standard-Pfad aus Settings)
COPY --chown=devstudio:devstudio --from=builder /build/target/release/devstudio /app/devstudio

EXPOSE 8080

ENV DEVSTUDIO_SERVER_HOST=0.0.0.0
ENV DEVSTUDIO_SERVER_PORT=8080
ENV DEVSTUDIO_LOG_JSON=true

ENTRYPOINT ["/app/devstudio"]
