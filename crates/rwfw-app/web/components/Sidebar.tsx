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

export default function Sidebar() {
  const { props } = usePage()
  const modules = ((props as any).modules ?? []) as ModuleNav[]

  return (
    <aside className="w-64 bg-gray-900 text-white p-4 flex flex-col min-h-screen">
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
    </aside>
  )
}
