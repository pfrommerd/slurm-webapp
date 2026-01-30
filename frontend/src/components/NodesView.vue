<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { fetchNodes } from '../api'

const nodes = ref([])
let interval = null

const update = async () => {
    try {
        nodes.value = await fetchNodes()
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

const getStateColor = (state) => {
    if (state.includes('down')) return 'bg-slurm-error border-slurm-error'
    if (state.includes('alloc') || state.includes('mix')) return 'bg-slurm-accent border-slurm-accent'
    if (state.includes('idle')) return 'bg-slurm-success border-slurm-success'
    return 'bg-slate-600 border-slate-600'
}
</script>

<template>
    <div>
        <h1 class="text-2xl font-bold text-white mb-6">Nodes</h1>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
            <div v-for="node in nodes" :key="node.name" 
                class="bg-slurm-panel border rounded-lg p-4 relative overflow-hidden transition-all hover:scale-105"
                :class="getStateColor(node.state).replace('bg-', 'border-').replace('border-', 'border-opacity-50 ')"
            >
                <div class="flex justify-between items-start mb-2">
                    <span class="font-mono font-bold text-white">{{ node.name }}</span>
                    <div class="h-3 w-3 rounded-full" :class="getStateColor(node.state)"></div>
                </div>
                <div class="text-xs text-slate-400 space-y-1">
                    <div>CPUs: <span class="text-slate-200">{{ node.cpus }}</span></div>
                    <div>Mem: <span class="text-slate-200">{{ (node.real_memory / 1024).toFixed(0) }} GB</span></div>
                    <div class="uppercase text-[10px] tracking-wide mt-1 opacity-70">{{ node.state }}</div>
                </div>
            </div>
        </div>
    </div>
</template>
