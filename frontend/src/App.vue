<script setup>
import { ref } from 'vue'
import Dashboard from './components/Dashboard.vue'
import NodesView from './components/NodesView.vue'
import JobsView from './components/JobsView.vue'

const currentView = ref('dashboard')

const views = {
  dashboard: Dashboard,
  nodes: NodesView,
  jobs: JobsView
}
</script>

<template>
  <div class="min-h-screen bg-slurm-dark font-sans text-slate-300">
    <nav class="bg-slurm-panel border-b border-slate-700">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <div class="flex items-center">
            <div class="flex-shrink-0 text-slurm-accent font-bold text-xl">
              SLURM Monitor
            </div>
            <div class="hidden md:block">
              <div class="ml-10 flex items-baseline space-x-4">
                <button 
                  v-for="(component, name) in views" 
                  :key="name"
                  @click="currentView = name"
                  class="px-3 py-2 rounded-md text-sm font-medium capitalize transition-colors"
                  :class="currentView === name ? 'bg-slate-700 text-white' : 'text-slate-300 hover:bg-slate-700 hover:text-white'"
                >
                  {{ name }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </nav>

    <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
      <component :is="views[currentView]" />
    </main>
  </div>
</template>
