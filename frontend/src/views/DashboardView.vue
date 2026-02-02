<script setup lang="ts">
  import AreaChartCard from '@/components/AreaChartCard.vue';
  import TableCard from '@/components/TableCard.vue';

  import { chartConfig, chartData, jobColumns, jobData } from './mock-data';

</script>

<template>
  <div class="flex flex-1 flex-col">
    <div class="@container/main flex flex-1 flex-col gap-2">
      <div class="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
        <div class="*:data-[slot=card]:from-primary/5
                    *:data-[slot=card]:to-card
                    dark:*:data-[slot=card]:bg-card
                    grid grid-cols-1 gap-4 px-4
                    *:data-[slot=card]:shadow-xs
                    lg:px-6
                    @xl/main:grid-cols-1">
          <TableCard name="Recent Jobs" description="Latest jobs in the queue."
            :columns="jobColumns" :data="jobData"/>
        </div>
        <div class="*:data-[slot=card]:from-primary/5
                    *:data-[slot=card]:to-card
                    dark:*:data-[slot=card]:bg-card
                    grid grid-cols-1 gap-4 px-4
                    *:data-[slot=card]:shadow-xs
                    lg:px-6
                    @xl/main:grid-cols-1
                    @5xl/main:grid-cols-2">
            <AreaChartCard name="Job Queue" description="Executing and pending jobs."
                :data="chartData" :config="chartConfig"/>
            <AreaChartCard name="Node Availability" description="Nodes by status."
                :data="chartData" :config="chartConfig"/>
        </div>
      </div>
    </div>
  </div>
</template>