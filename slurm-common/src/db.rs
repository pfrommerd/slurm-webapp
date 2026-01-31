use crate::table::Table;
use crate::{
    ClusterDiff, ClusterState, Job, JobAllocation, JobId, JobResource, JobStatus, Node, NodeName,
    NodePartition, NodeResource, NodeStatus, Partition, PartitionStatus, ResourceType,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

// --- Node ---

impl<'r> FromRow<'r, SqliteRow> for Node {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name_str: String = row.try_get("name")?;
        let status_str: String = row.try_get("status")?;
        let status = serde_json::from_str(&status_str).unwrap_or(NodeStatus::Unknown);

        // CPU stats
        let cpus: i64 = row.try_get("cpus")?;
        let cpus_alloc: i64 = row.try_get("cpus_alloc")?;
        let cpus_idle: i64 = row.try_get("cpus_idle")?;

        // Memory stats
        let memory: i64 = row.try_get("memory")?;
        let memory_alloc: i64 = row.try_get("memory_alloc")?;
        let memory_free: i64 = row.try_get("memory_free")?;

        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        Ok(Node {
            name: NodeName(name_str),
            status,
            cpus: cpus as u32,
            cpus_alloc: cpus_alloc as u32,
            cpus_idle: cpus_idle as u32,
            memory,
            memory_alloc,
            memory_free,
            partitions: Vec::new(), // Populated manually if needed, or left empty knowing it's in node_partitions
            updated_at,
        })
    }
}

pub async fn fetch_all_nodes(pool: &Pool<Sqlite>) -> Result<Vec<Node>> {
    let nodes = sqlx::query_as::<_, Node>("SELECT * FROM nodes ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(nodes)
}

pub async fn upsert_node(pool: &Pool<Sqlite>, node: &Node) -> Result<()> {
    let status = serde_json::to_string(&node.status).unwrap_or_default();
    sqlx::query!(
        r#"
        INSERT INTO nodes (name, status, cpus, cpus_alloc, cpus_idle, memory, memory_alloc, memory_free, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            status = excluded.status,
            cpus = excluded.cpus,
            cpus_alloc = excluded.cpus_alloc,
            cpus_idle = excluded.cpus_idle,
            memory = excluded.memory,
            memory_alloc = excluded.memory_alloc,
            memory_free = excluded.memory_free,
            updated_at = excluded.updated_at
        "#,
        node.name.0,
        status,
        node.cpus,
        node.cpus_alloc,
        node.cpus_idle,
        node.memory,
        node.memory_alloc,
        node.memory_free,
        node.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node(pool: &Pool<Sqlite>, name: &NodeName) -> Result<()> {
    sqlx::query!("DELETE FROM nodes WHERE name = ?", name.0)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Node Partition ---

impl<'r> FromRow<'r, SqliteRow> for NodePartition {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let node: String = row.try_get("node")?;
        let partition: String = row.try_get("partition")?;
        Ok(NodePartition {
            node: NodeName(node),
            partition,
        })
    }
}

pub async fn fetch_all_node_partitions(pool: &Pool<Sqlite>) -> Result<Vec<NodePartition>> {
    let items = sqlx::query_as::<_, NodePartition>("SELECT * FROM node_partitions")
        .fetch_all(pool)
        .await?;
    Ok(items)
}

pub async fn upsert_node_partition(pool: &Pool<Sqlite>, item: &NodePartition) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO node_partitions (node, partition)
        VALUES (?, ?)
        ON CONFLICT(node, partition) DO NOTHING
        "#,
        item.node.0,
        item.partition
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node_partition(
    pool: &Pool<Sqlite>,
    node: &NodeName,
    partition: &str,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM node_partitions WHERE node = ? AND partition = ?",
        node.0,
        partition
    )
    .execute(pool)
    .await?;
    Ok(())
}

// --- Node Resource ---

impl<'r> FromRow<'r, SqliteRow> for NodeResource {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let node: String = row.try_get("node")?;
        let resource: String = row.try_get("resource")?;
        let available: i64 = row.try_get("available")?;
        let total: i64 = row.try_get("total")?;
        Ok(NodeResource {
            node: NodeName(node),
            resource: ResourceType(resource),
            available: available as u64,
            total: total as u64,
        })
    }
}

pub async fn fetch_all_node_resources(pool: &Pool<Sqlite>) -> Result<Vec<NodeResource>> {
    let items = sqlx::query_as::<_, NodeResource>("SELECT * FROM node_resources")
        .fetch_all(pool)
        .await?;
    Ok(items)
}

pub async fn upsert_node_resource(pool: &Pool<Sqlite>, item: &NodeResource) -> Result<()> {
    // Cast u64 to i64 for sqlite
    let available = item.available as i64;
    let total = item.total as i64;
    sqlx::query!(
        r#"
        INSERT INTO node_resources (node, resource, available, total)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(node, resource) DO UPDATE SET
            available = excluded.available,
            total = excluded.total
        "#,
        item.node.0,
        item.resource.0,
        available,
        total
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node_resource(
    pool: &Pool<Sqlite>,
    node: &NodeName,
    resource: &ResourceType,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM node_resources WHERE node = ? AND resource = ?",
        node.0,
        resource.0
    )
    .execute(pool)
    .await?;
    Ok(())
}

// --- Job ---

impl<'r> FromRow<'r, SqliteRow> for Job {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let job_id_str: String = row.try_get("job_id")?;
        let user: String = row.try_get("user")?;
        let partition: String = row.try_get("partition")?;
        let status_str: String = row.try_get("status")?;
        let time_limit: Option<String> = row.try_get("time_limit")?;
        let start_time: Option<DateTime<Utc>> = row.try_get("start_time")?;
        let submit_time: DateTime<Utc> = row.try_get("submit_time")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        let status = serde_json::from_str(&status_str).unwrap_or(JobStatus::Unknown);
        let job_id_val = job_id_str.parse::<i64>().unwrap_or(0);

        Ok(Job {
            job_id: JobId(job_id_val),
            user,
            partition,
            status,
            time_limit,
            start_time,
            submit_time,
            updated_at,
        })
    }
}

pub async fn fetch_all_jobs(pool: &Pool<Sqlite>) -> Result<Vec<Job>> {
    let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY submit_time DESC")
        .fetch_all(pool)
        .await?;
    Ok(jobs)
}

pub async fn upsert_job(pool: &Pool<Sqlite>, job: &Job) -> Result<()> {
    let status = serde_json::to_string(&job.status).unwrap_or_default();
    let job_id_str = job.job_id.0.to_string();
    sqlx::query!(
        r#"
        INSERT INTO jobs (job_id, user, partition, status, time_limit, start_time, submit_time, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(job_id) DO UPDATE SET
            user = excluded.user,
            partition = excluded.partition,
            status = excluded.status,
            time_limit = excluded.time_limit,
            start_time = excluded.start_time,
            submit_time = excluded.submit_time,
            updated_at = excluded.updated_at
        "#,
        job_id_str,
        job.user,
        job.partition,
        status,
        job.time_limit,
        job.start_time,
        job.submit_time,
        job.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_job(pool: &Pool<Sqlite>, job_id: &JobId) -> Result<()> {
    let job_id_str = job_id.0.to_string();
    sqlx::query!("DELETE FROM jobs WHERE job_id = ?", job_id_str)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Job Resource ---

impl<'r> FromRow<'r, SqliteRow> for JobResource {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let job_id_str: String = row.try_get("job_id")?;
        let resource: String = row.try_get("resource")?;
        let requested: i64 = row.try_get("requested")?;
        let allocated: i64 = row.try_get("allocated")?;

        let job_id_val = job_id_str.parse::<i64>().unwrap_or(0);

        Ok(JobResource {
            job: JobId(job_id_val),
            resource: ResourceType(resource),
            requested,
            allocated,
        })
    }
}

pub async fn fetch_all_job_resources(pool: &Pool<Sqlite>) -> Result<Vec<JobResource>> {
    let items = sqlx::query_as::<_, JobResource>("SELECT * FROM job_resources")
        .fetch_all(pool)
        .await?;
    Ok(items)
}

async fn upsert_job_resource(pool: &Pool<Sqlite>, item: &JobResource) -> Result<()> {
    let job_id_str = item.job.0.to_string();
    sqlx::query!(
        r#"
        INSERT INTO job_resources (job_id, resource, requested, allocated)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(job_id, resource) DO UPDATE SET
            requested = excluded.requested,
            allocated = excluded.allocated
        "#,
        job_id_str,
        item.resource.0,
        item.requested,
        item.allocated
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn delete_job_resource(
    pool: &Pool<Sqlite>,
    job_id: &JobId,
    resource: &ResourceType,
) -> Result<()> {
    let job_id_str = job_id.0.to_string();
    sqlx::query!(
        "DELETE FROM job_resources WHERE job_id = ? AND resource = ?",
        job_id_str,
        resource.0
    )
    .execute(pool)
    .await?;
    Ok(())
}

// --- Job Allocation ---

impl<'r> FromRow<'r, SqliteRow> for JobAllocation {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let job_id_str: String = row.try_get("job_id")?;
        let node: String = row.try_get("node")?;
        let resource: String = row.try_get("resource")?;
        let used: i64 = row.try_get("used")?;

        let job_id_val = job_id_str.parse::<i64>().unwrap_or(0);

        Ok(JobAllocation {
            job: JobId(job_id_val),
            node: NodeName(node),
            resource: ResourceType(resource),
            used,
        })
    }
}

pub async fn fetch_all_job_allocations(pool: &Pool<Sqlite>) -> Result<Vec<JobAllocation>> {
    let items = sqlx::query_as::<_, JobAllocation>("SELECT * FROM job_allocations")
        .fetch_all(pool)
        .await?;
    Ok(items)
}

pub async fn upsert_job_allocation(pool: &Pool<Sqlite>, item: &JobAllocation) -> Result<()> {
    let job_id_str = item.job.0.to_string();
    sqlx::query!(
        r#"
        INSERT INTO job_allocations (job_id, node, resource, used)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(job_id, node, resource) DO UPDATE SET
            used = excluded.used
        "#,
        job_id_str,
        item.node.0,
        item.resource.0,
        item.used
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_job_allocation(
    pool: &Pool<Sqlite>,
    job_id: &JobId,
    node: &NodeName,
    resource: &ResourceType,
) -> Result<()> {
    let job_id_str = job_id.0.to_string();
    sqlx::query!(
        "DELETE FROM job_allocations WHERE job_id = ? AND node = ? AND resource = ?",
        job_id_str,
        node.0,
        resource.0
    )
    .execute(pool)
    .await?;
    Ok(())
}

// --- Partition ---

impl<'r> FromRow<'r, SqliteRow> for Partition {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get("name")?;
        let status_str: String = row.try_get("status")?;
        let status = serde_json::from_str(&status_str).unwrap_or(PartitionStatus::Unknown);

        let total_cpus: i64 = row.try_get("total_cpus")?;
        let total_cpus_alloc: i64 = row.try_get("total_cpus_alloc")?;
        let total_cpus_idle: i64 = row.try_get("total_cpus_idle")?;

        let total_memory: i64 = row.try_get("total_memory")?;
        let total_memory_alloc: i64 = row.try_get("total_memory_alloc")?;
        let total_memory_free: i64 = row.try_get("total_memory_free")?;

        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        Ok(Partition {
            name,
            status,
            total_cpus: total_cpus as u32,
            total_cpus_alloc: total_cpus_alloc as u32,
            total_cpus_idle: total_cpus_idle as u32,
            total_memory,
            total_memory_alloc,
            total_memory_free,
            updated_at,
        })
    }
}

pub async fn fetch_all_partitions(pool: &Pool<Sqlite>) -> Result<Vec<Partition>> {
    let parts = sqlx::query_as::<_, Partition>("SELECT * FROM partitions ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(parts)
}

pub async fn upsert_partition(pool: &Pool<Sqlite>, part: &Partition) -> Result<()> {
    let status = serde_json::to_string(&part.status).unwrap_or_default();
    sqlx::query!(
        r#"
        INSERT INTO partitions (name, status, total_cpus, total_cpus_alloc, total_cpus_idle, total_memory, total_memory_alloc, total_memory_free, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            status = excluded.status,
            total_cpus = excluded.total_cpus,
            total_cpus_alloc = excluded.total_cpus_alloc,
            total_cpus_idle = excluded.total_cpus_idle,
            total_memory = excluded.total_memory,
            total_memory_alloc = excluded.total_memory_alloc,
            total_memory_free = excluded.total_memory_free,
            updated_at = excluded.updated_at
        "#,
        part.name,
        status,
        part.total_cpus,
        part.total_cpus_alloc,
        part.total_cpus_idle,
        part.total_memory,
        part.total_memory_alloc,
        part.total_memory_free,
        part.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_partition(pool: &Pool<Sqlite>, name: &str) -> Result<()> {
    sqlx::query!("DELETE FROM partitions WHERE name = ?", name)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Cluster Status ---

pub async fn fetch_cluster_state(pool: &Pool<Sqlite>) -> Result<ClusterState> {
    let nodes_vec = fetch_all_nodes(pool).await?;
    let jobs_vec = fetch_all_jobs(pool).await?;
    let partitions_vec = fetch_all_partitions(pool).await?;
    let node_partitions_vec = fetch_all_node_partitions(pool).await?;
    let node_resources_vec = fetch_all_node_resources(pool).await?;
    let job_resources_vec = fetch_all_job_resources(pool).await?;
    let job_allocations_vec = fetch_all_job_allocations(pool).await?;

    // Calculate updated_at as the most recent update time
    // for any of the tables
    let updated_at = jobs_vec
        .iter()
        .map(|j| Some(j.updated_at))
        .chain(nodes_vec.iter().map(|n| Some(n.updated_at)))
        .chain(partitions_vec.iter().map(|p| Some(p.updated_at)))
        .max()
        .unwrap_or_else(|| None);

    Ok(ClusterState {
        nodes: Table::from(nodes_vec),
        jobs: Table::from(jobs_vec),
        partitions: Table::from(partitions_vec),
        node_partitions: Table::from(node_partitions_vec),
        node_resources: Table::from(node_resources_vec),
        job_resources: Table::from(job_resources_vec),
        job_allocations: Table::from(job_allocations_vec),
        updated_at,
    })
}

pub async fn apply_diff(pool: &Pool<Sqlite>, diff: ClusterDiff) -> Result<()> {
    // Partitions
    for item in diff.partitions.added {
        upsert_partition(pool, &item).await?;
    }
    for item in diff.partitions.changed {
        upsert_partition(pool, &item).await?;
    }
    for key in diff.partitions.removed {
        delete_partition(pool, &key).await?;
    }

    // Nodes
    for item in diff.nodes.added {
        upsert_node(pool, &item).await?;
    }
    for item in diff.nodes.changed {
        upsert_node(pool, &item).await?;
    }
    for key in diff.nodes.removed {
        delete_node(pool, &key).await?;
    }

    // Node Partitions
    for item in diff.node_partitions.added {
        upsert_node_partition(pool, &item).await?;
    }
    for item in diff.node_partitions.changed {
        upsert_node_partition(pool, &item).await?;
    }
    for key in diff.node_partitions.removed {
        delete_node_partition(pool, &key.0, &key.1).await?;
    }

    // Node Resources
    for item in diff.node_resources.added {
        upsert_node_resource(pool, &item).await?;
    }
    for item in diff.node_resources.changed {
        upsert_node_resource(pool, &item).await?;
    }
    for key in diff.node_resources.removed {
        delete_node_resource(pool, &key.0, &key.1).await?;
    }

    // Jobs
    for item in diff.jobs.added {
        upsert_job(pool, &item).await?;
    }
    for item in diff.jobs.changed {
        upsert_job(pool, &item).await?;
    }
    for key in diff.jobs.removed {
        delete_job(pool, &key).await?;
    }

    // Job Resources
    for item in diff.job_resources.added {
        upsert_job_resource(pool, &item).await?;
    }
    for item in diff.job_resources.changed {
        upsert_job_resource(pool, &item).await?;
    }
    for key in diff.job_resources.removed {
        delete_job_resource(pool, &key.0, &key.1).await?;
    }

    // Job Allocations
    for item in diff.job_allocations.added {
        upsert_job_allocation(pool, &item).await?;
    }
    for item in diff.job_allocations.changed {
        upsert_job_allocation(pool, &item).await?;
    }
    for key in diff.job_allocations.removed {
        delete_job_allocation(pool, &key.0, &key.1, &key.2).await?;
    }

    Ok(())
}
