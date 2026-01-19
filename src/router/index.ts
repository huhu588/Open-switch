import { createRouter, createWebHistory } from 'vue-router'
import ProvidersView from '@/views/ProvidersView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'providers',
      component: ProvidersView
    },
    {
      path: '/mcp',
      name: 'mcp',
      component: () => import('@/views/McpView.vue')
    },
    {
      path: '/skill',
      name: 'skill',
      component: () => import('@/views/SkillView.vue')
    },
    {
      path: '/backup',
      name: 'backup',
      component: () => import('@/views/BackupView.vue')
    },
    {
      path: '/status',
      name: 'status',
      component: () => import('@/views/StatusView.vue')
    }
  ]
})

export default router
