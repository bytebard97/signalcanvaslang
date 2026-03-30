import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'SignalCanvasDiagram',
      fileName: 'diagram',
    },
    rollupOptions: {
      external: ['vue', '@vue-flow/core', 'elkjs', 'lucide-vue-next'],
      output: {
        globals: {
          vue: 'Vue',
          '@vue-flow/core': 'VueFlow',
          elkjs: 'ELK',
          'lucide-vue-next': 'LucideVueNext',
        },
      },
    },
  },
})
