CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    query TEXT NOT NULL,
    status TEXT NOT NULL,
    pipeline_stage TEXT NOT NULL,
    plan_json TEXT,
    clarifier_result TEXT,
    telemetry_json TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE log_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    agent_tag TEXT NOT NULL,
    message TEXT NOT NULL,
    level TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

CREATE TABLE sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    snippet TEXT NOT NULL,
    relevance DOUBLE NOT NULL,
    source_type TEXT NOT NULL,
    verification_status TEXT NOT NULL,
    embedding_id TEXT
);

CREATE TABLE research_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    stats_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE citations (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    report_id INTEGER NOT NULL REFERENCES research_reports(id),
    reference_number INTEGER NOT NULL,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    context_snippet TEXT NOT NULL,
    verification_level TEXT NOT NULL
);
