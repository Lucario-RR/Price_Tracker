const env = import.meta.env || {};

export const appConfig = {
  storagePrefix: "pricetracker-nuxt",
  defaultApiBaseUrl:
    env.NUXT_PUBLIC_API_BASE_URL || env.VITE_API_BASE_URL || "/api/v1",
  debugToolsEnabled:
    env.DEV ||
    env.NUXT_PUBLIC_ENABLE_DEBUG_TOOLS === "true" ||
    env.VITE_ENABLE_DEBUG_TOOLS === "true"
};
