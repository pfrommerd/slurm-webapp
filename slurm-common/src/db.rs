use crate::{
    ClusterDiff, ClusterState, Job, JobStatus, Node, NodeStatus, Partition, PartitionStatus,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

// --- Node ---

impl<'r> FromRow<'r, SqliteRow> for Node {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get("name")?;
        let status_str: String = row.try_get("status")?;
        let status = status_str.parse().unwrap_or(NodeStatus::Unknown);
        let cpus: i64 = row.try_get("cpus")?;
        let real_memory: i64 = row.try_get("real_memory")?;
        let resources_str: Option<String> = row.try_get("resources")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        let resources = if let Some(s) = resources_str {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Default::default()
        };

        Ok(Node {
            name,
            status,
            cpus: cpus as u32,
            real_memory,
            resources,
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
    let resources = serde_json::to_string(&node.resources).unwrap_or_default();
    let status = node.status.as_ref();
    sqlx::query!(
        r#"
        INSERT INTO nodes (name, status, cpus, real_memory, resources, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            status = excluded.status,
            cpus = excluded.cpus,
            real_memory = excluded.real_memory,
            resources = excluded.resources,
            updated_at = excluded.updated_at
        "#,
        node.name,
        status,
        node.cpus,
        node.real_memory,
        resources,
        node.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node(pool: &Pool<Sqlite>, name: &str) -> Result<()> {
    sqlx::query!("DELETE FROM nodes WHERE name = ?", name)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Job ---

impl<'r> FromRow<'r, SqliteRow> for Job {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let job_id: String = row.try_get("job_id")?;
        let user: String = row.try_get("user")?;
        let partition: String = row.try_get("partition")?;
        let status_str: String = row.try_get("status")?;
        let num_nodes: i64 = row.try_get("num_nodes")?;
        let num_cpus: i64 = row.try_get("num_cpus")?;
        let time_limit: Option<String> = row.try_get("time_limit")?;
        let start_time: Option<chrono::DateTime<chrono::Utc>> = row.try_get("start_time")?;
        let submit_time: chrono::DateTime<chrono::Utc> = row.try_get("submit_time")?;

        let status = status_str.parse().unwrap_or(JobStatus::Unknown);
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        Ok(Job {
            job_id,
            user,
            partition,
            status,
            num_nodes: num_nodes as u32,
            num_cpus: num_cpus as u32,
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
    let status = job.status.as_ref();
    sqlx::query!(
        r#"
        INSERT INTO jobs (job_id, user, partition, status, num_nodes, num_cpus, time_limit, start_time, submit_time, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(job_id) DO UPDATE SET
            status = excluded.status,
            updated_at = excluded.updated_at
        "#,
        job.job_id,
        job.user,
        job.partition,
        status,
        job.num_nodes,
        job.num_cpus,
        job.time_limit,
        job.start_time,
        job.submit_time,
        job.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_job(pool: &Pool<Sqlite>, job_id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM jobs WHERE job_id = ?", job_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Partition ---

impl<'r> FromRow<'r, SqliteRow> for Partition {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get("name")?;
        let total_nodes: i64 = row.try_get("total_nodes")?;
        let total_cpus: i64 = row.try_get("total_cpus")?;
        let status: PartitionStatus = row
            .try_get::<String, _>("status")?
            .parse()
            .unwrap_or(PartitionStatus::Unknown);
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        Ok(Partition {
            name,
            total_nodes: total_nodes as u32,
            total_cpus: total_cpus as u32,
            status,
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
    let status = part.status.as_ref();
    sqlx::query!(
        r#"
        INSERT INTO partitions (name, total_nodes, total_cpus, status, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            total_nodes = excluded.total_nodes,
            total_cpus = excluded.total_cpus,
            status = excluded.status,
            updated_at = excluded.updated_at
        "#,
        part.name,
        part.total_nodes,
        part.total_cpus,
        status,
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

// --- Diff ---

pub async fn apply_diff(pool: &Pool<Sqlite>, diff: ClusterDiff) -> Result<()> {
    // Nodes
    for node in diff.nodes_upserted {
        upsert_node(pool, &node).await?;
    }
    for name in diff.nodes_removed {
        delete_node(pool, &name).await?;
    }

    // Jobs
    for job in diff.jobs_upserted {
        upsert_job(pool, &job).await?;
    }
    for job_id in diff.jobs_removed {
        delete_job(pool, &job_id).await?;
    }

    // Partitions
    for part in diff.partitions_upserted {
        upsert_partition(pool, &part).await?;
    }
    for name in diff.partitions_removed {
        delete_partition(pool, &name).await?;
    }

    Ok(())
}

// --- Cluster Status ---

pub async fn fetch_cluster_state(pool: &Pool<Sqlite>) -> Result<ClusterState> {
    let nodes = fetch_all_nodes(pool).await?;
    let jobs = fetch_all_jobs(pool).await?;
    let partitions = fetch_all_partitions(pool).await?;
    // Go through all nodes, jobs, partitions and
    // get the most recent updated_at time
    let mut updated_at = None;
    for node in &nodes {
        if Some(node.updated_at) > updated_at {
            updated_at = Some(node.updated_at);
        }
    }
    for job in &jobs {
        if Some(job.updated_at) > updated_at {
            updated_at = Some(job.updated_at);
        }
    }
    for part in &partitions {
        if Some(part.updated_at) > updated_at {
            updated_at = Some(part.updated_at);
        }
    }

    Ok(ClusterState {
        nodes,
        jobs,
        partitions,
        updated_at,
    })
}
