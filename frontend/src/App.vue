<script setup lang="ts">
import {
  SidebarInset,
  SidebarProvider,
} from "@/components/ui/sidebar"
import AppSidebar from "@/AppSidebar.vue"
import AppHeader from "@/AppHeader.vue"

</script>

<template>
  <SidebarProvider
    :style=" {
      '--sidebar-width': 'calc(var(--spacing) * 72)',
      '--header-height': 'calc(var(--spacing) * 12)',
    }"
  >
    <AppSidebar />
    <SidebarInset>
      <AppHeader/>
      <RouterView />
    </SidebarInset>
  </SidebarProvider>
</template>