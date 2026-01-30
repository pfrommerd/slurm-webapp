export type ClusterStatus = {
    nodes: Node[];
    jobs: Job[];
    partitions: Partition[];
    updated_at: string; // ISO string
};

export type Resource = {
    res_id: string;
    total: number;
    allocated: number;
};

export type Node = {
    name: string;
    state: string; // "idle", "alloc", "down"
    cpus: number;
    real_memory: number; // in MB
    resources: Record<string, Resource>;
};

export enum JobState {
    PENDING = "PENDING",
    RUNNING = "RUNNING",
    COMPLETED = "COMPLETED",
    FAILED = "FAILED",
    CANCELLED = "CANCELLED",
    UNKNOWN = "UNKNOWN",
}

export type Job = {
    job_id: string;
    user: string;
    partition: string;
    state: JobState;
    num_nodes: number;
    num_cpus: number;
    time_limit: string | null;
    start_time: string | null; // ISO string
    submit_time: string; // ISO string
};

export type Partition = {
    name: string;
    total_nodes: number;
    total_cpus: number;
    state: string; // "UP", "DOWN"
};
