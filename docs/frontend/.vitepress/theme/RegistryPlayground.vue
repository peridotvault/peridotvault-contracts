<script setup lang="ts">
import { ref, computed, watch, shallowRef } from "vue";

type Environment = "local" | "testnet" | "staging" | "mainnet";

const CLUSTER_URLS: Record<Environment, string> = {
  local: "http://localhost:8899",
  testnet: "https://api.devnet.solana.com",
  staging: "https://api.devnet.solana.com",
  mainnet: "https://api.mainnet-beta.solana.com",
};

const PROGRAM_IDS: Record<string, Record<Environment, string>> = {
  registry: {
    local: "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE",
    testnet: "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE",
    staging: "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE",
    mainnet: "TODO",
  },
  pgl1: {
    local: "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP",
    testnet: "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP",
    staging: "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP",
    mainnet: "TODO",
  },
};

interface WalletLike {
  publicKey: any;
  signTransaction(tx: any): Promise<any>;
  signAllTransactions(txs: any[]): Promise<any[]>;
  connect?(): Promise<{ publicKey: any }>;
  disconnect?(): Promise<void>;
}

interface ParamDef {
  name: string;
  type: string;
  description: string;
  required: boolean;
}

interface MethodDef {
  key: string;
  name: string;
  description: string;
  params: ParamDef[];
  builder: (args: Record<string, string>, lib: any) => Promise<any>;
  read?: boolean;
}

const environment = ref<Environment>("testnet");
const wallet = ref<WalletLike | null>(null);
const connecting = ref(false);
const executing = ref(false);
const result = ref<string>("");
const error = ref<string>("");

const selectedCategory = ref("queries");
const selectedMethod = ref("");
const formValues = ref<Record<string, string>>({});

const libs = shallowRef<any>(null);

async function loadLibs() {
  if (libs.value) return libs.value;
  const [solana, anchor] = await Promise.all([
    import("@solana/web3.js"),
    import("@coral-xyz/anchor"),
  ]);
  libs.value = { solana, anchor };
  return libs.value;
}

function b(seed: string) { return new TextEncoder().encode(seed); }

async function detectWallet(): Promise<WalletLike | null> {
  const win = window as any;
  const w = win.solana || win.phantom?.solana || win.solflare || win.backpack;
  if (!w) return null;
  return {
    publicKey: w.publicKey ?? null,
    signTransaction: (tx: any) => w.signTransaction(tx),
    signAllTransactions: (txs: any[]) => w.signAllTransactions(txs),
    connect: async () => {
      const resp = await w.connect();
      return { publicKey: resp.publicKey };
    },
    disconnect: async () => { await w.disconnect?.(); },
  };
}

async function connectWallet() {
  connecting.value = true;
  error.value = "";
  try {
    const detected = await detectWallet();
    if (!detected) {
      error.value = "No Solana wallet detected. Install Phantom, Solflare, or Backpack.";
      return;
    }
    if (detected.connect) await detected.connect();
    detected.publicKey = (window as any).solana?.publicKey ?? null;
    if (!detected.publicKey) {
      error.value = "Wallet connection failed. Please unlock your wallet.";
      return;
    }
    wallet.value = detected;
    await loadLibs();
  } catch (e: any) {
    error.value = e.message;
  } finally {
    connecting.value = false;
  }
}

function disconnectWallet() {
  wallet.value?.disconnect?.();
  wallet.value = null;
  result.value = "";
  libs.value = null;
}

function categoryName(cat: string) {
  const map: Record<string, string> = {
    config: "Registry Config",
    payments: "Payment Tokens",
    grants: "Publish Grants",
    games: "Game Management",
    queries: "Queries",
  };
  return map[cat] ?? cat;
}

function toPubkey(s: string, L: any) {
  try { return new L.solana.PublicKey(s); } catch { return L.solana.PublicKey.default; }
}

function getMethods(): Record<string, MethodDef[]> {
  return {
    config: [
      {
        key: "initializeRegistry",
        name: "initializeRegistry",
        description: "Initialize the Registry with a treasury address.",
        params: [{ name: "treasury", type: "PublicKey", description: "Treasury public key", required: true }],
        builder: async (args, L) => {
          const treasury = toPubkey(args.treasury, L);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).initializeRegistry(treasury).accounts({
            authority: wallet.value!.publicKey, config: configPda,
            systemProgram: new L.solana.PublicKey("11111111111111111111111111111111"),
          }).instruction();
        },
      },
      {
        key: "setTreasury",
        name: "setTreasury",
        description: "Update the Registry treasury address.",
        params: [{ name: "treasury", type: "PublicKey", description: "New treasury public key", required: true }],
        builder: async (args, L) => {
          const treasury = toPubkey(args.treasury, L);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).setTreasury(treasury).accounts({
            authority: wallet.value!.publicKey, config: configPda,
          }).instruction();
        },
      },
    ],
    payments: [
      {
        key: "addPaymentToken",
        name: "addPaymentToken",
        description: "Add an accepted payment token with a fee amount.",
        params: [
          { name: "mint", type: "PublicKey", description: "Token mint address", required: true },
          { name: "feeAmount", type: "NumberLike", description: "Fee amount in base units", required: true },
        ],
        builder: async (args, L) => {
          const mint = toPubkey(args.mint, L);
          const feeAmount = new L.anchor.BN(args.feeAmount);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const [token] = L.solana.PublicKey.findProgramAddressSync([b("accepted_payment_token"), mint.toBuffer()], registryProgramId);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).addPaymentToken(feeAmount).accounts({
            authority: wallet.value!.publicKey, config: configPda, mint, acceptedPaymentToken: token,
            systemProgram: new L.solana.PublicKey("11111111111111111111111111111111"),
          }).instruction();
        },
      },
      {
        key: "updatePaymentToken",
        name: "updatePaymentToken",
        description: "Enable/disable a payment token and update its fee.",
        params: [
          { name: "mint", type: "PublicKey", description: "Token mint address", required: true },
          { name: "active", type: "boolean", description: "true or false", required: true },
          { name: "feeAmount", type: "NumberLike", description: "Fee amount in base units", required: true },
        ],
        builder: async (args, L) => {
          const mint = toPubkey(args.mint, L);
          const active = args.active === "true";
          const feeAmount = new L.anchor.BN(args.feeAmount);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const [token] = L.solana.PublicKey.findProgramAddressSync([b("accepted_payment_token"), mint.toBuffer()], registryProgramId);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).updatePaymentToken(active, feeAmount).accounts({
            authority: wallet.value!.publicKey, config: configPda, mint, acceptedPaymentToken: token,
          }).instruction();
        },
      },
      {
        key: "removePaymentToken",
        name: "removePaymentToken",
        description: "Remove an accepted payment token.",
        params: [{ name: "mint", type: "PublicKey", description: "Token mint address", required: true }],
        builder: async (args, L) => {
          const mint = toPubkey(args.mint, L);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const [token] = L.solana.PublicKey.findProgramAddressSync([b("accepted_payment_token"), mint.toBuffer()], registryProgramId);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).removePaymentToken().accounts({
            authority: wallet.value!.publicKey, config: configPda, mint, acceptedPaymentToken: token,
          }).instruction();
        },
      },
    ],
    grants: [
      {
        key: "createPublishGrant",
        name: "createPublishGrant",
        description: "Grant publish permission to a publisher address.",
        params: [
          { name: "publisher", type: "PublicKey", description: "Publisher address (optional, defaults to wallet)", required: false },
          { name: "expiredAt", type: "NumberLike|null", description: "Expiration timestamp (optional)", required: false },
        ],
        builder: async (args, L) => {
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const publisher = args.publisher ? toPubkey(args.publisher, L) : wallet.value!.publicKey;
          const [publishGrant] = L.solana.PublicKey.findProgramAddressSync([b("publish_grant"), publisher.toBuffer()], registryProgramId);
          const expiredAt = args.expiredAt ? new L.anchor.BN(args.expiredAt) : null;
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).createPublishGrant(expiredAt).accounts({
            authority: wallet.value!.publicKey, config: configPda, publisher, publishGrant,
            systemProgram: new L.solana.PublicKey("11111111111111111111111111111111"),
          }).instruction();
        },
      },
      {
        key: "updatePublishGrant",
        name: "updatePublishGrant",
        description: "Update an existing publish grant expiration.",
        params: [
          { name: "publisher", type: "PublicKey", description: "Publisher address (optional, defaults to wallet)", required: false },
          { name: "expiredAt", type: "NumberLike|null", description: "New expiration timestamp", required: false },
        ],
        builder: async (args, L) => {
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const publisher = args.publisher ? toPubkey(args.publisher, L) : wallet.value!.publicKey;
          const [publishGrant] = L.solana.PublicKey.findProgramAddressSync([b("publish_grant"), publisher.toBuffer()], registryProgramId);
          const expiredAt = args.expiredAt ? new L.anchor.BN(args.expiredAt) : null;
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).updatePublishGrant(expiredAt).accounts({
            authority: wallet.value!.publicKey, config: configPda, publisher, publishGrant,
          }).instruction();
        },
      },
    ],
    games: [
      {
        key: "updateGameStatus",
        name: "updateGameStatus",
        description: "Update game status (active=0, suspended=1, banned=2).",
        params: [
          { name: "registryGame", type: "PublicKey", description: "Registry game account address", required: true },
          { name: "status", type: "number", description: "0=active, 1=suspended, 2=banned", required: true },
        ],
        builder: async (args, L) => {
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const registryGame = toPubkey(args.registryGame, L);
          const status = parseInt(args.status);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          return (program.methods as any).updateGameStatus(status).accounts({
            authority: wallet.value!.publicKey, config: configPda, registryGame,
          }).instruction();
        },
      },
      {
        key: "closeRegistryGame",
        name: "closeRegistryGame",
        description: "Close a registry game account and reclaim rent.",
        params: [
          { name: "game", type: "PublicKey", description: "Game (PGL-1) account address", required: true },
          { name: "registryGame", type: "PublicKey", description: "Registry game account address", required: true },
        ],
        builder: async (args, L) => {
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const [configPda] = L.solana.PublicKey.findProgramAddressSync([b("registry_config")], registryProgramId);
          const game = toPubkey(args.game, L);
          const registryGame = toPubkey(args.registryGame, L);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          const config = await program.account.registryConfig.fetch(configPda);
          return (program.methods as any).closeRegistryGame().accounts({
            publisher: wallet.value!.publicKey, config: configPda, game, registryGame, treasury: config.treasury,
          }).instruction();
        },
      },
    ],
    queries: [
      {
        key: "fetchGames",
        name: "fetchGames",
        description: "Fetch all registered games from the Registry (no signature needed).",
        params: [],
        builder: async (_args, L) => {
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          const games = await program.account.registryGame.all();
          result.value = JSON.stringify(games.map((g: any) => ({
            publicKey: g.publicKey.toBase58(),
            game: g.account.game.toBase58(),
            gameId: g.account.gameId,
            registeredAt: g.account.registeredAt.toString(),
            status: JSON.stringify(g.account.status),
          })), null, 2);
          return null;
        },
        read: true,
      },
      {
        key: "fetchGame",
        name: "fetchGame",
        description: "Fetch a single registry game by its account address.",
        params: [{ name: "registryGame", type: "PublicKey", description: "Registry game account address", required: true }],
        builder: async (args, L) => {
          const registryGame = toPubkey(args.registryGame, L);
          const registryProgramId = new L.solana.PublicKey(PROGRAM_IDS.registry[environment.value]);
          const prov = new L.anchor.AnchorProvider(new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed"), wallet.value!, { commitment: "confirmed" });
          const program = new L.anchor.Program(IDLS as any, prov);
          const account = await program.account.registryGame.fetch(registryGame);
          result.value = JSON.stringify({
            game: account.game.toBase58(),
            gameId: account.gameId,
            registeredAt: account.registeredAt.toString(),
            status: JSON.stringify(account.status),
          }, null, 2);
          return null;
        },
        read: true,
      },
    ],
  };
}

const categories = computed(() => Object.keys(getMethods()));
const methods = computed(() => getMethods()[selectedCategory.value] ?? []);
const currentMethod = computed(() => {
  if (!selectedMethod.value && methods.value.length > 0) return methods.value[0];
  return methods.value.find((m) => m!.key === selectedMethod.value) ?? methods.value[0] ?? null;
});

watch(selectedCategory, () => {
  formValues.value = {};
  result.value = "";
  error.value = "";
  if (methods.value.length > 0) selectedMethod.value = methods.value[0]!.key;
});

watch(currentMethod, (m) => {
  if (m) {
    formValues.value = {};
    for (const p of m.params) formValues.value[p.name] = "";
  }
}, { immediate: true });

async function execute() {
  if (!wallet.value) return;
  executing.value = true;
  error.value = "";
  const m = currentMethod.value;
  if (!m) { executing.value = false; return; }

  try {
    const L = await loadLibs();
    const conn = new L.solana.Connection(CLUSTER_URLS[environment.value], "confirmed");

    if (m.read) {
      result.value = "";
      await m.builder(formValues.value, L);
      executing.value = false;
      return;
    }

    const ix = await m.builder(formValues.value, L);
    if (!ix) { executing.value = false; return; }

    const tx = new L.solana.Transaction().add(ix);
    tx.feePayer = wallet.value.publicKey;
    const bh = await conn.getLatestBlockhash();
    tx.recentBlockhash = bh.blockhash;

    const signed = await wallet.value.signTransaction(tx);
    const sig = await conn.sendRawTransaction(signed.serialize(), {
      skipPreflight: false,
      preflightCommitment: "confirmed",
    });
    await conn.confirmTransaction({ signature: sig, ...bh }, "confirmed");
    result.value = `Transaction sent: ${sig}`;
  } catch (e: any) {
    error.value = e.message ?? String(e);
  } finally {
    executing.value = false;
  }
}

function setParam(name: string, value: string) {
  formValues.value[name] = value;
}

const IDLS = {
  version: "0.1.0",
  name: "registry",
  instructions: [
    { name: "initializeRegistry", accounts: ["authority", "config", "systemProgram"], args: [{ name: "treasury", type: "publicKey" }] },
    { name: "setTreasury", accounts: ["authority", "config"], args: [{ name: "treasury", type: "publicKey" }] },
    { name: "addPaymentToken", accounts: ["authority", "config", "mint", "acceptedPaymentToken", "systemProgram"], args: [{ name: "feeAmount", type: "u64" }] },
    { name: "updatePaymentToken", accounts: ["authority", "config", "mint", "acceptedPaymentToken"], args: [{ name: "active", type: "bool" }, { name: "feeAmount", type: "u64" }] },
    { name: "removePaymentToken", accounts: ["authority", "config", "mint", "acceptedPaymentToken"], args: [] },
    { name: "createPublishGrant", accounts: ["authority", "config", "publisher", "publishGrant", "systemProgram"], args: [{ name: "expiredAt", type: { option: "i64" } }] },
    { name: "updatePublishGrant", accounts: ["authority", "config", "publisher", "publishGrant"], args: [{ name: "expiredAt", type: { option: "i64" } }] },
    { name: "updateGameStatus", accounts: ["authority", "config", "registryGame"], args: [{ name: "status", type: "u8" }] },
    { name: "closeRegistryGame", accounts: ["publisher", "config", "game", "registryGame", "treasury"], args: [] },
  ],
  accounts: [{ name: "registryConfig" }, { name: "registryGame" }],
};
</script>

<template>
  <ClientOnly>
    <div class="registry-playground">
      <div class="playground-toolbar">
        <select v-model="environment" class="env-select">
          <option value="local">localnet</option>
          <option value="testnet">testnet (devnet)</option>
          <option value="staging">staging</option>
          <option value="mainnet">mainnet</option>
        </select>
        <template v-if="!wallet">
          <button class="btn-connect" :disabled="connecting" @click="connectWallet">
            {{ connecting ? "Connecting..." : "Connect Wallet" }}
          </button>
        </template>
        <template v-else>
          <span class="wallet-address">{{ wallet.publicKey?.toBase58().slice(0, 10) }}...</span>
          <button class="btn-disconnect" @click="disconnectWallet">Disconnect</button>
        </template>
      </div>

      <div v-if="!wallet" class="playground-empty">
        <h3>Registry Playground</h3>
        <p>Connect a Solana wallet to execute Registry methods interactively.</p>
        <p class="hint">Supports Phantom, Solflare, Backpack.</p>
      </div>

      <div v-else class="playground-layout">
        <div class="playground-sidebar">
          <div v-for="cat in categories" :key="cat" class="category-group">
            <div
              class="category-header"
              :class="{ active: selectedCategory === cat }"
              @click="selectedCategory = cat"
            >
              {{ categoryName(cat) }}
            </div>
            <div
              v-if="selectedCategory === cat"
              v-for="m in getMethods()[cat]"
              :key="m.key"
              class="method-item"
              :class="{ active: selectedMethod === m.key }"
              @click="selectedMethod = m.key"
            >
              <span class="method-icon">{{ m.read ? "📖" : "✏️" }}</span>
              {{ m.name }}
            </div>
          </div>
        </div>

        <div class="playground-main" v-if="currentMethod">
          <h3 class="method-title">{{ currentMethod.name }}</h3>
          <p class="method-desc">{{ currentMethod.description }}</p>

          <div v-if="currentMethod.params.length > 0" class="params-form">
            <div v-for="p in currentMethod.params" :key="p.name" class="param-row">
              <label :for="'p-' + p.name">
                {{ p.name }}
                <span class="param-type">{{ p.type }}</span>
                <span v-if="!p.required" class="param-optional">optional</span>
              </label>
              <input
                :id="'p-' + p.name"
                :value="formValues[p.name] || ''"
                :placeholder="p.description"
                class="param-input"
                @input="setParam(p.name, ($event.target as HTMLInputElement).value)"
              />
            </div>
          </div>

          <button class="btn-execute" :disabled="executing || !wallet" @click="execute">
            {{ executing ? "Executing..." : currentMethod.read ? "Query" : "Send Transaction" }}
          </button>

          <div v-if="error" class="output-error">
            <strong>Error:</strong> {{ error }}
          </div>
          <div v-if="result" class="output-result">
            <strong>Result:</strong>
            <pre>{{ result }}</pre>
          </div>
        </div>
      </div>
    </div>
    <template #fallback>
      <div class="playground-fallback">Loading Registry Playground...</div>
    </template>
  </ClientOnly>
</template>

<style scoped>
.registry-playground { margin: 1.5rem 0; border: 1px solid var(--vp-c-divider); border-radius: 10px; overflow: hidden; background: var(--vp-c-bg-soft); }
.playground-toolbar { display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem 1rem; background: var(--vp-c-bg-elv); border-bottom: 1px solid var(--vp-c-divider); }
.env-select { padding: 0.35rem 0.75rem; border: 1px solid var(--vp-c-divider); border-radius: 6px; font-size: 13px; background: var(--vp-c-bg); color: var(--vp-c-text-1); cursor: pointer; }
.btn-connect, .btn-disconnect, .btn-execute { padding: 0.4rem 1rem; border-radius: 6px; font-size: 13px; font-weight: 600; cursor: pointer; border: none; transition: opacity 0.15s; }
.btn-connect { background: var(--vp-c-brand-1); color: #fff; }
.btn-connect:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-disconnect { background: var(--vp-c-danger-1); color: #fff; }
.btn-execute { margin-top: 1rem; width: 100%; padding: 0.6rem; font-size: 14px; background: var(--vp-c-brand-1); color: #fff; }
.btn-execute:disabled { opacity: 0.6; cursor: not-allowed; }
.wallet-address { font-size: 12px; font-family: monospace; color: var(--vp-c-text-2); margin-left: auto; }
.playground-empty { padding: 3rem 2rem; text-align: center; }
.playground-empty h3 { margin-bottom: 0.5rem; }
.playground-empty p { color: var(--vp-c-text-2); font-size: 14px; }
.playground-empty .hint { font-size: 12px; color: var(--vp-c-text-3); }
.playground-layout { display: flex; min-height: 400px; }
.playground-sidebar { width: 220px; flex-shrink: 0; border-right: 1px solid var(--vp-c-divider); overflow-y: auto; max-height: 600px; }
.category-header { padding: 0.6rem 1rem; font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; color: var(--vp-c-text-2); cursor: pointer; transition: background 0.15s; }
.category-header:hover, .category-header.active { background: var(--vp-c-bg-elv); color: var(--vp-c-text-1); }
.method-item { padding: 0.5rem 1rem 0.5rem 1.5rem; font-size: 13px; font-family: monospace; cursor: pointer; color: var(--vp-c-text-1); }
.method-item:hover { background: var(--vp-c-bg-elv); }
.method-item.active { background: var(--vp-c-brand-soft); color: var(--vp-c-brand-1); font-weight: 600; }
.method-icon { margin-right: 0.4rem; }
.playground-main { flex: 1; padding: 1.25rem; overflow-y: auto; max-height: 600px; }
.method-title { font-size: 18px; font-family: monospace; margin-bottom: 0.25rem; }
.method-desc { font-size: 13px; color: var(--vp-c-text-2); margin-bottom: 1.25rem; }
.params-form { display: flex; flex-direction: column; gap: 0.75rem; }
.param-row { display: flex; flex-direction: column; gap: 0.3rem; }
.param-row label { font-size: 12px; font-weight: 600; font-family: monospace; color: var(--vp-c-text-2); }
.param-type { color: var(--vp-c-brand-1); margin-left: 0.5rem; font-weight: 400; }
.param-optional { color: var(--vp-c-text-3); margin-left: 0.25rem; font-style: italic; }
.param-input { padding: 0.45rem 0.6rem; border: 1px solid var(--vp-c-divider); border-radius: 6px; font-size: 13px; font-family: monospace; background: var(--vp-c-bg); color: var(--vp-c-text-1); }
.param-input:focus { outline: none; border-color: var(--vp-c-brand-1); }
.output-error { margin-top: 1rem; padding: 0.75rem; background: var(--vp-c-danger-soft); border: 1px solid var(--vp-c-danger-1); border-radius: 6px; font-size: 13px; color: var(--vp-c-danger-1); font-family: monospace; word-break: break-all; }
.output-result { margin-top: 1rem; padding: 0.75rem; background: var(--vp-c-bg-elv); border: 1px solid var(--vp-c-divider); border-radius: 6px; font-size: 13px; }
.output-result pre { margin: 0.5rem 0 0; font-family: monospace; font-size: 12px; white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.playground-fallback { padding: 3rem 2rem; text-align: center; color: var(--vp-c-text-2); }
</style>
