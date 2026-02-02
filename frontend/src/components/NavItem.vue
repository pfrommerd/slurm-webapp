<script setup lang="ts">
import type { Component } from "vue"

import {
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'

defineProps<{
  name: string
  url: string
  icon?: Component
}>()
</script>


<template>
    <SidebarMenuItem>
        <SidebarMenuButton as-child>
            <router-link :to="url">
                <component :is="icon" v-if="icon" class="!size-5" />
                {{ name }}
            </router-link>
        </SidebarMenuButton>
    </SidebarMenuItem>
</template>