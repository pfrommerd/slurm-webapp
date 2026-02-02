<script setup lang="ts">
import {
  Sidebar,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarContent,
} from "@/components/ui/sidebar"
import { IconDatabase, IconDashboard, IconInfoHexagon, IconStack2 } from "@tabler/icons-vue"
import NavItem from "@/components/NavItem.vue"
import NavGroup from "@/components/NavGroup.vue"
</script>

<template>
    <Sidebar collapsible="icon" variant="sidebar">
      <SidebarHeader class="border-b h-(--header-height)">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton as-child
              class="data-[slot=sidebar-menu-button]:!p-1.5"
            >
              <router-link to="/">
                <IconDatabase class="!size-5" />
                <span class="text-base font-semibold">Cluster Monitor</span>
              </router-link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavGroup label="Home">
            <NavItem name="Dashboard" url="/" :icon="IconDashboard" />
            <NavItem name="Jobs" url="/jobs" :icon="IconStack2" />
            <NavItem name="About" url="/about" :icon="IconInfoHexagon" />
        </NavGroup>
      </SidebarContent>
      <!-- <SidebarFooter>
        <NavUser :user="data.user" />
      </SidebarFooter> -->
    </Sidebar>
</template>