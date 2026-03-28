---
name: a5c-ai-docusaurus
description: Deep integration with Docusaurus for documentation site
  development. Configure projects, manage sidebars, versioning, i18n, develop
  plugins, and optimize builds for React-based documentation.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
backlog-id: SK-002
metadata:
  author: babysitter-sdk
  version: 1.0.0
---

# Docusaurus Skill

Deep integration with Docusaurus for documentation site development.

## Capabilities

- Generate Docusaurus project configuration
- Create and manage sidebar structures (sidebars.js)
- Configure versioning and i18n
- Develop custom Docusaurus plugins
- MDX component creation and integration
- Build optimization and debugging
- Algolia DocSearch configuration
- Theme customization

## Usage

Invoke this skill when you need to:
- Set up a new Docusaurus documentation site
- Configure sidebars and navigation
- Implement versioned documentation
- Add internationalization (i18n)
- Create custom plugins or themes

## Inputs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| action | string | Yes | init, configure, sidebar, version, i18n, plugin |
| projectPath | string | Yes | Path to Docusaurus project |
| config | object | No | Configuration options |
| version | string | No | Version tag for versioning |
| locale | string | No | Locale code for i18n |

### Input Example

```json
{
  "action": "configure",
  "projectPath": "./docs-site",
  "config": {
    "title": "My Documentation",
    "tagline": "Developer documentation for My Product",
    "url": "https://docs.example.com",
    "organizationName": "my-org",
    "projectName": "my-project"
  }
}
```

## Project Configuration

### docusaurus.config.js

```javascript
// @ts-check
const { themes } = require('prism-react-renderer');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'My Documentation',
  tagline: 'Developer documentation for My Product',
  favicon: 'img/favicon.ico',

  url: 'https://docs.example.com',
  baseUrl: '/',

  organizationName: 'my-org',
  projectName: 'my-project',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es', 'ja'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/my-org/my-project/edit/main/',
          showLastUpdateTime: true,
          showLastUpdateAuthor: true,
          versions: {
            current: {
              label: 'Next',
              path: 'next',
            },
          },
        },
        blog: {
          showReadingTime: true,
          editUrl: 'https://github.com/my-org/my-project/edit/main/',
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
      image: 'img/social-card.jpg',
      navbar: {
        title: 'My Project',
        logo: {
          alt: 'My Project Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'tutorialSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            type: 'docsVersionDropdown',
            position: 'right',
          },
          {
            type: 'localeDropdown',
            position: 'right',
          },
          {
            href: 'https://github.com/my-org/my-project',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              { label: 'Getting Started', to: '/docs/intro' },
              { label: 'API Reference', to: '/docs/api' },
            ],
          },
          {
            title: 'Community',
            items: [
              { label: 'Discord', href: 'https://discord.gg/example' },
              { label: 'Twitter', href: 'https://twitter.com/example' },
            ],
          },
        ],
        copyright: `Copyright ${new Date().getFullYear()} My Project.`,
      },
      prism: {
        theme: themes.github,
        darkTheme: themes.dracula,
        additionalLanguages: ['bash', 'json', 'yaml'],
      },
      algolia: {
        appId: 'YOUR_APP_ID',
        apiKey: 'YOUR_SEARCH_API_KEY',
        indexName: 'my-project',
        contextualSearch: true,
      },
    }),
};

module.exports = config;
```

## Sidebar Configuration

### sidebars.js

```javascript
/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/configuration',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/authentication',
        'guides/api-usage',
        {
          type: 'category',
          label: 'Advanced',
          items: [
            'guides/advanced/caching',
            'guides/advanced/performance',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      link: {
        type: 'generated-index',
        title: 'API Reference',
        description: 'Complete API documentation',
      },
      items: [
        'api/client',
        'api/authentication',
        'api/resources',
      ],
    },
    {
      type: 'link',
      label: 'GitHub',
      href: 'https://github.com/my-org/my-project',
    },
  ],
};

module.exports = sidebars;
```

## Custom Components

### Tabs Component

```jsx
// src/components/CodeTabs.jsx
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import CodeBlock from '@theme/CodeBlock';

export function CodeTabs({ children, labels = ['JavaScript', 'Python', 'cURL'] }) {
  return (
    <Tabs groupId="code-examples">
      {labels.map((label, index) => (
        <TabItem key={label} value={label.toLowerCase()} label={label}>
          <CodeBlock language={label.toLowerCase()}>
            {children[index]}
          </CodeBlock>
        </TabItem>
      ))}
    </Tabs>
  );
}
```

### API Endpoint Component

```jsx
// src/components/ApiEndpoint.jsx
import React from 'react';
import styles from './ApiEndpoint.module.css';

export function ApiEndpoint({ method, path, description }) {
  const methodColors = {
    GET: '#61affe',
    POST: '#49cc90',
    PUT: '#fca130',
    DELETE: '#f93e3e',
    PATCH: '#50e3c2',
  };

  return (
    <div className={styles.endpoint}>
      <span
        className={styles.method}
        style={{ backgroundColor: methodColors[method] }}
      >
        {method}
      </span>
      <code className={styles.path}>{path}</code>
      <p className={styles.description}>{description}</p>
    </div>
  );
}
```

## Versioning

### Creating a Version

```bash
# Create version snapshot
npm run docusaurus docs:version 1.0.0

# Project structure after versioning
docs/
├── intro.md              # Current (next) version
├── getting-started/
versioned_docs/
├── version-1.0.0/
│   ├── intro.md
│   └── getting-started/
versioned_sidebars/
├── version-1.0.0-sidebars.json
versions.json
```

### versions.json

```json
[
  "2.0.0",
  "1.1.0",
  "1.0.0"
]
```

## Internationalization (i18n)

### Translation Structure

```
i18n/
├── en/
│   └── docusaurus-plugin-content-docs/
│       └── current/
│           └── intro.md
├── es/
│   └── docusaurus-plugin-content-docs/
│       └── current/
│           └── intro.md
└── ja/
    └── docusaurus-plugin-content-docs/
        └── current/
            └── intro.md
```

### Write Translations Command

```bash
# Generate translation files
npm run write-translations -- --locale es

# Start dev server for locale
npm run start -- --locale es
```

## Custom Plugin

### Plugin Template

```javascript
// plugins/my-plugin/index.js
module.exports = function myPlugin(context, options) {
  return {
    name: 'my-plugin',

    async loadContent() {
      // Load custom content
      return { /* content */ };
    },

    async contentLoaded({ content, actions }) {
      const { addRoute, createData } = actions;

      // Create custom routes
      addRoute({
        path: '/my-custom-page',
        component: '@site/src/pages/MyPage.jsx',
        exact: true,
      });
    },

    configureWebpack(config, isServer, utils) {
      // Modify webpack config
      return {
        resolve: {
          alias: {
            '@custom': path.resolve(__dirname, 'src'),
          },
        },
      };
    },
  };
};
```

## Algolia DocSearch

### algolia.config.json

```json
{
  "index_name": "my-project",
  "start_urls": [
    "https://docs.example.com/"
  ],
  "sitemap_urls": [
    "https://docs.example.com/sitemap.xml"
  ],
  "selectors": {
    "lvl0": ".menu__link--active",
    "lvl1": "article h1",
    "lvl2": "article h2",
    "lvl3": "article h3",
    "lvl4": "article h4",
    "content": "article p, article li"
  }
}
```

## Workflow

1. **Initialize project** - Create new Docusaurus site
2. **Configure** - Set up docusaurus.config.js
3. **Structure content** - Organize docs and sidebars
4. **Add components** - Create custom MDX components
5. **Configure search** - Set up Algolia DocSearch
6. **Add versioning** - Create version snapshots
7. **Deploy** - Build and deploy to hosting

## Dependencies

```json
{
  "dependencies": {
    "@docusaurus/core": "^3.0.0",
    "@docusaurus/preset-classic": "^3.0.0",
    "@mdx-js/react": "^3.0.0",
    "prism-react-renderer": "^2.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  }
}
```

## CLI Commands

```bash
# Create new project
npx create-docusaurus@latest my-docs classic

# Start development server
npm run start

# Build for production
npm run build

# Create version
npm run docusaurus docs:version 1.0.0

# Generate translations
npm run write-translations -- --locale es

# Deploy to GitHub Pages
npm run deploy
```

## Best Practices Applied

- Use MDX for interactive components
- Implement versioning for stable releases
- Configure search for discoverability
- Add edit links for community contributions
- Use admonitions for callouts
- Optimize images with ideal-image
- Enable last updated timestamps

## References

- Docusaurus: https://docusaurus.io/
- MDX: https://mdxjs.com/
- Algolia DocSearch: https://docsearch.algolia.com/
- Prism: https://prismjs.com/

## Target Processes

- docs-as-code-pipeline.js
- docs-versioning.js
- interactive-tutorials.js
- knowledge-base-setup.js
