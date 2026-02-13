import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: ['getting-started/installation', 'getting-started/quick-start', 'getting-started/examples'],
    },
    {
      type: 'category',
      label: 'Language Guide',
      items: ['language/basics', 'language/types', 'language/functions', 'language/control-flow', 'language/structs'],
    },
    {
      type: 'category',
      label: 'Architecture',
      items: ['architecture/overview', 'architecture/lexer', 'architecture/parser', 'architecture/sema', 'architecture/optimizer', 'architecture/codegen', 'architecture/runtime'],
    },
    {
      type: 'category',
      label: 'Tools',
      items: ['tools/faxt-cli', 'tools/vscode'],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: ['api/standard-library'],
    },
  ],
};

export default sidebars;
