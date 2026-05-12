import { Link } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  body: string
  created_at: string
}

interface Pagination {
  page: number
  per_page: number
  total: number
  total_pages: number
}

interface Props {
  title: string
  posts: Post[]
  pagination: Pagination
}

export default function BlogIndex({ title, posts, pagination }: Props) {
  return (
    <AppLayout>
      <div className="max-w-4xl">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold">{title || 'Blog'}</h1>
          <Link
            href="/blog/posts/create"
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            New Post
          </Link>
        </div>

        {posts.length === 0 ? (
          <p className="text-gray-500">No posts yet.</p>
        ) : (
          <div className="space-y-4">
            {posts.map((post) => (
              <div key={post.id} className="p-6 bg-white rounded-lg shadow-sm border">
                <Link href={`/blog/posts/${post.id}`}>
                  <h2 className="text-xl font-semibold hover:text-blue-600">{post.title}</h2>
                </Link>
                <p className="mt-2 text-gray-600 line-clamp-2">{post.body}</p>
                <p className="mt-2 text-sm text-gray-400">{post.created_at}</p>
              </div>
            ))}
          </div>
        )}

        {pagination.total_pages > 1 && (
          <div className="mt-8 flex justify-center gap-2">
            {Array.from({ length: pagination.total_pages }, (_, i) => i + 1).map((page) => (
              <Link
                key={page}
                href={`/blog?page=${page}`}
                className={`px-3 py-1 rounded ${
                  page === pagination.page
                    ? 'bg-blue-600 text-white'
                    : 'bg-white text-gray-700 border hover:bg-gray-50'
                }`}
              >
                {page}
              </Link>
            ))}
          </div>
        )}
      </div>
    </AppLayout>
  )
}
