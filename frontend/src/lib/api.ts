import type { ClusterStatus, Node, Job, Partition } from './types';

const API_BASE = 'http://localhost:3000/api';

export async function fetchStatus(): Promise<ClusterStatus> {
    const res = await fetch(`${API_BASE}/status`);
    if (!res.ok) throw new Error('Failed to fetch status');
    return res.json();
}

export async function fetchNodes(): Promise<Node[]> {
    const res = await fetch(`${API_BASE}/nodes`);
    if (!res.ok) throw new Error('Failed to fetch nodes');
    return res.json();
}

export async function fetchJobs(): Promise<Job[]> {
    const res = await fetch(`${API_BASE}/jobs`);
    if (!res.ok) throw new Error('Failed to fetch jobs');
    return res.json();
}

export async function fetchPartitions(): Promise<Partition[]> {
    const res = await fetch(`${API_BASE}/partitions`);
    if (!res.ok) throw new Error('Failed to fetch partitions');
    return res.json();
}

export async function fetchUpdatedAt(): Promise<string | null> {
    const res = await fetch(`${API_BASE}/updated_at`);
    if (!res.ok) throw new Error('Failed to fetch updated_at');
    return res.json();
}

