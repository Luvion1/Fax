import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://luvion1.github.io',
  base: '/Fax',
  integrations: [
    starlight({
      title: 'Fax',
      tagline: 'A high-performance polyglot programming language',
      
      logo: {
        src: './src/assets/logo.svg',
      },

      social: {
        github: 'https://github.com/Luvion1/Fax',
      },

      editLink: {
        baseUrl: 'https://github.com/Luvion1/Fax/edit/main/',
      },

      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'getting-started/introduction' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Quick Start', slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'Language',
          items: [
            { label: 'Basics', slug: 'language/basics' },
            { label: 'Types', slug: 'language/types' },
            { label: 'Functions', slug: 'language/functions' },
            { label: 'Structs', slug: 'language/structs' },
            { label: 'Control Flow', slug: 'language/control-flow' },
            { label: 'Pattern Matching', slug: 'language/pattern-matching' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Architecture', slug: 'reference/architecture' },
            { label: 'Compiler Pipeline', slug: 'reference/pipeline' },
            { label: 'FGC', slug: 'reference/fgc' },
            { label: 'Toolchain', slug: 'reference/toolchain' },
          ],
        },
      ],

      expressiveCode: {
        themes: ['github-dark', 'github-light'],
      },

      customCss: [
        './src/styles/custom.css',
      ],
    }),
  ],
});
