export const appConfig = {
  storagePrefix: "pricetracker-vue",
  defaultApiBaseUrl: import.meta.env.VITE_API_BASE_URL || "/api/v1",
  debugToolsEnabled:
    import.meta.env.DEV || import.meta.env.VITE_ENABLE_DEBUG_TOOLS === "true"
};
