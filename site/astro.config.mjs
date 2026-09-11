import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mdx from '@astrojs/mdx';
import { unified } from '@astrojs/markdown-remark';
import { remarkPlantuml } from './src/plugins/remark-plantuml.ts';

export default defineConfig({
  site: 'https://vgerbot.github.io',
  base: '/plantuml.rs/',
  integrations: [
    starlight({
      title: 'plantuml.rs',
      logo: { src: './src/assets/logo.svg', replacesTitle: false },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/vgerbot/plantuml.rs' },
      ],
      customCss: ['./src/styles/custom.css'],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { slug: 'getting-started', label: 'Overview' },
            'getting-started/installation',
            'getting-started/quick-start',
          ],
        },
        {
          label: 'Guides',
          items: [
            'guides/architecture',
            'guides/rust-api',
            'guides/typescript-binding',
            'guides/java-binding',
            'guides/wasm-build',
          ],
        },
        {
          label: 'Language Reference',
          items: [
            { slug: 'language', label: 'Overview' },
            'language/sequence-diagram',
            'language/class-diagram',
            'language/activity-diagram',
            'language/use-case-diagram',
            'language/component-diagram',
            'language/state-diagram',
            'language/object-diagram',
            'language/deployment-diagram',
            'language/timing-diagram',
            'language/mindmap',
            'language/gantt',
            'language/wbs',
            'language/json',
            'language/yaml',
          ],
        },
        {
          label: 'API Reference',
          items: [
            'reference/render-svg',
            'reference/render-preproc',
            'reference/render',
            'reference/file-formats',
          ],
        },
        { label: 'Playground', link: '/playground/' },
      ],
      components: {
        // No overrides needed initially
      },
    }),
    mdx(),
  ],
  markdown: {
    processor: unified({ remarkPlugins: [remarkPlantuml] }),
  },
});
