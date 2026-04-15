export function prettyJson(value) {
  return JSON.stringify(value, null, 2);
}

export function parseJsonText(text) {
  if (!text?.trim()) {
    return undefined;
  }

  return JSON.parse(text);
}

export function buildApiUrl(baseUrl, path, query = {}) {
  const trimmedPath = String(path || "").trim();

  if (!trimmedPath) {
    throw new Error("A request path is required.");
  }

  const base = String(baseUrl || "").trim().replace(/\/+$/, "");
  const normalizedPath = trimmedPath.startsWith("/")
    ? trimmedPath
    : `/${trimmedPath}`;
  const rawUrl = /^https?:\/\//i.test(trimmedPath)
    ? trimmedPath
    : `${base}${normalizedPath}`;
  const url = /^https?:\/\//i.test(rawUrl)
    ? new URL(rawUrl)
    : new URL(rawUrl, window.location.origin);

  Object.entries(query).forEach(([key, value]) => {
    if (value === undefined || value === null || value === "") {
      return;
    }

    url.searchParams.set(key, value);
  });

  return url;
}

export function buildRequestHeaders({
  accountId,
  accessToken,
  extraHeaders = {},
  hasBody = false
}) {
  const headers = {};

  if (hasBody) {
    headers["Content-Type"] = "application/json";
  }

  if (accountId) {
    headers["x-account-id"] = accountId;
  }

  if (accessToken) {
    headers.Authorization = `Bearer ${accessToken}`;
  }

  return {
    ...headers,
    ...extraHeaders
  };
}

export async function sendJsonRequest({ url, method, headers, body }) {
  const response = await fetch(url, {
    method,
    credentials: "include",
    headers,
    body: body === undefined ? undefined : JSON.stringify(body)
  });

  const rawText = await response.text();
  const contentType = response.headers.get("content-type") || "";
  let data = null;

  if (rawText) {
    if (contentType.includes("application/json")) {
      data = JSON.parse(rawText);
    } else {
      try {
        data = JSON.parse(rawText);
      } catch (_error) {
        data = rawText;
      }
    }
  }

  return {
    ok: response.ok,
    status: response.status,
    headers: Object.fromEntries(response.headers.entries()),
    rawText,
    data
  };
}
