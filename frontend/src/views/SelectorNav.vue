<script setup lang="ts">
import ComboSelector from '@/components/ComboSelector.vue';

const userOptions = [
    { value: 'all', label: 'All' },
    { value: 'running', label: 'Running' },
    { value: 'pending', label: 'Pending' },
    { value: 'completed', label: 'Completed' },
    { value: 'failed', label: 'Failed' },
]
const partitionOptions = [
    { value: 'all', label: 'All' },
    { value: 'idle', label: 'Idle' },
    { value: 'running', label: 'Running' },
    { value: 'down', label: 'Down' },
]
</script>

<template>
    <ComboSelector :options="partitionOptions" :value="''" placeholder="None" prefix="Partition"/>
    <ComboSelector :options="userOptions" :value="''" placeholder="None" prefix="User"/>
</template>