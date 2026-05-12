import { usePage } from '@inertiajs/react'

export default function FlashMessages() {
  const { props } = usePage()
  const flash = (props as any).flash

  if (!flash) return null

  return (
    <div className="space-y-2 mb-4">
      {flash.success && (
        <div className="p-4 bg-green-100 text-green-700 rounded-md text-sm">
          {flash.success}
        </div>
      )}
      {flash.error && (
        <div className="p-4 bg-red-100 text-red-700 rounded-md text-sm">
          {flash.error}
        </div>
      )}
      {flash.info && (
        <div className="p-4 bg-blue-100 text-blue-700 rounded-md text-sm">
          {flash.info}
        </div>
      )}
    </div>
  )
}
