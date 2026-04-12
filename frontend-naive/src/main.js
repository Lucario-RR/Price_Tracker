import { createApp } from "vue";
import naive from "naive-ui";
import AppRoot from "./AppRoot.vue";
import "./styles.css";

createApp(AppRoot).use(naive).mount("#app");
