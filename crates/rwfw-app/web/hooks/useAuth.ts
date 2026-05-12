import { usePage } from '@inertiajs/react'

export function useAuth() {
  const { props } = usePage()
  const auth = (props as any).auth
  return {
    user: auth?.user ?? null,
    roles: auth?.roles ?? [],
    permissions: auth?.permissions ?? [],
    isAuthenticated: !!auth?.user,
    can: (permission: string) =>
      (auth?.roles ?? []).includes('admin') || (auth?.permissions ?? []).includes(permission),
    hasRole: (role: string) => (auth?.roles ?? []).includes(role),
  }
}
