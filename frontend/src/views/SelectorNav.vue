<script setup lang="ts">
import ComboSelector from '@/components/ComboSelector.vue';

const userOptions = [
    { value: 'any', label: 'Any' },
    { value: 'user1', label: 'User 1' },
    { value: 'user2', label: 'User 2' },
]
const partitionOptions = [
    { value: 'any', label: 'Any' },
    { value: 'partition1', label: 'Partition 1' },
    { value: 'partition2', label: 'Partition 2' },
]
</script>

<template>
    <ComboSelector :options="partitionOptions" :value="''" placeholder="Any" prefix="Partition"/>
    <ComboSelector :options="userOptions" :value="''" placeholder="Any" prefix="User"/>
</template>