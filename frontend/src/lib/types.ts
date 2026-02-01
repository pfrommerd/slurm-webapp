// --- Slurm-Common Structures ---

// Newtypes / Aliases
export type ResourceType = string;
export type NodeName = string;
export type JobId = string;
export type PartitionName = string;

// Enums
export enum NodeStatus {
    Idle = "Idle",
    Alloc = "Alloc",
    Mix = "Mix",
    Down = "Down",
    Unknown = "Unknown",
}

export enum JobStatus {
    Pending = "Pending",
    Running = "Running",
    Completed = "Completed",
    Failed = "Failed",
    Cancelled = "Cancelled",
    Unknown = "Unknown",
}

export enum PartitionStatus {
    Up = "Up",
    Down = "Down",
    Unknown = "Unknown",
}

// Domain Models
export type Node = {
    name: NodeName;
    status: NodeStatus;
    partitions: Partition[];
    resources: Map<ResourceType, NodeResource>;
    updated_at: string;
    jobs: Job[];
    cpus: number; // Added to match UI usage
    memory: number; // Added
};

export type NodeResource = {
    resource: ResourceType;
    available: number;
    total: number;
};

export type JobAllocation = {
    node: Node;
    resource: ResourceType;
    count: number;
};

export type Job = {
    job_id: JobId;
    name: string;
    user: string;
    partition: Partition;
    status: JobStatus;
    time_limit: number | null;
    start_time: string | null;
    submit_time: string;
    updated_at: string;
    allocations: JobAllocation[];
    resources: Map<ResourceType, JobResource>; // Added to store requested/allocated per job-resource
};

export type JobResource = {
    requested: number;
    allocated: number;
};

export type Partition = {
    name: string;
    status: PartitionStatus;
    access_qos: string | null;
    resource_qos: string | null;
    updated_at: string;
    // Nodes and jobs associated with this partition
    nodes: Node[];
    jobs: Job[];
};

export type ClusterState = {
    nodes: Node[];
    jobs: Job[];
    partitions: Partition[];
    updated_at: string | null;
};