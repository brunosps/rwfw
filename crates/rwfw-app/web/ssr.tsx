import { createInertiaApp } from '@inertiajs/react'
import type { ComponentType } from 'react'
import { renderToString } from 'react-dom/server.browser'

// SSR render function called from Rust V8 engine
// Receives the page JSON as argument and returns HTML string
export async function render(pageJson: string): Promise<string> {
  const page = JSON.parse(pageJson)

  let html = ''

  await createInertiaApp({
    page,
    resolve: (name) => {
      const pages = import.meta.glob<{ default: ComponentType<any> }>(
        '../../modules/*/web/pages/**/*.tsx',
        { eager: true }
      )

      const [module, ...rest] = name.split('/')
      const pagePath = rest.join('/')
      const key = `../../modules/${module}/web/pages/${pagePath}.tsx`

      if (!pages[key]) {
        throw new Error(`SSR: Page not found: ${name} (looked for ${key})`)
      }

      return pages[key]
    },
    setup({ App, props }) {
      html = renderToString(<App {...props} />)
      return <App {...props} />
    },
  })

  return html
}
