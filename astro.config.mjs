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
            { label: 'Introduction', link: '/getting-started/introduction/' },
            { label: 'Installation', link: '/getting-started/installation/' },
            { label: 'Quick Start', link: '/getting-started/quick-start/' },
          ],
        },
        {
          label: 'Language',
          items: [
            { label: 'Basics', link: '/language/basics/' },
            { label: 'Types', link: '/language/types/' },
            { label: 'Functions', link: '/language/functions/' },
            { label: 'Structs', link: '/language/structs/' },
            { label: 'Control Flow', link: '/language/control-flow/' },
            { label: 'Pattern Matching', link: '/language/pattern-matching/' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Architecture', link: '/reference/architecture/' },
            { label: 'Compiler Pipeline', link: '/reference/pipeline/' },
            { label: 'FGC', link: '/reference/fgc/' },
            { label: 'Toolchain', link: '/reference/toolchain/' },
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
