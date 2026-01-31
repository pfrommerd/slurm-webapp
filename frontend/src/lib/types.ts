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

export enum NodeStatus {
    IDLE = "Idle",
    MIX = "Mix",
    ALLOC = "Alloc",
    DOWN = "Down",
}

export type Node = {
    name: string;
    status: NodeStatus;
    cpus: number;
    real_memory: number; // in MB
    resources: Record<string, Resource>;
};

export enum JobStatus {
    PENDING = "Pending",
    RUNNING = "Running",
    COMPLETED = "Completed",
    FAILED = "Failed",
    CANCELLED = "Cancelled",
    UNKNOWN = "Unknown",
}

export type Job = {
    job_id: string;
    user: string;
    partition: string;
    status: JobStatus;
    num_nodes: number;
    num_cpus: number;
    time_limit: string | null;
    start_time: string | null; // ISO string
    submit_time: string; // ISO string
};

export enum PartitionStatus {
    UP = "Up",
    DOWN = "Down",
    Unknown = "Unknown",
}

export type Partition = {
    name: string;
    total_nodes: number;
    total_cpus: number;
    status: PartitionStatus;
};
