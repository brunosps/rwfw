import { PageProps } from '@inertiajs/react'

declare module '@inertiajs/react' {
  interface PageProps {
    auth?: {
      user: {
        id: string
        name: string
        email: string
      }
      roles: string[]
      permissions: string[]
    }
    flash?: {
      success?: string
      error?: string
      info?: string
    }
    errors?: Record<string, string>
    csrf_token?: string
    modules?: Array<{
      name: string
      nav_items: Array<{
        label: string
        href: string
        icon?: string
      }>
    }>
  }
}
