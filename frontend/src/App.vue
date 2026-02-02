<script setup lang="ts">
import {
  SidebarInset,
  SidebarProvider,
} from "@/components/ui/sidebar"
import AppSidebar from "@/AppSidebar.vue"
import AppHeader from "@/AppHeader.vue"
</script>

<template>
  <div :style=" {
      '--sidebar-width': 'calc(var(--spacing) * 72)',
      '--header-height': 'calc(var(--spacing) * 12)',
    }">
  <RouterView v-if="$route.meta.fullpage" />
  <SidebarProvider v-else
  >
    <AppSidebar />
    <SidebarInset>
      <AppHeader/>
      <RouterView />
    </SidebarInset>
  </SidebarProvider>
  </div>
</template>