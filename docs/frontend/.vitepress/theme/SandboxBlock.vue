<script setup lang="ts">
import { computed } from "vue";
import {
  SandpackProvider,
  SandpackLayout,
  SandpackCodeEditor,
  SandpackPreview,
} from "sandpack-vue3";

import pkgJson from "./sandpack-shims/package.json?raw";
import indexJs from "./sandpack-shims/index.js?raw";
import solanaJs from "./sandpack-shims/solana.js?raw";
import deploymentsJs from "./sandpack-shims/deployments.js?raw";

type Template = "install" | "deployments" | "pda" | "solana-programs";

const props = withDefaults(
  defineProps<{
    template?: Template;
    files?: Record<string, { code: string; active?: boolean }>;
    height?: number;
    showPreview?: boolean;
    showEditor?: boolean;
  }>(),
  {
    template: undefined,
    files: undefined,
    height: 400,
    showPreview: true,
    showEditor: true,
  }
);

const sandpackNodeModules = {
  "/node_modules/@peridotvault/contracts/package.json": { code: pkgJson },
  "/node_modules/@peridotvault/contracts/dist/index.js": { code: indexJs },
  "/node_modules/@peridotvault/contracts/dist/solana/index.js": { code: solanaJs },
  "/node_modules/@peridotvault/contracts/dist/deployments/index.js": { code: deploymentsJs },
};

const customSetup = {
  dependencies: {
    "@solana/web3.js": "^1.98.0",
  },
};

const installCode = `\
// Try importing the real PeridotVault SDK
import { deployments, evm, solana } from "@peridotvault/contracts";

console.log("Available namespaces:", { deployments, evm, solana });
console.log("Testnet programs:", solana.getSolanaPrograms("testnet"));
`;

const deploymentsCode = `\
// Look up program IDs by chain + environment
import { getDeployment } from "@peridotvault/contracts/deployments";

// Try "solana" | "ethereum" | "base"
const chain = "solana";

// Try "local" | "staging" | "testnet" | "mainnet"
const env = "testnet";

const dep = getDeployment({ chain, environment: env });
console.log(chain + " (" + env + "):", JSON.stringify(dep, null, 2));

// Try unresolved mainnet:
try {
  getDeployment({ chain: "ethereum", environment: "mainnet" });
} catch (err) {
  console.error("Expected error:", err.message);
}
`;

const pdaCode = `\
// PDA helpers — deterministic address derivation
import { PublicKey } from "@solana/web3.js";
import { findLicensePda } from "@peridotvault/contracts/solana";

const holder = new PublicKey("11111111111111111111111111111111");
const game = new PublicKey("22222222222222222222222222222222");

const [licensePda, bump] = findLicensePda(holder, game);

console.log("Holder:", holder.toBase58());
console.log("Game:", game.toBase58());
console.log("License PDA:", licensePda.toBase58());
console.log("Bump seed:", bump);
`;

const solanaProgramsCode = `\
// Solana program ID lookup
import { getSolanaPrograms } from "@peridotvault/contracts/solana";

// Try "local" | "staging" | "testnet" | "mainnet"
const programs = getSolanaPrograms("testnet");

console.log("=== testnet program IDs ===");
console.log("Registry:", programs.registry.toBase58());
console.log("PGL-1:", programs.pgl1.toBase58());
console.log("Game Store:", programs.store.toBase58());

try {
  getSolanaPrograms("mainnet");
} catch (err) {
  console.error("Expected error:", err.message);
}
`;

const templates: Record<Template, Record<string, { code: string }>> = {
  install: {
    "/index.ts": { code: installCode },
  },
  deployments: {
    "/index.ts": { code: deploymentsCode },
  },
  pda: {
    "/index.ts": { code: pdaCode },
  },
  "solana-programs": {
    "/index.ts": { code: solanaProgramsCode },
  },
};

const sandboxFiles = computed(() => {
  return props.files ?? templates[props.template ?? "install"] ?? templates.install;
});

const filesWithActive = computed(() => {
  const result: Record<string, { code: string; active?: boolean }> = {};
  for (const [path, file] of Object.entries(sandboxFiles.value)) {
    result[path] = { code: file.code };
  }
  if (result["/index.ts"]) {
    result["/index.ts"].active = true;
  }
  return result;
});
</script>

<template>
  <ClientOnly>
    <div
      class="sandbox-block"
      :style="{ height: `${props.height}px` }"
    >
      <SandpackProvider
        :files="filesWithActive"
        :custom-setup="{
          ...customSetup,
          files: sandpackNodeModules,
        }"
        template="vanilla-ts"
        :options="{
          visibleFiles: ['/index.ts'],
          activeFile: '/index.ts',
          showConsole: true,
          showConsoleButton: true,
        }"
      >
        <SandpackLayout>
          <SandpackCodeEditor
            v-if="showEditor"
            :show-line-numbers="true"
            :show-run-button="true"
            :style="{ height: `${props.height}px` }"
          />
          <SandpackPreview
            v-if="showPreview"
            :show-open-in-code-sandbox="false"
            :style="{ height: `${props.height}px` }"
          />
        </SandpackLayout>
      </SandpackProvider>
    </div>
    <template #fallback>
      <div class="sandbox-fallback">
        <p>Interactive editor loading...</p>
      </div>
    </template>
  </ClientOnly>
</template>
