import { Link, router } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  body: string
  author_name?: string
  created_at: string
}

interface Props {
  post: Post
}

export default function PostShow({ post }: Props) {
  function handleDelete() {
    if (confirm('Are you sure you want to delete this post?')) {
      router.delete(`/blog/posts/${post.id}`)
    }
  }

  return (
    <AppLayout>
      <div className="max-w-3xl">
        <div className="mb-6">
          <Link href="/blog/posts" className="text-blue-600 hover:underline text-sm">
            &larr; Back to posts
          </Link>
        </div>

        <article className="bg-white rounded-lg shadow-sm border p-8">
          <h1 className="text-3xl font-bold mb-4">{post.title}</h1>
          <div className="flex items-center gap-4 text-sm text-gray-400 mb-8">
            {post.author_name && <span>by {post.author_name}</span>}
            <span>{post.created_at}</span>
          </div>
          <div className="prose max-w-none">
            <p>{post.body}</p>
          </div>
        </article>

        <div className="mt-6 flex gap-4">
          <Link
            href={`/blog/posts/${post.id}/edit`}
            className="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200"
          >
            Edit
          </Link>
          <button
            onClick={handleDelete}
            className="px-4 py-2 bg-red-100 text-red-700 rounded-md hover:bg-red-200"
          >
            Delete
          </button>
        </div>
      </div>
    </AppLayout>
  )
}
