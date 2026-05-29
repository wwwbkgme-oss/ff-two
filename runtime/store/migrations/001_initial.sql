-- ForgeFabrik DevStudio — initiales Datenbankschema
--
-- Strategie: JSONB-Speicherung (jede Entität als JSON-Dokument).
-- Vorteile: schema-flexibel, keine sqlx-compile-time DB nötig,
--           einfache Erweiterbarkeit ohne Migrations-Overhead.
-- Nachteil: kein SQL-nativer Join; für komplexe Abfragen später relationales
--           Schema in 002_relational.sql einführen.

-- ── Projects ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_projects (
    id   UUID  PRIMARY KEY,
    data JSONB NOT NULL
);

-- ── Tasks ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_tasks (
    id         UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    data       JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tasks_project ON ds_tasks (project_id);

-- ── Agents ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_agents (
    id   UUID  PRIMARY KEY,
    data JSONB NOT NULL
);

-- ── Sandboxes ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_sandboxes (
    id         UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    data       JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sandboxes_project ON ds_sandboxes (project_id);

-- ── Deployments ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_deployments (
    id         UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    data       JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_deployments_project ON ds_deployments (project_id);

-- ── World Snapshots ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ds_snapshots (
    id         UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    data       JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_snapshots_project ON ds_snapshots (project_id);
