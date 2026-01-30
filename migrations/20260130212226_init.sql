-- Add migration script here
CREATE TABLE IF NOT EXISTS nodes (
    name TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    cpus INTEGER NOT NULL,
    real_memory INTEGER NOT NULL,
    resources TEXT,
    updated_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    job_id TEXT PRIMARY KEY,
    user TEXT NOT NULL,
    partition TEXT NOT NULL,
    state TEXT NOT NULL,
    num_nodes INTEGER NOT NULL,
    num_cpus INTEGER NOT NULL,
    time_limit TEXT,
    start_time DATETIME,
    submit_time DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS partitions (
    name TEXT PRIMARY KEY,
    total_nodes INTEGER NOT NULL,
    total_cpus INTEGER NOT NULL,
    state TEXT NOT NULL,
    updated_at DATETIME NOT NULL
);