import { defineConfig } from 'astro/config';
import sitemap from "@astrojs/sitemap";
// import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  outDir: '../src/assets',
  site: 'http://192.168.2.1',
  integrations: [
    sitemap(),
    // tailwind()

  ] 
});
