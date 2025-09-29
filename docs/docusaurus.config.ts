import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: "OctoType",
  tagline: "A typing trainer for your terminal",
  favicon: "img/favicon.ico",

  future: {
    v4: true,
  },

  url: "https://mahlquistj.github.io",
  baseUrl: "/octotype/",
  organizationName: "mahlquistj",
  projectName: "octotype",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  markdown: {
    mermaid: true,
  },

  themes: ["@docusaurus/theme-mermaid"],

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
        },
        blog: {
          showReadingTime: true,
          feedOptions: {
            type: ["rss", "atom"],
            xslt: true,
          },
          // Useful options to enforce blogging best practices
          onInlineTags: "warn",
          onInlineAuthors: "warn",
          onUntruncatedBlogPosts: "warn",
        },
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // TODO: Replace with social card
    image: "img/docusaurus-social-card.jpg",
    navbar: {
      title: "OctoType",
      logo: {
        alt: "My Site Logo",
        src: "/img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "configuration",
          position: "left",
          label: "Configuration",
        },
        {
          type: "docSidebar",
          sidebarId: "contributing",
          position: "left",
          label: "Contributing",
        },
        {
          href: "https://github.com/mahlquistj/octotype",
          label: "GitHub",
          position: "right",
        },
        {
          href: "https://discord.gg/zk4SXvdUxj",
          label: "Discord",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Getting started",
              to: "/docs/intro",
            },
          ],
        },
        {
          title: "Links",
          items: [
            {
              label: "Join the Discord",
              href: "https://discord.gg/zk4SXvdUxj",
            },
          ],
        },
      ],
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["toml", "nix", "bash"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
