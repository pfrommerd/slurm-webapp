<script setup lang="ts">
   import { jobData, jobColumns } from './mock-data';
   import TableCard from '@/components/TableCard.vue';
</script>

<template>
    <div class="m-6">
        <TableCard name="Nodes" description="List of nodes in the cluster."
            :columns="jobColumns" :data="jobData"/>
    </div>
</template>