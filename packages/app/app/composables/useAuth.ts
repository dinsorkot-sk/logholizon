export type AuthUser = { id: string; username: string; role: string }

export const useAuth = () => {
  const user = useState<AuthUser | null>('auth-user', () => null)
  const loading = useState<boolean>('auth-loading', () => true)

  async function fetchMe() {
    try {
      // useRequestFetch forwards the browser cookie to internal API routes
      // during SSR; on the client it behaves like $fetch.
      const requestFetch = useRequestFetch()
      const me = await requestFetch<AuthUser>('/api/auth/me')
      user.value = me
    } catch {
      user.value = null
    } finally {
      loading.value = false
    }
  }

  async function login(username: string, password: string) {
    const session = await $fetch<{ token: string; user: AuthUser }>('/api/auth/login', {
      method: 'POST',
      body: { username, password }
    })
    user.value = session.user
    return session.user
  }

  async function logout() {
    try {
      await $fetch('/api/auth/logout', { method: 'POST' })
    } finally {
      user.value = null
    }
  }

  return { user, loading, fetchMe, login, logout }
}