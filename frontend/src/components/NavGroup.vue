<script setup lang="ts">
import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
} from '@/components/ui/sidebar'

defineProps<{
    label: string
}>()
</script>

<template>
  <SidebarGroup>
    <SidebarGroupLabel>{{ label }}</SidebarGroupLabel>
    <SidebarGroupContent class="flex flex-col gap-2">
      <SidebarMenu>
        <slot />
      </SidebarMenu>
    </SidebarGroupContent>
  </SidebarGroup>
</template>