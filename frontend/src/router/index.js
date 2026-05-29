import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const routes = [
  { path: '/login', component: () => import('@/views/LoginView.vue'), meta: { public: true } },
  {
    path: '/',
    component: () => import('@/layouts/AdminLayout.vue'),
    children: [
      { path: '', redirect: '/dashboard' },
      { path: 'dashboard', component: () => import('@/views/DashboardView.vue') },
      { path: 'categories', component: () => import('@/views/CategoryView.vue') },
      { path: 'products', component: () => import('@/views/ProductView.vue') },
      { path: 'goods', redirect: '/products' },
      { path: 'orders', component: () => import('@/views/OrderView.vue') },
      { path: 'users', component: () => import('@/views/WechatUserView.vue') },
      { path: 'subscriptions', component: () => import('@/views/SubscriptionView.vue') },
      { path: 'subscription-records', component: () => import('@/views/SubscriptionRecordView.vue') },
      { path: 'logs', component: () => import('@/views/LogView.vue') },
      { path: 'admins', component: () => import('@/views/AdminUserView.vue') }
    ]
  }
]

const router = createRouter({ history: createWebHistory(), routes })
router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (to.meta.public) {
    if (to.path !== '/login' || !auth.isAuthenticated) return true
    return await auth.validateSession() ? '/dashboard' : true
  }
  if (!auth.isAuthenticated) return `/login?redirect=${encodeURIComponent(to.fullPath)}`
  if (!auth.user) {
    const valid = await auth.validateSession()
    if (!valid) return `/login?redirect=${encodeURIComponent(to.fullPath)}`
  }
  return true
})
export default router
