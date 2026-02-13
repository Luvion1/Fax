import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Fax Programming Language',
  tagline: 'A high-performance polyglot programming language with generational GC',
  url: 'https://luvion1.github.io',
  baseUrl: '/Fax/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',

  organizationName: 'Luvion1',
  projectName: 'Fax',

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/Luvion1/Fax/tree/main/',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/fax-social-card.jpg',
      announcementBar: {
        id: 'announcement',
        content: 'Welcome to Fax v0.0.1! A high-performance polyglot compiler.',
        backgroundColor: '#6366f1',
        textColor: '#ffffff',
        isCloseable: true,
      },
      navbar: {
        title: 'Fax',
        logo: {
          alt: 'Fax Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/Luvion1/Fax',
            label: 'GitHub',
            position: 'right',
          },
          {
            href: 'https://github.com/Luvion1/Fax/releases',
            label: 'Releases',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        copyright: `Copyright © ${new Date().getFullYear()} Fax Programming Language. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ['rust', 'zig', 'haskell', 'cpp', 'bash'],
      },
    }),
};

export default config;
