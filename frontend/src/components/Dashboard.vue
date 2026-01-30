<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { fetchStatus } from '../api'

const status = ref(null)
const error = ref(null)
let interval = null

const update = async () => {
    try {
        status.value = await fetchStatus()
        error.value = null
    } catch (e) {
        error.value = "Failed to connect to backend"
        console.error(e)
    }
}

onMounted(() => {
    update()
    interval = setInterval(update, 2000)
})

onUnmounted(() => {
    if (interval) clearInterval(interval)
})
</script>

<template>
    <div class="space-y-6">
        <header>
            <h1 class="text-3xl font-bold text-white">Cluster Overview</h1>
            <p v-if="status" class="text-slate-400 text-sm mt-1">Last updated: {{ new Date(status.updated_at).toLocaleString() }}</p>
            <p v-else-if="error" class="text-slurm-error mt-2">{{ error }}</p>
        </header>

        <div v-if="status" class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- Summary Cards -->
            <div class="bg-slurm-panel p-6 rounded-lg shadow-lg border border-slate-700">
                <h3 class="text-slate-400 text-sm uppercase font-bold tracking-wider">Active Jobs</h3>
                <div class="text-4xl font-bold text-slurm-accent mt-2">
                    {{ status.jobs.filter(j => j.state === 'RUNNING').length }}
                </div>
            </div>

            <div class="bg-slurm-panel p-6 rounded-lg shadow-lg border border-slate-700">
                <h3 class="text-slate-400 text-sm uppercase font-bold tracking-wider">Pending Jobs</h3>
                <div class="text-4xl font-bold text-slurm-warning mt-2">
                    {{ status.jobs.filter(j => j.state === 'PENDING').length }}
                </div>
            </div>

            <div class="bg-slurm-panel p-6 rounded-lg shadow-lg border border-slate-700">
                <h3 class="text-slate-400 text-sm uppercase font-bold tracking-wider">Nodes Up</h3>
                <div class="text-4xl font-bold text-slurm-success mt-2">
                    {{ status.nodes.filter(n => n.state !== 'down').length }} <span class="text-lg text-slate-500">/ {{ status.nodes.length }}</span>
                </div>
            </div>
        </div>

        <div v-if="status" class="bg-slurm-panel p-6 rounded-lg shadow-lg border border-slate-700">
            <h3 class="text-lg font-bold text-white mb-4">Partition Status</h3>
            <div class="overflow-x-auto">
                <table class="w-full text-left text-sm text-slate-300">
                    <thead class="bg-slate-800 text-xs uppercase text-slate-400">
                        <tr>
                            <th class="px-4 py-2">Name</th>
                            <th class="px-4 py-2">Nodes</th>
                            <th class="px-4 py-2">CPUs</th>
                            <th class="px-4 py-2">State</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-700">
                        <tr v-for="part in status.partitions" :key="part.name">
                            <td class="px-4 py-3 font-medium text-white">{{ part.name }}</td>
                            <td class="px-4 py-3">{{ part.total_nodes }}</td>
                            <td class="px-4 py-3">{{ part.total_cpus }}</td>
                            <td class="px-4 py-3">
                                <span :class="part.state === 'UP' ? 'text-slurm-success' : 'text-slurm-error'">
                                    {{ part.state }}
                                </span>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>
