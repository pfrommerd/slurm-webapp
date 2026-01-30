use crate::{ClusterDiff, Job, JobState, Node, Partition};
use anyhow::Result;
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

// --- Node ---

impl<'r> FromRow<'r, SqliteRow> for Node {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get("name")?;
        let state: String = row.try_get("state")?;
        let cpus: i64 = row.try_get("cpus")?;
        let real_memory: i64 = row.try_get("real_memory")?;
        let resources_str: Option<String> = row.try_get("resources")?;

        let resources = if let Some(s) = resources_str {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Default::default()
        };

        Ok(Node {
            name,
            state,
            cpus: cpus as u32,
            real_memory,
            resources,
        })
    }
}

pub async fn fetch_all_nodes(pool: &Pool<Sqlite>) -> Result<Vec<Node>> {
    let nodes = sqlx::query_as::<_, Node>("SELECT * FROM nodes ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(nodes)
}

pub async fn upsert_node(
    pool: &Pool<Sqlite>,
    node: &Node,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let resources = serde_json::to_string(&node.resources).unwrap_or_default();
    sqlx::query!(
        r#"
        INSERT INTO nodes (name, state, cpus, real_memory, resources, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            state = excluded.state,
            cpus = excluded.cpus,
            real_memory = excluded.real_memory,
            resources = excluded.resources,
            updated_at = excluded.updated_at
        "#,
        node.name,
        node.state,
        node.cpus,
        node.real_memory,
        resources,
        updated_at
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
        let state_str: String = row.try_get("state")?;
        let num_nodes: i64 = row.try_get("num_nodes")?;
        let num_cpus: i64 = row.try_get("num_cpus")?;
        let time_limit: Option<String> = row.try_get("time_limit")?;
        let start_time: Option<chrono::DateTime<chrono::Utc>> = row.try_get("start_time")?;
        let submit_time: chrono::DateTime<chrono::Utc> = row.try_get("submit_time")?;

        let state_enum = match state_str.as_str() {
            "PENDING" => JobState::PENDING,
            "RUNNING" => JobState::RUNNING,
            "COMPLETED" => JobState::COMPLETED,
            "FAILED" => JobState::FAILED,
            "CANCELLED" => JobState::CANCELLED,
            _ => JobState::UNKNOWN,
        };

        Ok(Job {
            job_id,
            user,
            partition,
            state: state_enum,
            num_nodes: num_nodes as u32,
            num_cpus: num_cpus as u32,
            time_limit,
            start_time,
            submit_time,
        })
    }
}

pub async fn fetch_all_jobs(pool: &Pool<Sqlite>) -> Result<Vec<Job>> {
    let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY submit_time DESC")
        .fetch_all(pool)
        .await?;
    Ok(jobs)
}

pub async fn upsert_job(
    pool: &Pool<Sqlite>,
    job: &Job,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let state = job.state.as_ref();
    sqlx::query!(
        r#"
        INSERT INTO jobs (job_id, user, partition, state, num_nodes, num_cpus, time_limit, start_time, submit_time, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(job_id) DO UPDATE SET
            state = excluded.state,
            updated_at = excluded.updated_at
        "#,
        job.job_id,
        job.user,
        job.partition,
        state,
        job.num_nodes,
        job.num_cpus,
        job.time_limit,
        job.start_time,
        job.submit_time,
        updated_at
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
        let state: String = row.try_get("state")?;

        Ok(Partition {
            name,
            total_nodes: total_nodes as u32,
            total_cpus: total_cpus as u32,
            state,
        })
    }
}

pub async fn fetch_all_partitions(pool: &Pool<Sqlite>) -> Result<Vec<Partition>> {
    let parts = sqlx::query_as::<_, Partition>("SELECT * FROM partitions ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(parts)
}

pub async fn upsert_partition(
    pool: &Pool<Sqlite>,
    part: &Partition,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO partitions (name, total_nodes, total_cpus, state, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(name) DO UPDATE SET
            total_nodes = excluded.total_nodes,
            total_cpus = excluded.total_cpus,
            state = excluded.state,
            updated_at = excluded.updated_at
        "#,
        part.name,
        part.total_nodes,
        part.total_cpus,
        part.state,
        updated_at
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
    let updated_at = diff.updated_at;

    // Nodes
    for node in diff.nodes_upserted {
        upsert_node(pool, &node, updated_at).await?;
    }
    for name in diff.nodes_removed {
        delete_node(pool, &name).await?;
    }

    // Jobs
    for job in diff.jobs_upserted {
        upsert_job(pool, &job, updated_at).await?;
    }
    for job_id in diff.jobs_removed {
        delete_job(pool, &job_id).await?;
    }

    // Partitions
    for part in diff.partitions_upserted {
        upsert_partition(pool, &part, updated_at).await?;
    }
    for name in diff.partitions_removed {
        delete_partition(pool, &name).await?;
    }

    Ok(())
}

// --- Metadata ---

pub async fn update_metadata<T: serde::Serialize>(
    pool: &Pool<Sqlite>,
    key: &str,
    value: &T,
) -> Result<()> {
    let value_str = serde_json::to_string(value)?;
    sqlx::query!(
        r#"
        INSERT INTO metadata (key, value)
        VALUES (?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value
        "#,
        key,
        value_str
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_metadata<T: serde::de::DeserializeOwned>(
    pool: &Pool<Sqlite>,
    key: &str,
) -> Result<Option<T>> {
    let rec = sqlx::query!("SELECT value FROM metadata WHERE key = ?", key)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = rec {
        let value = serde_json::from_str(&row.value)?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}
