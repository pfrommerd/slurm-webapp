<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { fetchJobs } from '../api'

const jobs = ref([])
const filter = ref('')
let interval = null

const update = async () => {
    try {
        jobs.value = await fetchJobs()
    } catch (e) {
        console.error(e)
    }
}

onMounted(() => {
    update()
    interval = setInterval(update, 5000)
})

onUnmounted(() => {
    if (interval) clearInterval(interval)
})

const filteredJobs = computed(() => {
    if (!filter.value) return jobs.value
    const f = filter.value.toLowerCase()
    return jobs.value.filter(j => 
        j.user.toLowerCase().includes(f) || 
        j.job_id.toLowerCase().includes(f) || 
        j.partition.toLowerCase().includes(f)
    )
})

const getStatusClass = (state) => {
    switch (state) {
        case 'RUNNING': return 'text-slurm-accent bg-slurm-accent/10 px-2 py-1 rounded'
        case 'PENDING': return 'text-slurm-warning bg-slurm-warning/10 px-2 py-1 rounded'
        case 'COMPLETED': return 'text-slurm-success bg-slurm-success/10 px-2 py-1 rounded'
        case 'FAILED': return 'text-slurm-error bg-slurm-error/10 px-2 py-1 rounded'
        default: return 'text-slate-400'
    }
}
</script>

<template>
    <div class="space-y-4">
        <div class="flex justify-between items-center">
            <h1 class="text-2xl font-bold text-white">Jobs Queue</h1>
            <input 
                v-model="filter" 
                type="text" 
                placeholder="Search jobs..."
                class="bg-slate-800 border-slate-700 text-white rounded-md px-4 py-2 text-sm focus:ring-2 focus:ring-slurm-accent focus:outline-none"
            >
        </div>

        <div class="bg-slurm-panel rounded-lg shadow overflow-hidden border border-slate-700">
            <div class="overflow-x-auto">
                <table class="w-full text-left text-sm text-slate-300">
                    <thead class="bg-slate-800 text-xs uppercase text-slate-400">
                        <tr>
                            <th class="px-6 py-3">Job ID</th>
                            <th class="px-6 py-3">User</th>
                            <th class="px-6 py-3">Partition</th>
                            <th class="px-6 py-3">State</th>
                            <th class="px-6 py-3">Nodes</th>
                            <th class="px-6 py-3">Time</th>
                            <th class="px-6 py-3">Submitted</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-700">
                        <tr v-for="job in filteredJobs" :key="job.job_id" class="hover:bg-slate-700/50 transition-colors">
                            <td class="px-6 py-4 font-mono text-white">{{ job.job_id }}</td>
                            <td class="px-6 py-4">{{ job.user }}</td>
                            <td class="px-6 py-4"><span class="bg-slate-700 px-2 py-1 rounded text-xs">{{ job.partition }}</span></td>
                            <td class="px-6 py-4">
                                <span :class="getStatusClass(job.state)" class="text-xs font-bold">{{ job.state }}</span>
                            </td>
                            <td class="px-6 py-4">{{ job.num_nodes }} <span class="text-slate-500 text-xs">({{ job.num_cpus }} cpus)</span></td>
                            <td class="px-6 py-4">{{ job.time_limit || 'N/A' }}</td>
                            <td class="px-6 py-4 text-xs text-slate-400">{{ new Date(job.submit_time).toLocaleString() }}</td>
                        </tr>
                        <tr v-if="filteredJobs.length === 0">
                            <td colspan="7" class="px-6 py-8 text-center text-slate-500">No jobs found</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>
