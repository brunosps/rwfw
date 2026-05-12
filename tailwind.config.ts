import type { Config } from 'tailwindcss'

export default {
  content: [
    'crates/rwfw-app/web/**/*.tsx',
    'crates/modules/*/web/**/*.tsx',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
} satisfies Config
