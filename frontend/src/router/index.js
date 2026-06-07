import { createRouter, createWebHistory } from 'vue-router'
import { ADMIN_PERMISSIONS } from '@/config/adminPermissions'
import { useAuthStore } from '@/stores/auth'

const routes = [
  { path: '/login', component: () => import('@/views/LoginView.vue'), meta: { public: true } },
  {
    path: '/',
    component: () => import('@/layouts/AdminLayout.vue'),
    children: [
      { path: '', redirect: '/dashboard' },
      {
        path: 'dashboard',
        component: () => import('@/views/DashboardView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.DASHBOARD_VIEW }
      },
      {
        path: 'categories',
        component: () => import('@/views/CategoryView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.CATEGORY_LIST_VIEW }
      },
      {
        path: 'products',
        component: () => import('@/views/ProductView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.GOODS_VIEW }
      },
      { path: 'goods', redirect: '/products' },
      {
        path: 'orders',
        component: () => import('@/views/OrderView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.ORDER_LIST_VIEW }
      },
      {
        path: 'users',
        component: () => import('@/views/WechatUserView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.WECHAT_USER_LIST_VIEW }
      },
      {
        path: 'subscriptions',
        component: () => import('@/views/SubscriptionView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.SUBSCRIPTION_VIEW }
      },
      {
        path: 'subscription-records',
        component: () => import('@/views/SubscriptionRecordView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.SUBSCRIPTION_RECORD_VIEW }
      },
      {
        path: 'logs',
        component: () => import('@/views/LogView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.LOGS_VIEW }
      },
      {
        path: 'admins',
        component: () => import('@/views/AdminUserView.vue'),
        meta: { permission: ADMIN_PERMISSIONS.ADMIN_USER_VIEW }
      }
    ]
  }
]

const router = createRouter({ history: createWebHistory(), routes })
router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (to.meta.public) {
    if (to.path !== '/login' || !auth.isAuthenticated) return true
    return await auth.validateSession() ? auth.firstAccessiblePath || '/dashboard' : true
  }
  if (!auth.isAuthenticated) return `/login?redirect=${encodeURIComponent(to.fullPath)}`
  if (!auth.user) {
    const valid = await auth.validateSession()
    if (!valid) return `/login?redirect=${encodeURIComponent(to.fullPath)}`
  }
  if (to.meta.permission && !auth.canAccess(to.meta.permission)) {
    const fallback = auth.firstAccessiblePath
    return fallback && fallback !== to.path ? fallback : false
  }
  return true
})
export default router
