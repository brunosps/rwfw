import AppLayout from '@app/layouts/AppLayout'

interface Props {
  title: string
  description: string
}

export default function HomeIndex({ title, description }: Props) {
  return (
    <AppLayout>
      <div className="max-w-4xl">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">{title}</h1>
        <p className="text-lg text-gray-600 mb-8">{description}</p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="p-6 bg-white rounded-lg shadow-sm border">
            <h2 className="text-xl font-semibold mb-2">Modular Architecture</h2>
            <p className="text-gray-600">
              Each module is self-contained with its own routes, models, and frontend.
            </p>
          </div>
          <div className="p-6 bg-white rounded-lg shadow-sm border">
            <h2 className="text-xl font-semibold mb-2">React + SSR</h2>
            <p className="text-gray-600">
              Server-side rendering via V8, hydrated with React on the client.
            </p>
          </div>
          <div className="p-6 bg-white rounded-lg shadow-sm border">
            <h2 className="text-xl font-semibold mb-2">Inertia.js Protocol</h2>
            <p className="text-gray-600">
              SPA navigation without a separate API layer.
            </p>
          </div>
          <div className="p-6 bg-white rounded-lg shadow-sm border">
            <h2 className="text-xl font-semibold mb-2">Single Binary Deploy</h2>
            <p className="text-gray-600">
              All assets embedded in one Rust binary for simple deployment.
            </p>
          </div>
        </div>
      </div>
    </AppLayout>
  )
}
