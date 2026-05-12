import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const ssrPolyfills = `
(function () {
  const global = globalThis;

  if (typeof global.console === 'undefined') {
    global.console = { log() {}, info() {}, warn() {}, error() {}, debug() {} };
  }

  if (typeof global.performance === 'undefined') {
    global.performance = { now: () => Date.now() };
  }

  if (typeof global.queueMicrotask === 'undefined') {
    global.queueMicrotask = (callback) => Promise.resolve().then(callback);
  }

  if (typeof global.setTimeout === 'undefined') {
    global.setTimeout = (callback) => {
      global.queueMicrotask(callback);
      return 0;
    };
  }

  if (typeof global.clearTimeout === 'undefined') {
    global.clearTimeout = () => {};
  }

  if (typeof global.FormData === 'undefined') {
    global.FormData = class FormData {
      constructor() {
        this._entries = [];
      }

      append(name, value) {
        this._entries.push([name, value]);
      }

      forEach(callback) {
        this._entries.forEach(([name, value]) => callback(value, name, this));
      }
    };
  }

  if (typeof global.URLSearchParams === 'undefined') {
    global.URLSearchParams = class URLSearchParams {
      constructor(init = '') {
        this._params = [];
        const query = String(init).replace(/^\\?/, '');

        if (query) {
          query.split('&').forEach((part) => {
            if (!part) return;
            const [name, value = ''] = part.split('=');
            this.append(decodeURIComponent(name), decodeURIComponent(value));
          });
        }
      }

      append(name, value) {
        this._params.push([String(name), String(value)]);
      }

      set(name, value) {
        this.delete(name);
        this.append(name, value);
      }

      delete(name) {
        this._params = this._params.filter(([key]) => key !== String(name));
      }

      toString() {
        return this._params
          .map(([name, value]) => encodeURIComponent(name) + '=' + encodeURIComponent(value))
          .join('&');
      }
    };
  }

  if (typeof global.URL === 'undefined') {
    global.URL = class URL {
      constructor(input) {
        const raw = String(input || '/');
        const hashIndex = raw.indexOf('#');
        const withoutHash = hashIndex >= 0 ? raw.slice(0, hashIndex) : raw;

        this.hash = hashIndex >= 0 ? raw.slice(hashIndex) : '';

        const queryIndex = withoutHash.indexOf('?');
        this.pathname = queryIndex >= 0 ? withoutHash.slice(0, queryIndex) || '/' : withoutHash || '/';
        this.searchParams = new global.URLSearchParams(queryIndex >= 0 ? withoutHash.slice(queryIndex) : '');
      }

      get search() {
        const query = this.searchParams.toString();
        return query ? '?' + query : '';
      }

      set search(value) {
        this.searchParams = new global.URLSearchParams(value);
      }

      get href() {
        return this.pathname + this.search + this.hash;
      }

      toString() {
        return this.href;
      }
    };
  }

  if (typeof global.MessageChannel === 'undefined') {
    global.MessageChannel = class MessageChannel {
      constructor() {
        const port1 = { onmessage: null };
        const port2 = {
          postMessage(data) {
            global.queueMicrotask(() => {
              if (typeof port1.onmessage === 'function') {
                port1.onmessage({ data });
              }
            });
          },
        };

        this.port1 = port1;
        this.port2 = port2;
      }
    };
  }

  if (typeof global.TextEncoder === 'undefined') {
    global.TextEncoder = class TextEncoder {
      encode(input = '') {
        const text = String(input);
        const bytes = [];

        for (let index = 0; index < text.length; index += 1) {
          let codePoint = text.charCodeAt(index);

          if (codePoint >= 0xd800 && codePoint <= 0xdbff && index + 1 < text.length) {
            const next = text.charCodeAt(index + 1);
            if (next >= 0xdc00 && next <= 0xdfff) {
              codePoint = 0x10000 + ((codePoint - 0xd800) << 10) + (next - 0xdc00);
              index += 1;
            }
          }

          if (codePoint <= 0x7f) {
            bytes.push(codePoint);
          } else if (codePoint <= 0x7ff) {
            bytes.push(0xc0 | (codePoint >> 6), 0x80 | (codePoint & 0x3f));
          } else if (codePoint <= 0xffff) {
            bytes.push(0xe0 | (codePoint >> 12), 0x80 | ((codePoint >> 6) & 0x3f), 0x80 | (codePoint & 0x3f));
          } else {
            bytes.push(
              0xf0 | (codePoint >> 18),
              0x80 | ((codePoint >> 12) & 0x3f),
              0x80 | ((codePoint >> 6) & 0x3f),
              0x80 | (codePoint & 0x3f),
            );
          }
        }

        return new Uint8Array(bytes);
      }
    };
  }
})();
`;

export default defineConfig(({ mode }) => {
  const isSsrBuild = mode === 'ssr'
  const devServerPort = Number(process.env.RWFW_VITE_PORT ?? '5173')

  return {
  plugins: [react()],
  root: '.',
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
  build: {
    outDir: isSsrBuild ? 'dist/server' : 'dist/client',
    manifest: !isSsrBuild,
    lib: isSsrBuild
      ? {
          entry: 'crates/rwfw-app/web/ssr.tsx',
          formats: ['iife'],
          name: 'RWFWSSR',
          fileName: () => 'ssr.js',
        }
      : undefined,
    rollupOptions: isSsrBuild
      ? {
          output: {
            banner: ssrPolyfills,
          },
        }
      : {
          input: 'crates/rwfw-app/web/app.tsx',
        },
  },
    server: {
      port: devServerPort,
      strictPort: true,
    },
  resolve: {
    alias: {
      '@app': '/crates/rwfw-app/web',
      '@modules': '/crates/modules',
    },
  },
  }
})
