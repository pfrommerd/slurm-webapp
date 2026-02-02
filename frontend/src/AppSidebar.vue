<script setup lang="ts">
import {
  Sidebar,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarContent,
  SidebarFooter,
} from "@/components/ui/sidebar"
import { IconDatabase, IconDashboard, IconStack2, IconInfoCircle, IconCloudComputing, IconCircleHalf } from "@tabler/icons-vue"
import NavItem from "@/components/NavItem.vue"
import NavGroup from "@/components/NavGroup.vue"
import Separator from "@/components/ui/separator/Separator.vue";
import NavUser from "@/components/NavUser.vue";
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
            <NavItem name="Nodes" url="/nodes" :icon="IconCloudComputing" />
            <NavItem name="Partitions" url="/partitions" :icon="IconCircleHalf" />
            <NavItem name="About" url="/about" :icon="IconInfoCircle" />
        </NavGroup>
        <div class="flex justify-center mx-3">
        <Separator class="data-[orientation=horizontal]"/>
        </div>
        <NavGroup label="Recents">
            <NavItem name="job 12345" url="/job/12345" :icon="IconStack2" />
            <NavItem name="job 12346" url="/job/12346" :icon="IconStack2" />
            <NavItem name="node1232" url="/node/1232" :icon="IconCloudComputing" />
        </NavGroup>
      </SidebarContent>
      <SidebarFooter>
        <NavUser/>
      </SidebarFooter>
    </Sidebar>
</template>