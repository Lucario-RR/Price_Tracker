const backendProxyTarget =
  process.env.NUXT_BACKEND_PROXY_TARGET ||
  process.env.VITE_BACKEND_PROXY_TARGET ||
  "http://127.0.0.1:3000";

const debugToolsEnabled =
  process.env.NUXT_PUBLIC_ENABLE_DEBUG_TOOLS === "true" ||
  process.env.VITE_ENABLE_DEBUG_TOOLS === "true";

export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2026-04-14",
  modules: ["@nuxt/ui"],
  css: ["~/assets/css/main.css"],
  devServer: {
    port: 5173
  },
  runtimeConfig: {
    backendProxyTarget,
    public: {
      apiBaseUrl:
        process.env.NUXT_PUBLIC_API_BASE_URL ||
        process.env.VITE_API_BASE_URL ||
        "/api/v1",
      debugToolsEnabled
    }
  },
  ui: {
    colorMode: false,
    fonts: false,
    theme: {
      defaultVariants: {
        color: "primary",
        size: "md"
      }
    }
  },
  app: {
    head: {
      title: "PriceTracker",
      meta: [
        {
          name: "viewport",
          content: "width=device-width, initial-scale=1, viewport-fit=cover"
        },
        {
          name: "theme-color",
          content: "#eef8f4"
        }
      ]
    }
  }
});
