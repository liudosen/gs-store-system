import axios from 'axios'
import { Message } from '@arco-design/web-vue'
import { getAdminToken, setAdminToken } from '@/utils/authStorage'

const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '',
  timeout: 15000
})

const PUBLIC_AUTH_PATHS = new Set(['/auth/login'])
let authExpiredNotified = false

function resolvePath(config) {
  const base = config?.baseURL || window.location.origin
  return new URL(config?.url || '', base || window.location.origin).pathname
}

function clearAuthAndRedirect() {
  setAdminToken('')
  if (!authExpiredNotified) {
    authExpiredNotified = true
    Message.error('登录状态已过期，请重新登录')
  }
  if (window.location.pathname !== '/login') {
    const redirect = encodeURIComponent(`${window.location.pathname}${window.location.search}`)
    window.location.replace(`/login?redirect=${redirect}`)
  }
}

http.interceptors.request.use((config) => {
  if (PUBLIC_AUTH_PATHS.has(resolvePath(config))) {
    delete config.headers.Authorization
    return config
  }

  const token = getAdminToken()
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

http.interceptors.response.use(
  (response) => {
    const payload = response.data
    if (payload && typeof payload === 'object' && 'code' in payload) {
      if (![0, 200].includes(Number(payload.code))) {
        return Promise.reject(new Error(payload.message || '请求失败'))
      }
      return payload.data
    }
    return payload
  },
  (error) => {
    if (error?.response?.status === 401 && !PUBLIC_AUTH_PATHS.has(resolvePath(error.config))) {
      clearAuthAndRedirect()
    }
    const message = error?.response?.data?.message || error?.message || '网络请求失败，请稍后重试'
    return Promise.reject(new Error(message))
  }
)

export default http
