import http from './http'

export function login(data) { return http.post('/auth/login', data) }
export function logout() { return http.post('/auth/logout') }
export function fetchAccessCodes() { return http.get('/auth/codes') }
export function fetchDashboard(params) { return http.get('/api/admin/dashboard', { params }) }
export function fetchCategories() { return http.get('/api/admin/categories') }
export function createCategory(data) { return http.post('/api/admin/categories', data) }
export function updateCategory(id, data) { return http.put(`/api/admin/categories/${id}`, data) }
export function deleteCategory(id) { return http.delete(`/api/admin/categories/${id}`) }
export function fetchGoods(params) { return http.get('/api/admin/goods', { params }) }
export function fetchGoodsDetail(id) { return http.get(`/api/admin/goods/${id}`) }
export function createGoods(data) { return http.post('/api/admin/goods', data) }
export function updateGoods(id, data) { return http.put(`/api/admin/goods/${id}`, data) }
export function deleteGoods(id) { return http.delete(`/api/admin/goods/${id}`) }
export function fetchOrders(params) { return http.get('/api/admin/orders', { params }) }
export function fetchOrder(id) { return http.get(`/api/admin/orders/${id}`) }
export function updateOrderStatus(id, data) { return http.put(`/api/admin/orders/${id}/status`, data) }
export function fetchAdminUsers() { return http.get('/api/admin/admin-users') }
export function fetchPermissionCatalog() { return http.get('/api/admin/permissions') }
export function updateAdminUserPermissions(id, data) { return http.put(`/api/admin/admin-users/${id}/permissions`, data) }
export function fetchWechatUsers(params) { return http.get('/api/admin/wechat/users', { params }) }
export function fetchWechatUser(id) { return http.get(`/api/admin/wechat/users/${id}`) }
export function createWechatUser(data) { return http.post('/api/admin/wechat/users', data) }
export function updateWechatUser(id, data) { return http.put(`/api/admin/wechat/users/${id}`, data) }
export function deleteWechatUser(id) { return http.delete(`/api/admin/wechat/users/${id}`) }
export function checkIdCard(data) { return http.post('/api/admin/wechat/users/check-id-card', data) }
export function fetchPaymentPassword(openid) { return http.get(`/api/admin/wechat/users/${openid}/payment-password`) }
export function fetchSubscriptionRecords(params) { return http.get('/api/admin/subscription/records', { params }) }
export function triggerAutoRecharge() { return http.post('/api/admin/subscription/auto-recharge') }
export function fetchBalance(openid) { return http.get(`/api/admin/wechat/users/${openid}/balance`) }
export function fetchBalanceTransactions(openid) { return http.get(`/api/admin/wechat/users/${openid}/balance/transactions`) }
export function fetchRecentLogs(params) { return http.get('/api/admin/logs/recent', { params }) }
export function fetchUploadSignature(filename) { return http.get('/api/admin/upload/signature', { params: { filename } }) }
