import { useForm, Link } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

export default function PostCreate() {
  const { data, setData, post, processing, errors } = useForm({
    title: '',
    body: '',
  })

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    post('/blog/posts')
  }

  return (
    <AppLayout>
      <div className="max-w-3xl">
        <div className="mb-6">
          <Link href="/blog/posts" className="text-blue-600 hover:underline text-sm">
            &larr; Back to posts
          </Link>
        </div>

        <h1 className="text-3xl font-bold mb-8">New Post</h1>

        <form onSubmit={handleSubmit} className="bg-white rounded-lg shadow-sm border p-8 space-y-6">
          <div>
            <label htmlFor="title" className="block text-sm font-medium text-gray-700">
              Title
            </label>
            <input
              id="title"
              type="text"
              value={data.title}
              onChange={(e) => setData('title', e.target.value)}
              className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              required
            />
            {errors.title && <p className="mt-1 text-sm text-red-600">{errors.title}</p>}
          </div>

          <div>
            <label htmlFor="body" className="block text-sm font-medium text-gray-700">
              Content
            </label>
            <textarea
              id="body"
              value={data.body}
              onChange={(e) => setData('body', e.target.value)}
              rows={10}
              className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              required
            />
            {errors.body && <p className="mt-1 text-sm text-red-600">{errors.body}</p>}
          </div>

          <div className="flex gap-4">
            <button
              type="submit"
              disabled={processing}
              className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              {processing ? 'Creating...' : 'Create Post'}
            </button>
            <Link href="/blog/posts" className="px-6 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200">
              Cancel
            </Link>
          </div>
        </form>
      </div>
    </AppLayout>
  )
}
