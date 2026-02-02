import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { createWebHistory, createRouter } from 'vue-router'
import DashboardView from './views/DashboardView.vue'
import JobsView from './views/JobsView.vue'
import AboutView from './views/AboutView.vue'
import NotFound from './views/NotFound.vue'
import SelectorNav from './views/SelectorNav.vue'

const routes = [
    { path: '/', name: 'Dashboard', components: { default: DashboardView, nav: SelectorNav } },
    { path: '/jobs', name: 'Jobs', components: { default: JobsView, nav: SelectorNav } },
    { path: '/about', name: 'About', component: AboutView },
    { path: '/:pathMatch(.*)*', name: 'NotFound', component: NotFound },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
})

createApp(App).use(router).mount('#app')
