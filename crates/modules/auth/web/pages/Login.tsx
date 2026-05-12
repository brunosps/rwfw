import { useForm, usePage } from '@inertiajs/react'
import AuthLayout from '@app/layouts/AuthLayout'

type SsoProvider = {
  name: string
  displayName: string
  loginUrl: string
}

export default function Login() {
  const { props } = usePage()
  const ssoProviders = ((props as any).ssoProviders ?? []) as SsoProvider[]
  const { data, setData, post, processing, errors } = useForm({
    email: '',
    password: '',
  })

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    post('/auth/login')
  }

  return (
    <AuthLayout>
      <form onSubmit={handleSubmit} className="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-sm">
        <h2 className="text-2xl font-bold text-center">Sign In</h2>

        {ssoProviders.length > 0 && (
          <div className="space-y-3">
            {ssoProviders.map((provider) => (
              <a
                key={provider.name}
                href={provider.loginUrl}
                className="block w-full rounded-md border border-gray-300 px-4 py-2 text-center text-sm font-semibold text-gray-800 hover:bg-gray-50"
              >
                Continue with {provider.displayName}
              </a>
            ))}
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-200" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-white px-2 text-gray-500">or sign in locally</span>
              </div>
            </div>
          </div>
        )}

        <div>
          <label htmlFor="email" className="block text-sm font-medium text-gray-700">
            Email
          </label>
          <input
            id="email"
            type="email"
            value={data.email}
            onChange={(e) => setData('email', e.target.value)}
            className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            required
          />
          {errors.email && <p className="mt-1 text-sm text-red-600">{errors.email}</p>}
        </div>

        <div>
          <label htmlFor="password" className="block text-sm font-medium text-gray-700">
            Password
          </label>
          <input
            id="password"
            type="password"
            value={data.password}
            onChange={(e) => setData('password', e.target.value)}
            className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            required
          />
          {errors.password && <p className="mt-1 text-sm text-red-600">{errors.password}</p>}
        </div>

        <button
          type="submit"
          disabled={processing}
          className="w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
        >
          {processing ? 'Signing in...' : 'Sign In'}
        </button>

        <p className="text-center text-sm text-gray-600">
          Don't have an account?{' '}
          <a href="/auth/register" className="text-blue-600 hover:underline">
            Register
          </a>
        </p>
      </form>
    </AuthLayout>
  )
}
