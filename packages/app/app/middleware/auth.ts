export default defineNuxtRouteMiddleware(async (to) => {
  const { user, fetchMe } = useAuth()

  // Always re-validate the session on navigation (SSR has no shared state).
  await fetchMe()

  const isLoginPage = to.path === '/login'
  if (!user.value && !isLoginPage) {
    return navigateTo('/login')
  }
  if (user.value && isLoginPage) {
    return navigateTo('/dashboard')
  }
})