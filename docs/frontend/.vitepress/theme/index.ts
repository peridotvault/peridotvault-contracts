import type { Theme } from "vitepress";
import DefaultTheme from "vitepress/theme";
import SandboxBlock from "./SandboxBlock.vue";
import RegistryPlayground from "./RegistryPlayground.vue";
import "./custom.css";
import "./style.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("SandboxBlock", SandboxBlock);
    app.component("RegistryPlayground", RegistryPlayground);
  },
} satisfies Theme;
