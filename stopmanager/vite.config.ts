import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  // `npm run dev` talks to a synth running locally (the built app is served by the synth itself).
  server: { proxy: { '/api': 'http://localhost:8080' } },
})
