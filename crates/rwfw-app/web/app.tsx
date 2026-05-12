import { createInertiaApp } from '@inertiajs/react'
import { createRoot, hydrateRoot } from 'react-dom/client'
import './app.css'

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob<{ default: React.ComponentType<any> }>(
      '../../modules/*/web/pages/**/*.tsx',
      { eager: false }
    )

    const [module, ...rest] = name.split('/')
    const page = rest.join('/')
    const key = `../../modules/${module}/web/pages/${page}.tsx`

    if (!pages[key]) {
      throw new Error(`Page not found: ${name} (looked for ${key})`)
    }

    return pages[key]()
  },
  setup({ el, App, props }) {
    if (el.innerHTML) {
      // SSR: hydrate
      hydrateRoot(el, <App {...props} />)
    } else {
      // CSR: create root
      createRoot(el!).render(<App {...props} />)
    }
  },
})
