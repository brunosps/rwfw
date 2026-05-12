import { Link, usePage } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  body: string
  author_name?: string
  created_at: string
}

interface Pagination {
  page: number
  per_page: number
  total: number
  total_pages: number
}

interface Props {
  posts: Post[]
  pagination: Pagination
}

interface SharedData {
  auth?: { user?: { name: string; email: string } }
}

export default function PostsIndex({ posts, pagination }: Props) {
  const { props } = usePage<SharedData & Record<string, unknown>>()
  const auth = props.auth

  return (
    <AppLayout>
      <div className="max-w-4xl">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold">Posts</h1>
          {auth?.user && (
            <Link
              href="/blog/posts/create"
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
            >
              New Post
            </Link>
          )}
        </div>

        {!auth?.user && (
          <div className="mb-6 rounded-lg border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900">
            Posts are public.{' '}
            <Link href="/auth/login" className="font-semibold underline">
              Sign in
            </Link>{' '}
            or{' '}
            <Link href="/auth/register" className="font-semibold underline">
              create an account
            </Link>{' '}
            to create new posts.
          </div>
        )}

        {posts.length === 0 ? (
          <p className="text-gray-500">No posts yet. Sign in to create the first post.</p>
        ) : (
          <div className="space-y-4">
            {posts.map((post) => (
              <article key={post.id} className="p-6 bg-white rounded-lg shadow-sm border">
                <Link href={`/blog/posts/${post.id}`}>
                  <h2 className="text-xl font-semibold hover:text-blue-600 transition-colors">
                    {post.title}
                  </h2>
                </Link>
                <p className="mt-2 text-gray-600 line-clamp-2">{post.body}</p>
                <div className="mt-3 flex items-center gap-4 text-sm text-gray-400">
                  {post.author_name && <span>by {post.author_name}</span>}
                  <span>{post.created_at}</span>
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
    </AppLayout>
  )
}
