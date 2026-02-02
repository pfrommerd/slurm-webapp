<script setup lang="ts">
   import { jobColumns, jobData } from './mock-data';
   import TableCard from '@/components/TableCard.vue';
</script>

<template>
    <div class="m-6">
        <TableCard name="Recent Jobs" description="Latest jobs in the queue."
            :columns="jobColumns" :data="jobData"/>
    </div>
</template>