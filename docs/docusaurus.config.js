// @ts-nocheck
// Note: type annotations allow type checking and IDEs autocompletion

// const progress = require("./scripts/progress_translate_lang.json");
import { themes as prismThemes } from "prism-react-renderer";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Farm",
  tagline: "Super fast web build tool written in Rust",
  favicon: "img/favicon.ico",

  // Set the production url of your site here
  url: "https://farmfe.org",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: "/",
  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "farm-fe", // Usually your GitHub org/user name.
  projectName: "farm-fe.github.io", // Usually your repo name.
  deploymentBranch: "gh-pages",
  onBrokenLinks: "ignore",
  // onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en", "zh"],
    localeConfigs: {
      en: {
        label: "English",
        direction: "ltr",
      },
      zh: {
        label: `简体中文`,
        direction: "ltr",
      },
    },
  },

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve("./sidebars.js"),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: "https://github.com/farm-fe/farm-fe.github.io/tree/main/",
          lastVersion: "current",
          versions: {
            current: {
              label: "2.0.0",
            },
            // "0.x": {
            //   label: "0.15",
            // },
          },
        },
        theme: {
          customCss: require.resolve("./src/css/custom.scss"),
        },
      }),
    ],
  ],
  // themes: ["@docusaurus/theme-search-algolia"],
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: "img/farm-social-card.png",
      navbar: {
        title: "Farm",
        logo: {
          alt: "My Site Logo",
          src: "img/logo.png",
        },
        items: [
          {
            type: "doc",
            docId: "quick-start",
            position: "left",
            label: "Guides",
          },
          {
            type: "doc",
            docId: "config/configuring-farm",
            position: "left",
            label: "Config",
          },
          {
            type: "doc",
            docId: "api/rust-plugin-api",
            position: "left",
            label: "API",
          },
          {
            type: "doc",
            docId: "plugins/official-plugins/overview",
            position: "left",
            label: "Plugins",
          },
          {
            position: "left",
            label: "Community",
            items: [
              {
                label: "Team",
                to: "/team",
              },
              {
                label: "Blog",
                to: "/blog/index",
              },
              {
                label: "Awesome Farm",
                href: "https://github.com/farm-fe/awesome-farm",
              },
              {
                label: "Discord",
                href: "https://discord.com/invite/mDErq9aFnF",
              },
              {
                label: "Twitter",
                href: "https://x.com/brightwwu46799",
              },
              {
                label: "WeChat Group",
                href: "https://github.com/farm-fe/farm?tab=readme-ov-file#chat-with-us",
              },
            ],
          },
          {
            to: '/roadmap',
            position: "left",
            label: "Roadmap",
          },
          {
            type: "docsVersionDropdown",
            position: "right",
          },
          {
            type: "localeDropdown",
            position: "right",
          },

          {
            href: "https://github.com/farm-fe/farm",
            position: "right",
            className: "header-github-link",
            "aria-label": "GitHub repository",
          },
        ],
      },
      docs: {
        versionPersistence: "localStorage",
      },

      // announcementBar: {
      //   id: "announcementBar-2", // Increment on change
      //   content: `🎉 Farm is now in 1.0.0 stable and production ready. If you like Farm, give it a ⭐️ on <a target="_blank" rel="noopener noreferrer" href="https://github.com/farm-fe/farm">GitHub</a>`,
      // },
      customFields: {
        customFooter: "src/theme/Footer/index.tsx",
      },
      prism: {
        additionalLanguages: ["powershell"],
        theme: prismThemes.nightOwlLight,
        darkTheme: prismThemes.oneDark,
        magicComments: [
          // Remember to extend the default highlight class name as well!
          {
            className: "theme-code-block-highlighted-line",
            line: "highlight-next-line",
            block: { start: "highlight-start", end: "highlight-end" },
          },
          {
            className: "code-block-highlight-line",
            line: "c-highlight-next-line",
            block: { start: "c-highlight-start", end: "c-highlight-end" },
          },
        ],
      },
      algolia: {
        appId: "G3J92PUFY2",
        apiKey: "2b0f3f1f06f381249d44682a21206f4f",
        indexName: "farm-feio",
      },
      colorMode: {
        defaultMode: "light",
      },
    }),
  plugins: [
    "docusaurus-plugin-sass",
    async function TailwindCSSPlugin(context, options) {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          // Appends TailwindCSS and AutoPrefixer.
          postcssOptions.plugins.push(require("tailwindcss"));
          postcssOptions.plugins.push(require("autoprefixer"));
          return postcssOptions;
        },
      };
    },
  ],
};

module.exports = config;
