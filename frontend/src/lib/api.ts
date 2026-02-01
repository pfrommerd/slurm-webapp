import type {
    ClusterState,
    Node,
    Job,
    Partition,
    NodeResource,
    JobResource,
    JobAllocation,
    NodeName,
    JobId,
    PartitionName,
    ResourceType
} from './types';
import {
    NodeStatus,
    JobStatus,
    PartitionStatus
} from './types';

// --- Backend Wire Format (Exact JSON match) ---

export type BackendClusterState = {
    nodes: BackendNode[];
    jobs: BackendJob[];
    partitions: BackendPartition[];
    node_resources: BackendNodeResource[];
    node_parittions: BackendNodePartition[];
    job_resources: BackendJobResource[];
    job_allocations: BackendJobAllocation[];
    updated_at: string;
};

export type BackendNode = {
    name: string;
    status: string;
    cpus: number;
    cpus_alloc: number;
    cpus_idle: number;
    real_memory: number;
    memory_alloc: number;
    memory_free: number;
    partitions: string[];
    updated_at: string;
};

export type BackendNodeResource = {
    node: string;
    resource: string;
    available: number;
    total: number;
};

export type BackendNodePartition = {
    node: string;
    partition: string;
};

export type BackendJob = {
    job_id: string;
    name: string;
    user: string;
    partition: string;
    status: string;
    time_limit: number | null;
    start_time: string | null;
    submit_time: string;
    updated_at: string;
};

export type BackendJobResource = {
    job: string;
    resource: string;
    requested: number;
    allocated: number;
};

export type BackendJobAllocation = {
    job: JobId;
    node: NodeName;
    resource: ResourceType;
    allocated: number;
};

export type BackendPartition = {
    name: string;
    status: PartitionStatus;
    access_qos: string | null;
    resource_qos: string | null;
    updated_at: string;
};

// --- API & Conversion Logic ---

const API_BASE = 'http://localhost:3000/api';

export async function fetchState(): Promise<ClusterState> {
    const res = await fetch(`${API_BASE}/state`);
    if (!res.ok) throw new Error('Failed to fetch state');

    const backendState: BackendClusterState = await res.json();
    return convertToModel(backendState);
}

export function convertToModel(state: BackendClusterState): ClusterState {
    const nodes = new Map<string, Node>();
    const jobs = new Map<string, Job>();
    const partitions = new Map<string, Partition>();

    // 1. Init Partitions
    for (const p of state.partitions) {
        partitions.set(p.name, {
            ...p,
            status: p.status as PartitionStatus,
            nodes: [],
            jobs: []
        });
    }

    // 2. Init Nodes
    for (const n of state.nodes) {
        nodes.set(n.name, {
            ...n,
            status: n.status as NodeStatus,
            partitions: [],
            resources: new Map(),
            updated_at: n.updated_at,
            jobs: [],
            cpus: n.cpus,
            memory: n.real_memory // Map Backend 'real_memory' to Node 'memory'
        });
    }

    // 3. Init Jobs
    for (const j of state.jobs) {
        // Partition linking
        let part = partitions.get(j.partition);
        if (!part) {
            // Create stub if missing
            part = {
                name: j.partition,
                status: PartitionStatus.Unknown,
                access_qos: null,
                resource_qos: null,
                updated_at: j.updated_at,
                nodes: [],
                jobs: []
            };
            partitions.set(part.name, part);
        }

        const job: Job = {
            ...j,
            status: j.status as JobStatus,
            partition: part,
            allocations: [],
            resources: new Map()
        };
        jobs.set(j.job_id, job);
        part.jobs.push(job);
    }

    // 4. Populate Node Resources
    for (const nr of state.node_resources) {
        const node = nodes.get(nr.node);
        if (node) {
            node.resources.set(nr.resource, {
                resource: nr.resource,
                available: nr.available,
                total: nr.total
            });
        }
    }

    // 5. Populate Node Partitions
    // Using typo 'node_parittions' as defined in BackendClusterState type in this file
    for (const np of state.node_parittions) {
        const node = nodes.get(np.node);
        const part = partitions.get(np.partition);
        if (node && part) {
            node.partitions.push(part);
            part.nodes.push(node);
        }
    }

    // 6. Populate Job Resources
    for (const jr of state.job_resources) {
        const job = jobs.get(jr.job);
        if (job) {
            job.resources.set(jr.resource, {
                requested: jr.requested,
                allocated: jr.allocated
            });
        }
    }

    // 7. Populate Job Allocations
    for (const ja of state.job_allocations) {
        const job = jobs.get(ja.job);
        const node = nodes.get(ja.node);
        if (job && node) {
            job.allocations.push({
                node: node,
                resource: ja.resource,
                count: ja.allocated
            });
            if (!node.jobs.includes(job)) {
                node.jobs.push(job);
            }
        }
    }

    return {
        nodes: Array.from(nodes.values()),
        jobs: Array.from(jobs.values()),
        partitions: Array.from(partitions.values()),
        updated_at: state.updated_at
    };
}