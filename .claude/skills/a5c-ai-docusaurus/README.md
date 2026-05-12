# Docusaurus Skill

Deep integration with Docusaurus for documentation site development.

## Overview

This skill provides comprehensive support for building documentation sites with Docusaurus. It covers project configuration, sidebar management, versioning, internationalization, custom plugins, and theme customization.

## When to Use

- Setting up a new Docusaurus documentation site
- Configuring sidebars and navigation
- Implementing versioned documentation
- Adding internationalization (i18n)
- Creating custom MDX components and plugins

## Quick Start

### Initialize Project

```bash
npx create-docusaurus@latest my-docs classic
```

### Configure Site

```json
{
  "action": "configure",
  "projectPath": "./my-docs",
  "config": {
    "title": "My Documentation",
    "url": "https://docs.example.com"
  }
}
```

### Create Version

```json
{
  "action": "version",
  "projectPath": "./my-docs",
  "version": "1.0.0"
}
```

## Key Features

### 1. Project Configuration
- docusaurus.config.js setup
- Preset configuration
- Theme customization

### 2. Sidebar Management
- Auto-generated sidebars
- Category nesting
- External links

### 3. Versioning
- Version snapshots
- Version dropdown
- Migration support

### 4. Internationalization
- Multi-locale support
- Translation workflows
- RTL support

### 5. Search Integration
- Algolia DocSearch
- Local search fallback

## Configuration Example

```javascript
// docusaurus.config.js
module.exports = {
  title: 'My Docs',
  url: 'https://docs.example.com',
  presets: [
    ['classic', {
      docs: {
        sidebarPath: './sidebars.js',
        editUrl: 'https://github.com/org/repo/edit/main/',
      },
    }],
  ],
};
```

## Sidebar Example

```javascript
// sidebars.js
module.exports = {
  docs: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: ['installation', 'quick-start'],
    },
  ],
};
```

## CLI Commands

```bash
# Development
npm run start

# Build
npm run build

# Create version
npm run docusaurus docs:version 1.0.0

# Generate translations
npm run write-translations -- --locale es
```

## Process Integration

| Process | Usage |
|---------|-------|
| `docs-as-code-pipeline.js` | CI/CD setup |
| `docs-versioning.js` | Version management |
| `interactive-tutorials.js` | MDX tutorials |
| `knowledge-base-setup.js` | Site structure |

## Dependencies

- @docusaurus/core
- @docusaurus/preset-classic
- @mdx-js/react
- react, react-dom

## References

- [Docusaurus](https://docusaurus.io/)
- [Docusaurus Configuration](https://docusaurus.io/docs/configuration)
- [Algolia DocSearch](https://docsearch.algolia.com/)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-24 | Initial release |

---

**Backlog ID:** SK-002
**Category:** Documentation Site Generators
**Status:** Active
