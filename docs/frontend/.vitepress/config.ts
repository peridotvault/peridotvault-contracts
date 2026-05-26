import { cpSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitepress";

const vitepressDir = dirname(fileURLToPath(import.meta.url));
const docsRoot = join(vitepressDir, "..");
const typedocApiDir = join(docsRoot, "sdk", "api");
const builtApiDir = join(docsRoot, ".vitepress", "dist", "sdk", "api");

export default defineConfig({
  title: "PeridotVault Contracts",
  description:
    "Official contract interface SDK and Solana program documentation for PeridotVault",

  vite: {
    ssr: {
      noExternal: ["@solana/web3.js", "@coral-xyz/anchor"],
    },
    optimizeDeps: {
      include: ["@solana/web3.js", "@coral-xyz/anchor"],
    },
  },

  ignoreDeadLinks: [/^\/sdk\/api\//],
  lastUpdated: true,
  cleanUrls: true,

  head: [
    ["link", { rel: "icon", href: "/favicon.ico" }],
  ],

  buildEnd() {
    if (existsSync(typedocApiDir)) {
      cpSync(typedocApiDir, builtApiDir, { recursive: true, force: true });
    }
  },

  themeConfig: {
    logo: { text: "PeridotVault" },

    nav: [
      { text: "Guide", link: "/guide/introduction" },
      { text: "Solana", link: "/solana/overview" },
      { text: "SDK", link: "/sdk/" },
      { text: "Audit", link: "/audit/" },
    ],

    sidebar: [
      {
        text: "Guide",
        items: [
          { text: "Introduction", link: "/guide/introduction" },
          { text: "Installation", link: "/guide/installation" },
          { text: "Environments", link: "/guide/environments" },
          { text: "Deployments", link: "/guide/deployments" },
        ],
      },
      {
        text: "Solana",
        items: [
          { text: "Overview", link: "/solana/overview" },
          { text: "PGL-1", link: "/solana/pgl1" },
          { text: "Registry", link: "/solana/registry" },
          { text: "Registry Playground", link: "/solana/registry-playground" },
          { text: "Game Store", link: "/solana/game-store" },
          { text: "PDA Reference", link: "/solana/pda" },
          { text: "Instruction Builders", link: "/solana/instructions" },
          { text: "Transaction Builders", link: "/solana/transactions" },
          { text: "Account Fetchers", link: "/solana/account-fetchers" },
        ],
      },
      {
        text: "SDK",
        items: [
          { text: "Overview", link: "/sdk/" },
          { text: "API Reference", link: "/sdk/api-reference" },
        ],
      },
      {
        text: "Audit",
        items: [
          { text: "Methodology", link: "/audit/" },
          { text: "Findings Summary", link: "/audit/findings-summary" },
          { text: "PGL-1 Audit", link: "/audit/pgl1-audit" },
          { text: "Registry Audit", link: "/audit/registry-audit" },
          { text: "Game Store Audit", link: "/audit/game-store-audit" },
          {
            text: "Cross-Program CPI Audit",
            link: "/audit/cross-program-cpi-audit",
          },
          {
            text: "Economic Security Audit",
            link: "/audit/economic-security-audit",
          },
          { text: "Test Coverage Audit", link: "/audit/test-coverage-audit" },
          { text: "Remediation Plan", link: "/audit/remediation-plan" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/peridotvault" },
    ],

    search: {
      provider: "local",
    },

    editLink: {
      pattern: "https://github.com/peridotvault/peridotvault-contracts/edit/main/docs/frontend/:path",
      text: "Edit this page on GitHub",
    },

    lastUpdated: {
      text: "Last updated",
      formatOptions: {
        forceLocale: true,
        dateStyle: "medium",
      },
    },

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2025 PeridotVault",
    },

    outline: {
      level: [2, 3],
      label: "On this page",
    },

    docFooter: {
      prev: "Previous page",
      next: "Next page",
    },
  },
});
