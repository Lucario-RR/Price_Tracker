import { getProxyRequestHeaders, getRequestURL, proxyRequest } from "h3";

function trimTrailingSlash(value: string) {
  return value.replace(/\/+$/, "");
}

export default defineEventHandler((event) => {
  const config = useRuntimeConfig();
  const requestUrl = getRequestURL(event);
  const backendBase = trimTrailingSlash(
    String(config.backendProxyTarget || "http://127.0.0.1:11451")
  );
  const target = `${backendBase}${requestUrl.pathname}${requestUrl.search}`;

  return proxyRequest(event, target, {
    headers: getProxyRequestHeaders(event, { host: false })
  });
});
