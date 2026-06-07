import { defineStore } from 'pinia'
import { fetchAccessCodes, login, logout } from '@/api/admin'
import { canAccessPermission, firstAccessibleAdminPath } from '@/config/adminPermissions'
import { getAdminToken, setAdminToken } from '@/utils/authStorage'

export const useAuthStore = defineStore('auth', {
  state: () => ({ token: getAdminToken(), user: null, accessCodes: [] }),
  getters: {
    isAuthenticated: (state) => Boolean(state.token),
    canAccess: (state) => (permission) => canAccessPermission(state.accessCodes, state.user?.isAdmin, permission),
    firstAccessiblePath: (state) => firstAccessibleAdminPath(state.accessCodes, state.user?.isAdmin)
  },
  actions: {
    persistToken(token) {
      this.token = token || ''
      setAdminToken(token)
    },
    clearLocalAuth() {
      this.user = null
      this.accessCodes = []
      this.persistToken('')
    },
    async login(payload) {
      const data = await login(payload)
      this.persistToken(data.access_token)
      await this.loadProfile()
      return data
    },
    async loadProfile() {
      if (!this.token) return null
      const data = await fetchAccessCodes()
      this.user = { username: data.username, role: data.role, isAdmin: data.is_admin }
      this.accessCodes = data.codes || []
      return data
    },
    async validateSession() {
      if (!this.token) return false
      try {
        await this.loadProfile()
        return true
      } catch {
        this.clearLocalAuth()
        return false
      }
    },
    async logout() {
      try { if (this.token) await logout() } finally { this.clearLocalAuth() }
    }
  }
})
