CREATE TABLE IF NOT EXISTS nodes (
    name TEXT PRIMARY KEY,
    status TEXT NOT NULL,

    cpus INTEGER NOT NULL,
    cpus_alloc INTEGER NOT NULL,
    cpus_idle INTEGER NOT NULL,

    memory INTEGER NOT NULL,
    memory_alloc INTEGER NOT NULL,
    memory_free INTEGER NOT NULL,

    updated_at DATETIME NOT NULL
);

-- The partitions to which the node belongs
CREATE TABLE IF NOT EXISTS node_partitions (
    node TEXT NOT NULL,
    partition TEXT NOT NULL,
    PRIMARY KEY (node, partition)
);

CREATE TABLE IF NOT EXISTS node_resources (
    node TEXT NOT NULL,
    resource TEXT NOT NULL,
    available INTEGER NOT NULL,
    total INTEGER NOT NULL,
    PRIMARY KEY (node, resource)
);

CREATE TABLE IF NOT EXISTS partitions (
    name TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    access_qos TEXT,
    resource_qos TEXT,
    updated_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    job_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    user TEXT NOT NULL,
    partition TEXT NOT NULL,
    status TEXT NOT NULL,
    time_limit INTEGER,
    start_time DATETIME,
    submit_time DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- The currently requested and allocated resources for a job
CREATE TABLE IF NOT EXISTS job_resources (
    job_id TEXT NOT NULL,
    resource TEXT NOT NULL,
    requested INTEGER NOT NULL,
    allocated INTEGER NOT NULL,
    PRIMARY KEY (job_id, resource)
);

-- The allocations of a job onto specific nodes
CREATE TABLE IF NOT EXISTS job_allocations (
    job_id TEXT NOT NULL,
    node TEXT NOT NULL,
    resource TEXT NOT NULL,
    used INTEGER NOT NULL,
    PRIMARY KEY (job_id, node, resource)
);