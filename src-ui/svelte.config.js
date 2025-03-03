import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  // Enable Svelte 5 runes
  compilerOptions: {
    runes: true
  },
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  preprocess: vitePreprocess()
}; 