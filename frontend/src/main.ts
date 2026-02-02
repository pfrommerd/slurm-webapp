import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { createWebHistory, createRouter } from 'vue-router'
import LoginPage from './LoginPage.vue'
import DashboardView from './views/DashboardView.vue'
import Job from './views/Job.vue'
import Jobs from './views/Jobs.vue'
import Node from './views/Node.vue'
import Nodes from './views/Nodes.vue'
import PartitionsView from './views/PartitionsView.vue'
import AboutView from './views/AboutView.vue'
import NotFound from './views/NotFound.vue'
import SelectorNav from './views/SelectorNav.vue'

const routes = [
    { path: '/login', name: 'Login', meta: { fullpage: true }, component: LoginPage },
    { path: '/signup', name: 'Create Account', meta: { fullpage: true }, component: LoginPage },
    { path: '/', name: 'Dashboard', components: { default: DashboardView, nav: SelectorNav } },
    { path: '/jobs', name: 'Jobs', components: { default: Jobs, nav: SelectorNav } },
    { path: '/job/:id', name: 'Job', components: { default: Job, nav: SelectorNav } },
    { path: '/nodes', name: 'Nodes', components: { default: Nodes, nav: SelectorNav } },
    { path: '/node/:id', name: 'Node', components: { default: Node, nav: SelectorNav } },
    { path: '/partitions', name: 'Partitions', components: { default: PartitionsView, nav: SelectorNav } },
    { path: '/about', name: 'About', component: AboutView },
    { path: '/:pathMatch(.*)*', name: 'NotFound', component: NotFound },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
})

createApp(App).use(router).mount('#app')