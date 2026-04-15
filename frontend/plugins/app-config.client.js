import { appConfig } from "../src/config";

function toBoolean(value, fallback = false) {
  if (typeof value === "boolean") {
    return value;
  }

  if (typeof value === "string") {
    return value.toLowerCase() === "true";
  }

  return fallback;
}

export default defineNuxtPlugin(() => {
  const runtimeConfig = useRuntimeConfig();
  const publicConfig = runtimeConfig.public || {};

  appConfig.defaultApiBaseUrl = publicConfig.apiBaseUrl || appConfig.defaultApiBaseUrl;
  appConfig.debugToolsEnabled = toBoolean(
    publicConfig.debugToolsEnabled,
    appConfig.debugToolsEnabled
  );
});
