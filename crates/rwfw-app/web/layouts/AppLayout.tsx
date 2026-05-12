import React from 'react'
import { Link, usePage } from '@inertiajs/react'

interface NavItem {
  label: string
  href: string
  icon?: string
}

interface ModuleNav {
  name: string
  nav_items: NavItem[]
}

interface SharedData {
  auth?: { user: { name: string; email: string } }
  flash?: { success?: string; error?: string; info?: string }
  modules?: ModuleNav[]
}

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const { props } = usePage<SharedData & Record<string, any>>()
  const { auth, flash, modules = [] } = props as unknown as SharedData

  return (
    <div className="min-h-screen flex">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-900 text-white p-4 flex flex-col">
        <div className="mb-8">
          <h1 className="text-xl font-bold">RWFW</h1>
          <p className="text-gray-400 text-sm">Modular Framework</p>
        </div>
        <nav className="flex-1 space-y-1">
          {modules.map((mod) => (
            <div key={mod.name}>
              {mod.nav_items.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="block px-3 py-2 rounded-md text-sm hover:bg-gray-800 transition-colors"
                >
                  {item.label}
                </Link>
              ))}
            </div>
          ))}
        </nav>
        {auth?.user && (
          <div className="border-t border-gray-700 pt-4 mt-4">
            <p className="text-sm text-gray-300">{auth.user.name}</p>
            <p className="text-xs text-gray-500">{auth.user.email}</p>
          </div>
        )}
      </aside>

      {/* Main content */}
      <main className="flex-1 p-8">
        {/* Flash messages */}
        {flash?.success && (
          <div className="mb-4 p-4 bg-green-100 text-green-700 rounded-md">
            {flash.success}
          </div>
        )}
        {flash?.error && (
          <div className="mb-4 p-4 bg-red-100 text-red-700 rounded-md">
            {flash.error}
          </div>
        )}
        {flash?.info && (
          <div className="mb-4 p-4 bg-blue-100 text-blue-700 rounded-md">
            {flash.info}
          </div>
        )}
        {children}
      </main>
    </div>
  )
}
