const ADMIN_TOKEN_KEY = 'gs_store_system_admin_token'

export function getAdminToken() {
  const token = localStorage.getItem(ADMIN_TOKEN_KEY)
  if (token) return token

  return ''
}

export function setAdminToken(token) {
  if (token) localStorage.setItem(ADMIN_TOKEN_KEY, token)
  else localStorage.removeItem(ADMIN_TOKEN_KEY)
}
