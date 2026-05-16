<script setup lang="ts">
import { computed } from "vue";
import {
  SandpackProvider,
  SandpackLayout,
  SandpackCodeEditor,
  SandpackPreview,
} from "sandpack-vue3";

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

const installCode = `\
// ── @peridotvault/contracts — namespace overview ──────────
// In production, replace mocks with:
//   import { deployments, evm, solana } from "@peridotvault/contracts";

// ── Mock: solana namespace ────────────────────────────────
const solana = {
  /**
   * Returns program IDs for the given environment.
   * Real usage: solana.getSolanaPrograms("testnet")
   */
  getSolanaPrograms: (env: string) => ({
    registry: "REGISTRY_PUBKEY_PLACEHOLDER",
    pgl1: "PGL1_PUBKEY_PLACEHOLDER",
    gameStore: env === "testnet"
      ? "GAMESTORE_PUBKEY_PLACEHOLDER"
      : null,
    environment: env,
  }) as const,
};

// ── Mock: evm namespace ───────────────────────────────────
const evm = {
  /** Returns a deployed contract address. */
  getEvmContractAddress: (opts: {
    chain: string;
    environment: string;
    contract: string;
  }) => \`0x_\${opts.chain}_\${opts.contract}_\${opts.environment}\`,
};

// ── Mock: deployments namespace ───────────────────────────
const deployments = {
  SOLANA_DEPLOYMENTS: { testnet: { registry: "...", pgl1: "..." } },
  EVM_DEPLOYMENTS: { sepolia: { registry: "0x..." } },
  getDeployment: (opts: { chain: string; environment: string }) => ({
    chain: opts.chain,
    environment: opts.environment,
    programs: { registry: "REGISTRY", pgl1: "PGL1" },
  }),
};

// ── Inspect the API surface ───────────────────────────────
console.log("Available namespaces:");
console.log("  solana:", Object.keys(solana));
console.log("  evm:", Object.keys(evm));
console.log("  deployments:", Object.keys(deployments));

console.log("\\nTestnet programs:", solana.getSolanaPrograms("testnet"));
`;

const deploymentsCode = `\
// ── @peridotvault/contracts/deployments ───────────────────
// In production, replace with:
//   import { getDeployment } from "@peridotvault/contracts/deployments";

type Chain = "solana" | "ethereum" | "base";
type Environment = "local" | "staging" | "testnet" | "mainnet";

interface Deployment {
  chain: Chain;
  environment: Environment;
  programs: Record<string, string | null>;
}

function getDeployment(opts: { chain: Chain; environment: Environment }): Deployment {
  const chain = opts.chain;
  const env = opts.environment;

  if (env === "mainnet") {
    throw new Error(\`PeridotContractsError: no mainnet deployment for \${chain}\`);
  }

  return {
    chain,
    environment: env,
    programs: {
      registry: \`REGISTRY_PROGRAM_ID_\${chain}_\${env}\`,
      pgl1: \`PGL1_PROGRAM_ID_\${chain}_\${env}\`,
      ...(env === "testnet" ? { gameStore: \`GAMESTORE_PROGRAM_ID_\${chain}_\${env}\` } : {}),
    },
  };
}

// ── Try it ────────────────────────────────────────────────
// Edit chain / environment below:
const chain: Chain = "solana";
const env: Environment = "testnet";

const dep = getDeployment({ chain, environment: env });
console.log(\`\${chain} (\${env}):\`, JSON.stringify(dep, null, 2));

// Try unresolved mainnet:
try {
  getDeployment({ chain: "ethereum", environment: "mainnet" });
} catch (err) {
  console.error("\\nExpected error for unresolved env:", (err as Error).message);
}
`;

const pdaCode = `\
// ── @peridotvault/contracts/solana PDA helpers ────────────
// In production, replace with:
//   import { PublicKey } from "@solana/web3.js";
//   import { findLicensePda } from "@peridotvault/contracts/solana";

// ── Minimal PublicKey for PDA demonstration ───────────────
class PublicKey {
  constructor(private readonly base58: string) {}
  toBase58() { return this.base58; }
  toBuffer() { return Buffer.from(this.base58.slice(0, 32)); }

  /**
   * Derives a Program Derived Address from seeds + programId.
   * Real: PublicKey.findProgramAddressSync(seeds, programId) -> [PublicKey, bump]
   */
  static findProgramAddressSync(
    seeds: Buffer[],
    programId: PublicKey
  ): [PublicKey, number] {
    const seedHash = seeds.map(s => s.toString("hex")).join("");
    const bump = 255;
    return [
      new PublicKey("PDA_" + programId.toBase58().slice(0, 8) + "_" + seedHash.slice(0, 8)),
      bump,
    ];
  }
}

/**
 * Derives the License PDA for a (holder, game) pair.
 * Real signature: findLicensePda(holder: PublicKey, game: PublicKey) => [PublicKey, number]
 */
function findLicensePda(holder: PublicKey, game: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("license"), holder.toBuffer(), game.toBuffer()],
    new PublicKey("PGL1_PROGRAM_ID"),
  );
}

// ── Try it ────────────────────────────────────────────────
const holder = new PublicKey("11111111111111111111111111111111");
const game = new PublicKey("22222222222222222222222222222222");

const [licensePda, bump] = findLicensePda(holder, game);

console.log("Holder:", holder.toBase58());
console.log("Game:", game.toBase58());
console.log("License PDA:", licensePda.toBase58());
console.log("Bump seed:", bump);
`;

const solanaProgramsCode = `\
// ── @peridotvault/contracts/solana program IDs ────────────
// In production, replace with:
//   import { getSolanaPrograms } from "@peridotvault/contracts/solana";

type Environment = "local" | "staging" | "testnet" | "mainnet";

interface SolanaPrograms {
  registry: string;
  pgl1: string;
  gameStore: string | null;
  environment: Environment;
}

const KNOWN_PROGRAMS: Record<string, SolanaPrograms> = {
  testnet: {
    registry: "HgsBSzSmyKfuKnjR3zcJjMSDfJjqnn6WfJBDjtTV4qGA",
    pgl1: "9uCRABVFHG1hyG2VfPxrveFJs2dXiN7VJNYjNynDnGYa",
    gameStore: "6rZyXTFEMV6TfL3GurVmHUNjBwiH8etcoYw2BgNKmeUj",
    environment: "testnet",
  },
  local: {
    registry: "Fg6PaFpoGXkYsidMpWTK6W2BeZ7LfcZfRYYPf2GvQbJF",
    pgl1: "7uhe4L2gKtNtLhKqY4eKm5hvMx2KnPqJjQ2G6yHfFaVJ",
    gameStore: "8vHeKtNwLhKqY5eKm6hvNx3KnQqJjQ3H7yIfGbWgHdUk",
    environment: "local",
  },
};

function getSolanaPrograms(env: Environment): SolanaPrograms {
  const programs = KNOWN_PROGRAMS[env];
  if (!programs) {
    throw new Error(\`PeridotContractsError: no deployment for "\${env}"\`);
  }
  return programs;
}

// ── Try it ────────────────────────────────────────────────
// Edit the environment below: "local" | "staging" | "testnet" | "mainnet"
const env: Environment = "testnet";

const programs = getSolanaPrograms(env);
console.log(\`\\n=== \${env} program IDs ===\`);
console.log("Registry:", programs.registry);

if (programs.gameStore) {
  console.log("Game Store:", programs.gameStore);
} else {
  console.log("Game Store: not deployed on", env);
}

// Try unresolved mainnet:
try {
  getSolanaPrograms("mainnet");
} catch (err) {
  console.error("\\nExpected error:", (err as Error).message);
}
`;

const templates: Record<Template, Record<string, { code: string }>> = {
  install: {
    "/index.ts": { code: installCode },
    "/package.json": {
      code: JSON.stringify({ name: "sandbox", dependencies: {} }, null, 2),
    },
  },
  deployments: {
    "/index.ts": { code: deploymentsCode },
    "/package.json": {
      code: JSON.stringify({ name: "sandbox", dependencies: {} }, null, 2),
    },
  },
  pda: {
    "/index.ts": { code: pdaCode },
    "/package.json": {
      code: JSON.stringify({ name: "sandbox", dependencies: {} }, null, 2),
    },
  },
  "solana-programs": {
    "/index.ts": { code: solanaProgramsCode },
    "/package.json": {
      code: JSON.stringify({ name: "sandbox", dependencies: {} }, null, 2),
    },
  },
};

const sandboxFiles = computed(() => {
  if (props.files) return props.files;
  if (props.template) return templates[props.template];
  return templates.install;
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
