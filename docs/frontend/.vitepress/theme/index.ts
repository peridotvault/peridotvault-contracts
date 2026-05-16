import type { Theme } from "vitepress";
import DefaultTheme from "vitepress/theme";
import SandboxBlock from "./SandboxBlock.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("SandboxBlock", SandboxBlock);
  },
} satisfies Theme;
