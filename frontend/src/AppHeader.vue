<script setup lang="ts">
import { Separator } from '@/components/ui/separator'
import { SidebarTrigger } from '@/components/ui/sidebar'
import { Badge } from '@/components/ui/badge'
</script>

<template>
  <header class="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
    <div class="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
      <SidebarTrigger class="-ml-1" />
      <Separator
        orientation="vertical"
        class="mx-2 data-[orientation=vertical]:h-4"
      />
      <h1 class="text-base font-medium min-w-24">
        {{ $route.name }}
      </h1>
      <div class="ml-1"></div>
      <router-view name="nav" />
      <div class="flex items-center gap-2 ml-auto"></div>
      <Badge variant="outline" class="bg-green-50 text-green-700 dark:bg-green-950 dark:text-green-300">Connected</Badge>
    </div>
  </header>
</template>
