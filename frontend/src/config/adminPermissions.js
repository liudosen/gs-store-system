export const ADMIN_PERMISSIONS = Object.freeze({
  DASHBOARD_VIEW: 'dashboard:view',
  CATEGORY_LIST_VIEW: 'category:list:view',
  GOODS_VIEW: 'goods:view',
  ORDER_LIST_VIEW: 'order:list:view',
  WECHAT_USER_LIST_VIEW: 'wechat-user:list:view',
  ADMIN_USER_VIEW: 'admin:user:view',
  LOGS_VIEW: 'logs:view',
  SUBSCRIPTION_VIEW: 'subscription:view',
  SUBSCRIPTION_RECORD_VIEW: 'subscription:record:view'
})

export const ADMIN_ROUTE_PERMISSIONS = Object.freeze({
  dashboard: ADMIN_PERMISSIONS.DASHBOARD_VIEW,
  categories: ADMIN_PERMISSIONS.CATEGORY_LIST_VIEW,
  products: ADMIN_PERMISSIONS.GOODS_VIEW,
  goods: ADMIN_PERMISSIONS.GOODS_VIEW,
  orders: ADMIN_PERMISSIONS.ORDER_LIST_VIEW,
  users: ADMIN_PERMISSIONS.WECHAT_USER_LIST_VIEW,
  subscriptions: ADMIN_PERMISSIONS.SUBSCRIPTION_VIEW,
  'subscription-records': ADMIN_PERMISSIONS.SUBSCRIPTION_RECORD_VIEW,
  logs: ADMIN_PERMISSIONS.LOGS_VIEW,
  admins: ADMIN_PERMISSIONS.ADMIN_USER_VIEW
})

export const ADMIN_ROUTE_ORDER = Object.freeze([
  'dashboard',
  'categories',
  'products',
  'orders',
  'subscriptions',
  'subscription-records',
  'users',
  'admins',
  'logs'
])

export function canAccessPermission(accessCodes, isAdmin, permission) {
  if (!permission) return true
  if (isAdmin) return true
  return Array.isArray(accessCodes) && accessCodes.includes(permission)
}

export function firstAccessibleAdminPath(accessCodes, isAdmin) {
  const firstRoute = ADMIN_ROUTE_ORDER.find((routeName) =>
    canAccessPermission(accessCodes, isAdmin, ADMIN_ROUTE_PERMISSIONS[routeName])
  )
  return firstRoute ? `/${firstRoute}` : null
}
