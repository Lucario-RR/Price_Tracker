import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

import { useOnlineStatus } from "./useOnlineStatus";
import { usePersistentState } from "./usePersistentState";
import { appConfig } from "../config";
import { ENDPOINTS } from "../data/endpoints";
import {
  buildApiUrl,
  buildRequestHeaders,
  parseJsonText,
  prettyJson,
  sendJsonRequest
} from "../lib/api";

const endpointMap = new Map(ENDPOINTS.map((endpoint) => [endpoint.id, endpoint]));
const DEMO_PASSWORD = "StrongPassword!234";
const HEALTH_POLL_INTERVAL_MS = 15000;

function pad(value) {
  return String(value).padStart(2, "0");
}

function createLocalDateTimeValue(date = new Date()) {
  return [
    date.getFullYear(),
    "-",
    pad(date.getMonth() + 1),
    "-",
    pad(date.getDate()),
    "T",
    pad(date.getHours()),
    ":",
    pad(date.getMinutes())
  ].join("");
}

function toIsoString(localValue) {
  if (!localValue) {
    return new Date().toISOString();
  }

  return new Date(localValue).toISOString();
}

function createId() {
  if (globalThis.crypto?.randomUUID) {
    return globalThis.crypto.randomUUID();
  }

  return `local-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function cloneValue(value) {
  if (typeof structuredClone === "function") {
    return structuredClone(value);
  }

  return JSON.parse(JSON.stringify(value));
}

function createInitialWorkbench() {
  return {
    workspace: {
      apiBaseUrl: appConfig.defaultApiBaseUrl
    },
    ui: {
      activeView: "public"
    },
    session: {
      accountId: "",
      accessToken: "",
      email: "",
      password: "",
      displayName: "",
      primaryPhone: "",
      roles: [],
      scopes: []
    },
    lookup: {
      shopId: "",
      itemId: "",
      variantId: "",
      purchaseId: "",
      priceId: "",
      fileId: "",
      alertId: "",
      watchId: "",
      emailId: "",
      phoneId: ""
    },
    capture: {
      selectedDeviceId: "",
      lookupCode: "",
      manualCode: "",
      recentCaptures: []
    },
    submission: {
      purchaseTime: createLocalDateTimeValue(),
      purchaseNotes: "Captured from the Vue workbench",
      attachmentFileIds: "",
      originalAmount: "1.80",
      originalCurrency: "GBP",
      discountAmount: "",
      discountCurrency: "GBP",
      discountTypeId: "",
      recordedAt: createLocalDateTimeValue(),
      priceNotes: "Submitted from the Vue workbench",
      fileUpload: {
        filename: "receipt.jpg",
        contentType: "image/jpeg",
        size: "120000",
        purpose: "PRICE_EVIDENCE",
        checksumSha256: ""
      }
    },
    explorer: {
      selectedEndpointId: "health",
      selectedGroup: "All",
      search: "",
      customMethod: "GET",
      customPath: "/health",
      pathParams: {},
      queryParams: {},
      bodyText: "",
      extraHeaders: [],
      queueOnFailure: true
    }
  };
}

function createInitialHealthState() {
  return {
    status: "checking",
    service: "pricetracker-backend",
    api: {
      status: "checking",
      connected: false,
      detail: "Waiting for the first backend probe.",
      checkedAt: ""
    },
    database: {
      status: "unknown",
      connected: false,
      detail: "Waiting for the first database probe.",
      checkedAt: ""
    },
    appliedMigrations: [],
    utcTime: "",
    lastError: ""
  };
}

function createInitialAdminState() {
  return {
    overview: null,
    users: [],
    table: null,
    rows: [],
    lookups: {},
    selectedTableId: "categories",
    selectedRecordId: "",
    draftMode: "create",
    draftValues: {},
    error: "",
    loadingTable: false,
    loadingUsers: false,
    savingRecord: false,
    deletingRecord: false,
    approvingRecord: false,
    savingUsers: false,
    moderationPrices: [],
    loadingModeration: false
  };
}

function createDefaultAdminValue(column) {
  if (column.input === "boolean") {
    return column.key === "isActive";
  }

  if (column.key === "timezoneName") {
    return "Europe/London";
  }

  if (column.key === "status") {
    return "approved";
  }

  if (column.key === "retailerType") {
    return "SUPERMARKET";
  }

  return "";
}

function cloneAdminDraft(table, row = null) {
  const draft = {};

  ensureArray(table?.columns).forEach((column) => {
    draft[column.key] =
      row && Object.prototype.hasOwnProperty.call(row, column.key)
        ? row[column.key]
        : createDefaultAdminValue(column);
  });

  return draft;
}

function mergeDeep(base, incoming) {
  if (Array.isArray(base)) {
    return Array.isArray(incoming) ? incoming : [...base];
  }

  if (base && typeof base === "object") {
    const merged = { ...base };
    const source = incoming && typeof incoming === "object" ? incoming : {};

    Object.keys(source).forEach((key) => {
      merged[key] =
        key in base ? mergeDeep(base[key], source[key]) : source[key];
    });

    return merged;
  }

  return incoming === undefined ? base : incoming;
}

function ensureArray(value) {
  return Array.isArray(value) ? value : [];
}

function removeEmptyValues(value) {
  if (Array.isArray(value)) {
    return value
      .map((item) => removeEmptyValues(item))
      .filter((item) => item !== "" && item !== undefined && item !== null);
  }

  if (value && typeof value === "object") {
    return Object.entries(value).reduce((accumulator, [key, entry]) => {
      const cleaned = removeEmptyValues(entry);
      if (cleaned === "" || cleaned === undefined || cleaned === null) {
        return accumulator;
      }

      if (
        Array.isArray(cleaned) &&
        cleaned.length === 0 &&
        key !== "acceptedLegalDocuments" &&
        key !== "variantIds" &&
        key !== "attachmentFileIds"
      ) {
        return accumulator;
      }

      accumulator[key] = cleaned;
      return accumulator;
    }, {});
  }

  return value;
}

function templateValueForKey(context, key) {
  return context[key] ?? "";
}

function resolveTemplateValue(template, context) {
  if (Array.isArray(template)) {
    return template.map((entry) => resolveTemplateValue(entry, context));
  }

  if (template && typeof template === "object") {
    return Object.fromEntries(
      Object.entries(template).map(([key, value]) => [
        key,
        resolveTemplateValue(value, context)
      ])
    );
  }

  if (typeof template !== "string") {
    return template;
  }

  const exactMatch = template.match(/^\$\{([^}]+)\}$/);
  if (exactMatch) {
    return templateValueForKey(context, exactMatch[1]);
  }

  return template.replace(/\$\{([^}]+)\}/g, (_match, key) =>
    String(templateValueForKey(context, key))
  );
}

function buildContext(model) {
  return {
    email: model.session.email,
    password: model.session.password,
    displayName: model.session.displayName,
    primaryPhone: model.session.primaryPhone,
    accountId: model.session.accountId,
    accessToken: model.session.accessToken,
    shopId: model.lookup.shopId,
    itemId: model.lookup.itemId,
    variantId: model.lookup.variantId,
    itemVariantId: model.lookup.variantId,
    purchaseId: model.lookup.purchaseId,
    priceId: model.lookup.priceId,
    fileId: model.lookup.fileId,
    alertId: model.lookup.alertId,
    watchId: model.lookup.watchId,
    emailId: model.lookup.emailId,
    phoneId: model.lookup.phoneId,
    code: model.capture.lookupCode,
    purchaseTimeIso: toIsoString(model.submission.purchaseTime),
    recordedAtIso: toIsoString(model.submission.recordedAt),
    originalAmount: model.submission.originalAmount,
    originalCurrency: model.submission.originalCurrency,
    discountAmount: model.submission.discountAmount,
    discountCurrency: model.submission.discountCurrency,
    discountTypeId: model.submission.discountTypeId,
    tableId: model.admin?.selectedTableId || "categories",
    recordId: model.admin?.selectedRecordId || ""
  };
}

function buildDraftFromEndpoint(endpoint, workbench, adminState = null) {
  const context = buildContext({
    ...workbench,
    admin: adminState
  });
  const pathParams = Object.fromEntries(
    (endpoint.pathParams || []).map((param) => [
      param,
      resolveTemplateValue(endpoint.pathDefaults?.[param] ?? "", context)
    ])
  );
  const queryParams = Object.fromEntries(
    (endpoint.queryParams || []).map((param) => [
      param,
      resolveTemplateValue(endpoint.queryDefaults?.[param] ?? "", context)
    ])
  );
  const resolvedBody = endpoint.bodyTemplate
    ? removeEmptyValues(resolveTemplateValue(endpoint.bodyTemplate, context))
    : undefined;

  return {
    pathParams,
    queryParams,
    bodyText: resolvedBody ? prettyJson(resolvedBody) : "",
    customMethod: endpoint.method,
    customPath: endpoint.path,
    queueOnFailure: endpoint.method !== "GET"
  };
}

function isObjectRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function isBackendSuccessEnvelope(payload) {
  return (
    isObjectRecord(payload) &&
    Object.prototype.hasOwnProperty.call(payload, "data") &&
    isObjectRecord(payload.meta) &&
    typeof payload.meta.requestId === "string"
  );
}

function isBackendErrorEnvelope(payload) {
  return (
    isObjectRecord(payload) &&
    isObjectRecord(payload.error) &&
    typeof payload.error.code === "string" &&
    typeof payload.error.message === "string"
  );
}

function resultErrorMessage(result, fallbackMessage) {
  if (isBackendErrorEnvelope(result?.data)) {
    return result.data.error.message;
  }

  if (typeof result?.error === "string" && result.error.trim()) {
    return result.error;
  }

  if (result?.status) {
    return `${fallbackMessage} (HTTP ${result.status}).`;
  }

  return fallbackMessage;
}

function isValidHealthEnvelope(payload) {
  return (
    isBackendSuccessEnvelope(payload) &&
    payload.data?.service === "pricetracker-backend" &&
    isObjectRecord(payload.data?.api) &&
    isObjectRecord(payload.data?.database)
  );
}

export function useWorkbench() {
  const browserOnline = useOnlineStatus();
  const busyAction = ref("");
  const explorerError = ref("");
  const isFlushingQueue = ref(false);
  const health = ref(createInitialHealthState());
  const admin = ref(createInitialAdminState());
  const healthTimer = ref(null);

  const defaultWorkbench = createInitialWorkbench();
  const workbenchStorageKey = `${appConfig.storagePrefix}:workbench`;
  const queueStorageKey = `${appConfig.storagePrefix}:queue`;
  const historyStorageKey = `${appConfig.storagePrefix}:history`;
  const workbench = usePersistentState(workbenchStorageKey, defaultWorkbench);
  const queue = usePersistentState(queueStorageKey, []);
  const history = usePersistentState(historyStorageKey, []);

  workbench.value = mergeDeep(defaultWorkbench, workbench.value);
  workbench.value.session.roles = ensureArray(workbench.value.session.roles);
  workbench.value.session.scopes = ensureArray(workbench.value.session.scopes);
  queue.value = ensureArray(queue.value);
  history.value = ensureArray(history.value);

  const debugToolsEnabled = appConfig.debugToolsEnabled;
  const isBusy = computed(() => Boolean(busyAction.value) || isFlushingQueue.value);
  const isAdmin = computed(() =>
    ensureArray(workbench.value.session.roles).includes("admin")
  );
  const apiOnline = computed(() => Boolean(health.value.api?.connected));
  const databaseOnline = computed(() => Boolean(health.value.database?.connected));
  const online = computed(() => browserOnline.value && apiOnline.value);
  const selectedEndpoint = computed(
    () => endpointMap.get(workbench.value.explorer.selectedEndpointId) || ENDPOINTS[0]
  );
  const endpointGroups = computed(() => [
    "All",
    ...new Set(ENDPOINTS.map((endpoint) => endpoint.group))
  ]);
  const filteredEndpoints = computed(() => {
    const search = workbench.value.explorer.search.trim().toLowerCase();

    return ENDPOINTS.filter((endpoint) => {
      const matchesGroup =
        workbench.value.explorer.selectedGroup === "All" ||
        endpoint.group === workbench.value.explorer.selectedGroup;
      const matchesSearch =
        !search ||
        endpoint.label.toLowerCase().includes(search) ||
        endpoint.path.toLowerCase().includes(search) ||
        endpoint.description.toLowerCase().includes(search);

      return matchesGroup && matchesSearch;
    });
  });
  const summaryCards = computed(() => [
    {
      label: "Queued sync jobs",
      value: String(queue.value.length),
      tone: queue.value.length ? "info" : "quiet"
    },
    {
      label: "Current user",
      value:
        workbench.value.session.displayName ||
        workbench.value.session.email ||
        "Not signed in",
      tone: workbench.value.session.accountId ? "good" : "quiet"
    },
    {
      label: "Selected store",
      value: workbench.value.lookup.shopId || "Choose a shop for capture and lookup",
      tone: workbench.value.lookup.shopId ? "info" : "quiet"
    },
    {
      label: "Release mode",
      value: debugToolsEnabled ? "Admin explorer available" : "Explorer hidden in production",
      tone: debugToolsEnabled ? "accent" : "good"
    }
  ]);
  const adminSummaryCards = computed(() => [
    {
      label: "Accounts",
      value: String(admin.value.overview?.accountCount || 0),
      tone: "info"
    },
    {
      label: "Shops",
      value: String(admin.value.overview?.shopCount || 0),
      tone: "good"
    },
    {
      label: "Items",
      value: String(admin.value.overview?.itemCount || 0),
      tone: "accent"
    },
    {
      label: "Variants",
      value: String(admin.value.overview?.itemVariantCount || 0),
      tone: "accent"
    },
    {
      label: "Pending moderation",
      value: String(admin.value.overview?.pendingModerationCount || 0),
      tone:
        (admin.value.overview?.pendingModerationCount || 0) > 0 ? "warn" : "good"
    },
    {
      label: "Settings",
      value: String(admin.value.overview?.systemSettingCount || 0),
      tone: "quiet"
    }
  ]);
  const statusLights = computed(() => [
    {
      key: "browser",
      label: "Browser",
      state: browserOnline.value ? "Connected" : "Offline",
      tone: browserOnline.value ? "good" : "warn",
      detail: browserOnline.value
        ? "This device can reach the network."
        : "The browser is offline, so requests are being kept locally."
    },
    {
      key: "api",
      label: "Backend API",
      state:
        health.value.api.status === "checking"
          ? "Checking"
          : apiOnline.value
            ? "Reachable"
            : "Unavailable",
      tone:
        health.value.api.status === "checking"
          ? "info"
          : apiOnline.value
            ? "good"
            : "warn",
      detail: health.value.api.detail
    },
    {
      key: "database",
      label: "Database",
      state:
        health.value.database.status === "unknown"
          ? "Checking"
          : databaseOnline.value
            ? "Connected"
            : "Unavailable",
      tone:
        health.value.database.status === "unknown"
          ? "info"
          : databaseOnline.value
            ? "good"
            : "warn",
      detail: health.value.database.detail
    }
  ]);
  const adminTables = computed(() => admin.value.overview?.tables || []);
  const adminTable = computed(() => {
    if (admin.value.table) {
      return admin.value.table;
    }

    return (
      adminTables.value.find(
        (table) => table.id === admin.value.selectedTableId
      ) || null
    );
  });

  watch(
    () => workbench.value.explorer.selectedEndpointId,
    () => {
      const draft = buildDraftFromEndpoint(
        selectedEndpoint.value,
        workbench.value,
        admin.value
      );
      workbench.value.explorer.pathParams = draft.pathParams;
      workbench.value.explorer.queryParams = draft.queryParams;
      workbench.value.explorer.bodyText = draft.bodyText;
      workbench.value.explorer.customMethod = draft.customMethod;
      workbench.value.explorer.customPath = draft.customPath;
      workbench.value.explorer.queueOnFailure = draft.queueOnFailure;
      workbench.value.explorer.extraHeaders = [];
      explorerError.value = "";
    },
    { immediate: true }
  );

  function requestMethodOptions() {
    return ["GET", "POST", "PATCH", "DELETE"];
  }

  function withBusyState(label, task) {
    if (busyAction.value) {
      return Promise.resolve(null);
    }

    busyAction.value = label;
    return Promise.resolve(task()).finally(() => {
      busyAction.value = "";
    });
  }

  function fillPathTemplate(pathTemplate, pathParams) {
    return pathTemplate.replace(/\{([^}]+)\}/g, (_match, key) => {
      const value = pathParams[key];

      if (!value) {
        throw new Error(`Path parameter "${key}" is required.`);
      }

      return encodeURIComponent(String(value));
    });
  }

  function headerPairsToObject(pairs) {
    return ensureArray(pairs).reduce((headers, pair) => {
      if (!pair?.key) {
        return headers;
      }

      headers[pair.key] = pair.value || "";
      return headers;
    }, {});
  }

  function parseAttachmentFileIds() {
    const raw = workbench.value.submission.attachmentFileIds
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);

    return raw.length ? raw : [];
  }

  function appendHistory(record) {
    history.value = [record, ...history.value].slice(0, 18);
  }

  function recordLocalEvent(label, message, level = "info") {
    appendHistory({
      id: createId(),
      label,
      method: "LOCAL",
      url: "client://workspace",
      status: 0,
      ok: level !== "error",
      startedAt: new Date().toISOString(),
      durationMs: 0,
      requestHeaders: {},
      requestBodyText: "",
      responseHeaders: {},
      responsePretty: message
    });
  }

  function removeStorageKey(storage, key) {
    try {
      storage.removeItem(key);
    } catch (_error) {
      // Ignore storage cleanup failures so sign-out can continue.
    }
  }

  function clearAppOwnedStorage() {
    [workbenchStorageKey, queueStorageKey, historyStorageKey].forEach((key) => {
      removeStorageKey(window.localStorage, key);
      removeStorageKey(window.sessionStorage, key);
    });

    try {
      Object.keys(window.localStorage)
        .filter((key) => key.startsWith(appConfig.storagePrefix))
        .forEach((key) => window.localStorage.removeItem(key));
    } catch (_error) {
      // Ignore localStorage cleanup failures so sign-out can continue.
    }

    try {
      Object.keys(window.sessionStorage)
        .filter((key) => key.startsWith(appConfig.storagePrefix))
        .forEach((key) => window.sessionStorage.removeItem(key));
    } catch (_error) {
      // Ignore sessionStorage cleanup failures so sign-out can continue.
    }
  }

  function resetClientState() {
    workbench.value = cloneValue(defaultWorkbench);
    queue.value = [];
    history.value = [];
    admin.value = createInitialAdminState();
    explorerError.value = "";
  }

  function clearClientState() {
    clearAppOwnedStorage();
    resetClientState();
  }

  function isProtectedRequest(path) {
    return (
      path === "/auth/refresh" ||
      path === "/auth/logout" ||
      path === "/auth/password/change" ||
      path === "/purchases" ||
      path === "/prices" ||
      path.startsWith("/me") ||
      path.startsWith("/files/uploads") ||
      path.startsWith("/admin") ||
      path.startsWith("/privacy/cookie-preferences")
    );
  }

  function setApiReachable(detail) {
    health.value = {
      ...health.value,
      lastError: "",
      api: {
        ...health.value.api,
        status: "ok",
        connected: true,
        detail,
        checkedAt: new Date().toISOString()
      }
    };
  }

  function setApiUnavailable(detail) {
    health.value = {
      ...health.value,
      status: "degraded",
      lastError: detail,
      api: {
        ...health.value.api,
        status: "offline",
        connected: false,
        detail,
        checkedAt: new Date().toISOString()
      },
      database: {
        ...health.value.database,
        status: "offline",
        connected: false,
        detail: "Database status cannot be confirmed while the backend health check is unavailable.",
        checkedAt: new Date().toISOString()
      }
    };
  }

  function applyHealthPayload(payload) {
    const data = payload?.data || payload;
    if (!data) {
      return;
    }

    health.value = {
      status: data.status || "unknown",
      service: data.service || "pricetracker-backend",
      api: {
        status: data.api?.status || "unknown",
        connected: Boolean(data.api?.connected),
        detail: data.api?.detail || "No API detail was returned.",
        checkedAt: data.api?.checkedAt || ""
      },
      database: {
        status: data.database?.status || "unknown",
        connected: Boolean(data.database?.connected),
        detail: data.database?.detail || "No database detail was returned.",
        checkedAt: data.database?.checkedAt || ""
      },
      appliedMigrations: ensureArray(data.appliedMigrations),
      utcTime: data.utcTime || "",
      lastError: ""
    };
  }

  function captureResponseState(endpointId, payload) {
    const data = payload?.data;

    if (!data) {
      return;
    }

    if (endpointId === "health") {
      applyHealthPayload(payload);
      return;
    }

    if (
      endpointId === "auth-login" ||
      endpointId === "auth-register" ||
      endpointId === "auth-register-admin" ||
      endpointId === "auth-refresh"
    ) {
      if (data.user?.id) {
        workbench.value.session.accountId = data.user.id;
      }
      if (data.accessToken) {
        workbench.value.session.accessToken = data.accessToken;
      }
      if (data.user?.displayName) {
        workbench.value.session.displayName = data.user.displayName;
      }
      if (Array.isArray(data.user?.roles)) {
        workbench.value.session.roles = data.user.roles;
      }
      if (Array.isArray(data.user?.scopes)) {
        workbench.value.session.scopes = data.user.scopes;
      }
    }

    if (endpointId === "me-get" && data.id) {
      workbench.value.session.accountId = data.id;
      workbench.value.session.displayName =
        data.displayName || workbench.value.session.displayName;
      workbench.value.session.roles = ensureArray(data.roles);
      workbench.value.session.scopes = ensureArray(data.scopes);
    }

    if (endpointId === "admin-overview") {
      admin.value.overview = data;
    }

    if (endpointId === "admin-users-list" && Array.isArray(data)) {
      admin.value.users = ensureArray(data);
    }

    if (endpointId === "moderation-prices") {
      admin.value.moderationPrices = ensureArray(data);
    }

    if (
      endpointId === "shops-list" &&
      Array.isArray(data) &&
      data[0]?.id &&
      !workbench.value.lookup.shopId
    ) {
      workbench.value.lookup.shopId = data[0].id;
    }

    if (
      endpointId === "items-list" &&
      Array.isArray(data) &&
      data[0]?.id &&
      !workbench.value.lookup.itemId
    ) {
      workbench.value.lookup.itemId = data[0].id;
    }

    if (endpointId === "item-detail" && data.id) {
      workbench.value.lookup.itemId = data.id;
    }

    if (
      endpointId === "item-variants" &&
      Array.isArray(data) &&
      data[0]?.id &&
      !workbench.value.lookup.variantId
    ) {
      workbench.value.lookup.variantId = data[0].id;
    }

    if (endpointId === "variant-detail" && (data.id || data.summary?.id)) {
      workbench.value.lookup.variantId = data.id || data.summary.id;
    }

    if (endpointId === "product-code-lookup" && data.variantId) {
      workbench.value.lookup.variantId = data.variantId;
    }

    if (endpointId === "file-upload-intent" && data.fileId) {
      workbench.value.lookup.fileId = data.fileId;
    }

    if (endpointId === "file-upload-complete" && data.id) {
      workbench.value.lookup.fileId = data.id;
    }

    if (endpointId === "purchase-create" && data.id) {
      workbench.value.lookup.purchaseId = data.id;
    }

    if (endpointId === "price-create" && data.id) {
      workbench.value.lookup.priceId = data.id;
    }

    if (endpointId === "watchlist-create" && data.id) {
      workbench.value.lookup.watchId = data.id;
    }

    if ((endpointId === "alert-create" || endpointId === "alert-update") && data.id) {
      workbench.value.lookup.alertId = data.id;
    }

    if ((endpointId === "emails-create" || endpointId === "emails-verify") && data.id) {
      workbench.value.lookup.emailId = data.id;
    }

    if ((endpointId === "phones-create" || endpointId === "phones-verify") && data.id) {
      workbench.value.lookup.phoneId = data.id;
    }
  }

  async function executeNetworkRequest(preparedRequest, options = {}) {
    const url = buildApiUrl(
      workbench.value.workspace.apiBaseUrl,
      preparedRequest.path,
      preparedRequest.query
    );
    const headers = buildRequestHeaders({
      accountId: workbench.value.session.accountId,
      accessToken: workbench.value.session.accessToken,
      extraHeaders: preparedRequest.extraHeaders,
      hasBody: preparedRequest.body !== undefined
    });
    const startedAt = new Date().toISOString();
    const started = performance.now();
    const response = await sendJsonRequest({
      url,
      method: preparedRequest.method,
      headers,
      body: preparedRequest.body
    });
    const durationMs = Math.round(performance.now() - started);
    const responsePretty =
      response.data === null
        ? response.rawText || "(no response body)"
        : typeof response.data === "string"
          ? response.data
          : prettyJson(response.data);

    if (
      preparedRequest.endpointId !== "health" &&
      (isBackendSuccessEnvelope(response.data) || isBackendErrorEnvelope(response.data))
    ) {
      setApiReachable(`Last response came from ${preparedRequest.label}.`);
    }

    if (options.recordHistory !== false) {
      appendHistory({
        id: createId(),
        label: preparedRequest.label,
        method: preparedRequest.method,
        url: url.toString(),
        status: response.status,
        ok: response.ok,
        startedAt,
        durationMs,
        requestHeaders: headers,
        requestBodyText:
          preparedRequest.body === undefined ? "" : prettyJson(preparedRequest.body),
        responseHeaders: response.headers,
        responsePretty
      });
    }

    captureResponseState(preparedRequest.endpointId, response.data);

    if (
      response.status === 401 &&
      workbench.value.session.accountId &&
      isProtectedRequest(preparedRequest.path)
    ) {
      clearClientState();
    }

    return {
      ok: response.ok,
      status: response.status,
      data: response.data,
      error: response.data?.error?.message || ""
    };
  }

  function addQueueEntry(entry) {
    queue.value = [entry, ...queue.value].slice(0, 30);
  }

  function enqueuePreparedRequest(preparedRequest, status, lastError = "") {
    addQueueEntry({
      id: createId(),
      kind: "request",
      status,
      label: preparedRequest.label,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      attemptCount: 0,
      lastError,
      payload: preparedRequest
    });
  }

  function prepareExplorerRequest() {
    const endpoint = selectedEndpoint.value;
    const method =
      endpoint.id === "custom"
        ? workbench.value.explorer.customMethod
        : endpoint.method;
    const pathTemplate =
      endpoint.id === "custom"
        ? workbench.value.explorer.customPath
        : endpoint.path;
    const path = fillPathTemplate(pathTemplate, workbench.value.explorer.pathParams);
    const query = Object.fromEntries(
      Object.entries(workbench.value.explorer.queryParams || {}).filter(
        ([, value]) => value !== ""
      )
    );
    const bodyText = workbench.value.explorer.bodyText.trim();
    const body =
      method === "GET" || method === "DELETE" || !bodyText
        ? undefined
        : parseJsonText(bodyText);

    return {
      endpointId: endpoint.id,
      label: endpoint.label,
      method,
      path,
      query,
      body,
      extraHeaders: headerPairsToObject(workbench.value.explorer.extraHeaders)
    };
  }

  function buildPreparedRequestFromDefinition(endpointId, overrides = {}) {
    const endpoint = endpointMap.get(endpointId);
    const context = buildContext({
      ...workbench.value,
      admin: admin.value
    });
    const pathParams = Object.fromEntries(
      (endpoint.pathParams || []).map((param) => [
        param,
        overrides.pathParams?.[param] ??
          resolveTemplateValue(endpoint.pathDefaults?.[param] ?? "", context)
      ])
    );
    const query = Object.fromEntries(
      (endpoint.queryParams || []).map((param) => [
        param,
        overrides.query?.[param] ??
          resolveTemplateValue(endpoint.queryDefaults?.[param] ?? "", context)
      ])
    );
    const body =
      overrides.body ??
      (endpoint.bodyTemplate
        ? removeEmptyValues(resolveTemplateValue(endpoint.bodyTemplate, context))
        : undefined);

    return {
      endpointId: endpoint.id,
      label: endpoint.label,
      method: overrides.method || endpoint.method,
      path: fillPathTemplate(endpoint.path, pathParams),
      query: Object.fromEntries(
        Object.entries(query).filter(([, value]) => value !== "")
      ),
      body,
      extraHeaders: overrides.extraHeaders || {}
    };
  }

  async function requestEndpoint(endpointId, overrides = {}, options = {}) {
    return sendPreparedRequest(
      buildPreparedRequestFromDefinition(endpointId, overrides),
      options
    );
  }

  async function runEndpoint(endpointId, overrides = {}, options = {}) {
    return withBusyState(endpointId, () =>
      requestEndpoint(endpointId, overrides, options)
    );
  }

  async function sendPreparedRequest(preparedRequest, options = {}) {
    const backendUnavailable = health.value.api.status === "offline";

    if (
      options.forceQueue ||
      (options.queueOnFailure && (!browserOnline.value || backendUnavailable))
    ) {
      const waitingFor =
        !browserOnline.value && backendUnavailable
          ? "browser network and backend"
          : !browserOnline.value
            ? "browser network"
            : "backend API";
      enqueuePreparedRequest(
        preparedRequest,
        "queued",
        online.value ? "Saved manually" : `Waiting for ${waitingFor}`
      );
      recordLocalEvent(
        preparedRequest.label,
        `Saved to the retry queue because ${waitingFor} is currently unavailable.`
      );
      return { ok: false, queued: true, status: 0, data: null };
    }

    try {
      const record = await executeNetworkRequest(preparedRequest);

      if (!record.ok) {
        recordLocalEvent(
          preparedRequest.label,
          `Request completed with HTTP ${record.status}.`,
          "error"
        );
      }
      return record;
    } catch (error) {
      const message = String(error.message || error);
      setApiUnavailable(`The backend could not be reached: ${message}`);

      if (options.queueOnFailure) {
        const safeToRetry =
          !browserOnline.value || health.value.api.status === "offline";
        enqueuePreparedRequest(
          preparedRequest,
          safeToRetry ? "queued" : "attention",
          message
        );
        recordLocalEvent(
          preparedRequest.label,
          safeToRetry
            ? "The request was queued and will retry once the backend is reachable again."
            : "The request was saved for manual review because the connection failed mid-flight and retrying automatically could duplicate a write.",
          safeToRetry ? "info" : "error"
        );
        return {
          ok: false,
          queued: safeToRetry,
          status: 0,
          data: null,
          error: message
        };
      }

      throw error;
    }
  }

  function updateQueueEntry(id, patch) {
    queue.value = queue.value.map((entry) =>
      entry.id === id ? { ...entry, ...patch } : entry
    );
  }

  async function replayPriceCapture(entry) {
    const purchaseRequest = {
      endpointId: "purchase-create",
      label: `${entry.label} create purchase`,
      method: "POST",
      path: "/purchases",
      query: {},
      body: entry.payload.purchase,
      extraHeaders: {}
    };
    const purchaseResult = await executeNetworkRequest(purchaseRequest);

    if (!purchaseResult.ok) {
      throw new Error(`Purchase creation failed with HTTP ${purchaseResult.status}.`);
    }

    const createdPurchaseId = workbench.value.lookup.purchaseId;

    if (!createdPurchaseId) {
      throw new Error("Purchase completed but no purchase ID was captured.");
    }

    const priceRequest = {
      endpointId: "price-create",
      label: `${entry.label} submit price`,
      method: "POST",
      path: "/prices",
      query: {},
      body: {
        ...entry.payload.price,
        purchaseId: createdPurchaseId
      },
      extraHeaders: {}
    };
    const priceResult = await executeNetworkRequest(priceRequest);

    if (!priceResult.ok) {
      throw new Error(`Price submission failed with HTTP ${priceResult.status}.`);
    }
  }

  async function flushQueue() {
    if (isFlushingQueue.value || !online.value) {
      return;
    }

    isFlushingQueue.value = true;

    try {
      for (const entry of [...queue.value]) {
        if (entry.status !== "queued") {
          continue;
        }

        try {
          if (entry.kind === "request") {
            const record = await executeNetworkRequest(entry.payload);

            if (!record.ok) {
              updateQueueEntry(entry.id, {
                status: "attention",
                attemptCount: entry.attemptCount + 1,
                updatedAt: new Date().toISOString(),
                lastError: `HTTP ${record.status}`
              });
              continue;
            }
          }

          if (entry.kind === "priceCapture") {
            await replayPriceCapture(entry);
          }

          queue.value = queue.value.filter((queuedEntry) => queuedEntry.id !== entry.id);
        } catch (error) {
          setApiUnavailable(String(error.message || error));
          const safeToRetry = !browserOnline.value || !apiOnline.value;

          updateQueueEntry(entry.id, {
            status: safeToRetry ? "queued" : "attention",
            attemptCount: entry.attemptCount + 1,
            updatedAt: new Date().toISOString(),
            lastError: String(error)
          });

          if (safeToRetry) {
            break;
          }
        }
      }
    } finally {
      isFlushingQueue.value = false;
    }
  }

  function removeQueueEntry(id) {
    queue.value = queue.value.filter((entry) => entry.id !== id);
  }

  function retryQueueEntry(id) {
    updateQueueEntry(id, {
      status: "queued",
      updatedAt: new Date().toISOString()
    });
    flushQueue();
  }

  function clearQueue() {
    queue.value = [];
  }

  function addExplorerHeader() {
    workbench.value.explorer.extraHeaders.push({ key: "", value: "" });
  }

  function removeExplorerHeader(index) {
    workbench.value.explorer.extraHeaders.splice(index, 1);
  }

  function resetExplorerDraft() {
    const draft = buildDraftFromEndpoint(
      selectedEndpoint.value,
      workbench.value,
      admin.value
    );
    workbench.value.explorer.pathParams = draft.pathParams;
    workbench.value.explorer.queryParams = draft.queryParams;
    workbench.value.explorer.bodyText = draft.bodyText;
    workbench.value.explorer.customMethod = draft.customMethod;
    workbench.value.explorer.customPath = draft.customPath;
    explorerError.value = "";
  }

  function handleCapturedCode(payload) {
    workbench.value.capture.lookupCode = payload.text;
  }

  async function refreshHealth(options = {}) {
    const preparedRequest = buildPreparedRequestFromDefinition("health");

    try {
      const result = await executeNetworkRequest(preparedRequest, {
        recordHistory: options.recordHistory
      });

      if (result.ok && isValidHealthEnvelope(result.data)) {
        applyHealthPayload(result.data);
        return result;
      }

      const detail = result.status
        ? `Health probe returned HTTP ${result.status}, but not a valid backend health payload.`
        : "Health probe did not return a valid backend response.";
      setApiUnavailable(detail);
      return {
        ...result,
        ok: false,
        error: detail
      };
    } catch (error) {
      setApiUnavailable(String(error.message || error));

      if (options.recordHistory) {
        appendHistory({
          id: createId(),
          label: preparedRequest.label,
          method: preparedRequest.method,
          url: buildApiUrl(
            workbench.value.workspace.apiBaseUrl,
            preparedRequest.path,
            preparedRequest.query
          ).toString(),
          status: 0,
          ok: false,
          startedAt: new Date().toISOString(),
          durationMs: 0,
          requestHeaders: {},
          requestBodyText: "",
          responseHeaders: {},
          responsePretty: String(error.message || error)
        });
      }

      return { ok: false, status: 0, data: null, error: String(error) };
    }
  }

  function fillDemoSession() {
    workbench.value.session.email = "alex@example.com";
    workbench.value.session.password = DEMO_PASSWORD;
    workbench.value.session.displayName = "Alex Pricewatch";
    workbench.value.session.primaryPhone = "+447700900123";
  }

  function fillAdminSession() {
    workbench.value.session.email = "admin@pricetracker.local";
    workbench.value.session.password = DEMO_PASSWORD;
    workbench.value.session.displayName = "PriceTracker Admin";
    workbench.value.session.primaryPhone = "+447700900999";
  }

  async function loadAdminOverviewInternal() {
    if (!isAdmin.value) {
      return null;
    }

    const result = await sendPreparedRequest(
      buildPreparedRequestFromDefinition("admin-overview")
    );

    if (result?.ok && result.data?.data) {
      admin.value.overview = result.data.data;
    }

    return result;
  }

  async function loadModerationPricesInternal() {
    if (!isAdmin.value) {
      return null;
    }

    admin.value.loadingModeration = true;

    try {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition("moderation-prices")
      );

      if (result?.ok && result.data?.data) {
        admin.value.moderationPrices = ensureArray(result.data.data);
      }

      return result;
    } finally {
      admin.value.loadingModeration = false;
    }
  }

  async function loadAdminUsersInternal() {
    if (!isAdmin.value) {
      return null;
    }

    admin.value.loadingUsers = true;

    try {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition("admin-users-list")
      );

      if (result?.ok && result.data?.data) {
        admin.value.users = ensureArray(result.data.data);
      }

      return result;
    } finally {
      admin.value.loadingUsers = false;
    }
  }

  async function loadAdminTableInternal(tableId = admin.value.selectedTableId) {
    if (!isAdmin.value) {
      return null;
    }

    admin.value.loadingTable = true;
    admin.value.error = "";
    admin.value.selectedTableId = tableId;
    admin.value.table =
      adminTables.value.find((entry) => entry.id === tableId) || admin.value.table;

    try {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition("admin-db-table", {
          pathParams: {
            tableId
          }
        })
      );

      if (result?.ok && result.data?.data) {
        admin.value.table = result.data.data.table;
        admin.value.rows = ensureArray(result.data.data.rows);
        admin.value.lookups =
          result.data.data.lookups && typeof result.data.data.lookups === "object"
            ? result.data.data.lookups
            : {};

        if (admin.value.selectedRecordId) {
          const matching = admin.value.rows.find(
            (row) => row.id === admin.value.selectedRecordId
          );

          if (matching) {
            admin.value.draftMode = "edit";
            admin.value.draftValues = cloneAdminDraft(admin.value.table, matching);
          } else {
            admin.value.selectedRecordId = "";
            admin.value.draftMode = "create";
            admin.value.draftValues = cloneAdminDraft(admin.value.table);
          }
        } else {
          admin.value.draftMode = "create";
          admin.value.draftValues = cloneAdminDraft(admin.value.table);
        }
      } else {
        admin.value.rows = [];
        admin.value.lookups = {};
        admin.value.error = resultErrorMessage(
          result,
          `Unable to load the ${String(tableId || "admin")} table.`
        );
      }

      return result;
    } catch (error) {
      admin.value.rows = [];
      admin.value.lookups = {};
      admin.value.error = String(error.message || error);
      return null;
    } finally {
      admin.value.loadingTable = false;
    }
  }

  async function loadAdminBootstrap() {
    if (!isAdmin.value) {
      return;
    }

    await Promise.all([
      loadAdminOverviewInternal(),
      loadAdminUsersInternal(),
      loadModerationPricesInternal(),
      loadAdminTableInternal(admin.value.selectedTableId)
    ]);
  }

  watch(online, (isNowOnline) => {
    if (
      isNowOnline &&
      workbench.value.session.accountId &&
      workbench.value.session.accessToken
    ) {
      flushQueue();
    }
  });

  watch(isAdmin, async (isNowAdmin) => {
    if (!isNowAdmin) {
      workbench.value.ui.activeView = "public";
      admin.value = createInitialAdminState();
    }
  });

  onMounted(async () => {
    await refreshHealth({ recordHistory: false });

    if (workbench.value.session.accountId && !workbench.value.session.accessToken) {
      clearClientState();
    }

    let sessionValidated = !workbench.value.session.accountId;

    if (workbench.value.session.accountId && workbench.value.session.accessToken) {
      try {
        const refreshResult = await requestEndpoint(
          "auth-refresh",
          {},
          { recordHistory: false }
        );
        sessionValidated = Boolean(refreshResult?.ok);
      } catch (_error) {
        sessionValidated = false;
      }
    }

    if (
      sessionValidated &&
      online.value &&
      workbench.value.session.accountId &&
      queue.value.some((entry) => entry.status === "queued")
    ) {
      flushQueue();
    }

    healthTimer.value = window.setInterval(() => {
      refreshHealth({ recordHistory: false });
    }, HEALTH_POLL_INTERVAL_MS);
  });

  onBeforeUnmount(() => {
    if (healthTimer.value) {
      window.clearInterval(healthTimer.value);
    }
  });

  async function runHealthCheck() {
    return withBusyState("health", () =>
      refreshHealth({ recordHistory: true })
    );
  }

  async function loginSession() {
    return withBusyState("login", async () => {
      const result = await requestEndpoint("auth-login");
      if (result?.ok) {
        await refreshHealth({ recordHistory: false });
      }

      return result;
    });
  }

  async function loginDemo() {
    fillDemoSession();
    return loginSession();
  }

  async function loginAdmin() {
    fillAdminSession();
    return loginSession();
  }

  async function refreshSession() {
    return withBusyState("refresh", () => requestEndpoint("auth-refresh"));
  }

  async function loadProfile() {
    return withBusyState("profile", () => requestEndpoint("me-get"));
  }

  async function logoutSession() {
    return withBusyState("logout", async () => {
      let result;

      try {
        result = await requestEndpoint("auth-logout", {}, { recordHistory: false });
      } catch (error) {
        result = {
          ok: false,
          status: 0,
          data: null,
          error: String(error.message || error)
        };
      }

      clearClientState();
      return result;
    });
  }

  function openPublicView() {
    workbench.value.ui.activeView = "public";
  }

  async function openAdminView() {
    workbench.value.ui.activeView = "admin";

    if (isAdmin.value) {
      await loadAdminBootstrap();
    }
  }

  async function listShops() {
    return withBusyState("shops", () =>
      sendPreparedRequest(buildPreparedRequestFromDefinition("shops-list"))
    );
  }

  async function listItems() {
    return withBusyState("items", () =>
      sendPreparedRequest(buildPreparedRequestFromDefinition("items-list"))
    );
  }

  async function lookupCapturedCode() {
    if (!workbench.value.lookup.shopId || !workbench.value.capture.lookupCode) {
      recordLocalEvent(
        "Code lookup",
        "Set both a shop ID and a scanned or typed code before looking up a product.",
        "error"
      );
      return;
    }

    return withBusyState("lookup-code", () =>
      sendPreparedRequest(
        buildPreparedRequestFromDefinition("product-code-lookup", {
          pathParams: {
            shopId: workbench.value.lookup.shopId,
            code: workbench.value.capture.lookupCode
          }
        })
      )
    );
  }

  async function loadVariantDetail() {
    if (!workbench.value.lookup.variantId) {
      recordLocalEvent(
        "Variant detail",
        "Set a variant ID or resolve one from the scanner before loading details.",
        "error"
      );
      return;
    }

    return withBusyState("variant-detail", () =>
      sendPreparedRequest(buildPreparedRequestFromDefinition("variant-detail"))
    );
  }

  async function compareVariant() {
    if (!workbench.value.lookup.variantId) {
      recordLocalEvent(
        "Compare variant",
        "Set a variant ID before running a comparison.",
        "error"
      );
      return;
    }

    return withBusyState("compare", () =>
      sendPreparedRequest(buildPreparedRequestFromDefinition("compare-body"))
    );
  }

  async function createUploadIntent() {
    const fileUpload = workbench.value.submission.fileUpload;
    const size = Number(fileUpload.size);

    if (!fileUpload.filename || !fileUpload.contentType || !Number.isFinite(size)) {
      recordLocalEvent(
        "File upload intent",
        "Set a filename, content type, and numeric size before creating an upload intent.",
        "error"
      );
      return;
    }

    return withBusyState("upload-intent", () =>
      sendPreparedRequest(
        buildPreparedRequestFromDefinition("file-upload-intent", {
          body: removeEmptyValues({
            filename: fileUpload.filename,
            contentType: fileUpload.contentType,
            size,
            purpose: fileUpload.purpose,
            checksumSha256: fileUpload.checksumSha256
          })
        }),
        { queueOnFailure: true }
      )
    );
  }

  async function completeUploadIntent() {
    if (!workbench.value.lookup.fileId) {
      recordLocalEvent(
        "Complete upload",
        "Create an upload intent first so a file ID is available.",
        "error"
      );
      return;
    }

    return withBusyState("upload-complete", () =>
      sendPreparedRequest(buildPreparedRequestFromDefinition("file-upload-complete"))
    );
  }

  function buildPurchaseBody() {
    return removeEmptyValues({
      shopId: workbench.value.lookup.shopId,
      purchaseTime: toIsoString(workbench.value.submission.purchaseTime),
      attachmentFileIds: parseAttachmentFileIds(),
      notes: workbench.value.submission.purchaseNotes
    });
  }

  function buildPriceBody(purchaseIdOverride = workbench.value.lookup.purchaseId) {
    return removeEmptyValues({
      itemVariantId: workbench.value.lookup.variantId,
      purchaseId: purchaseIdOverride,
      originalAmount: workbench.value.submission.originalAmount,
      originalCurrency: workbench.value.submission.originalCurrency,
      discountAmount: workbench.value.submission.discountAmount,
      discountCurrency: workbench.value.submission.discountCurrency,
      discountTypeId: workbench.value.submission.discountTypeId,
      recordedAt: toIsoString(workbench.value.submission.recordedAt),
      notes: workbench.value.submission.priceNotes
    });
  }

  async function createPurchase() {
    if (!workbench.value.lookup.shopId) {
      recordLocalEvent(
        "Create purchase",
        "Set a shop ID before creating a purchase.",
        "error"
      );
      return;
    }

    return withBusyState("purchase", () =>
      sendPreparedRequest(
        buildPreparedRequestFromDefinition("purchase-create", {
          body: buildPurchaseBody()
        }),
        { queueOnFailure: true }
      )
    );
  }

  async function submitPrice() {
    if (!workbench.value.lookup.variantId) {
      recordLocalEvent(
        "Submit price",
        "Set a variant ID before submitting a price.",
        "error"
      );
      return;
    }

    if (!workbench.value.lookup.purchaseId) {
      recordLocalEvent(
        "Submit price",
        "Create or select a purchase first, or save a full capture for retry if you are offline.",
        "error"
      );
      return;
    }

    return withBusyState("price", () =>
      sendPreparedRequest(
        buildPreparedRequestFromDefinition("price-create", {
          body: buildPriceBody()
        }),
        { queueOnFailure: true }
      )
    );
  }

  function saveFullCaptureForRetry() {
    if (!workbench.value.lookup.shopId || !workbench.value.lookup.variantId) {
      recordLocalEvent(
        "Save full capture",
        "Set both a shop ID and a variant ID before saving a complete capture workflow.",
        "error"
      );
      return;
    }

    addQueueEntry({
      id: createId(),
      kind: "priceCapture",
      status: "queued",
      label: `Saved capture for ${workbench.value.lookup.variantId}`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      attemptCount: 0,
      lastError: "",
      payload: {
        purchase: buildPurchaseBody(),
        price: buildPriceBody("")
      }
    });

    recordLocalEvent(
      "Save full capture",
      "Saved a purchase-and-price workflow to the retry queue."
    );
  }

  function normalizeAdminPayloadValues(table, values) {
    const normalized = {};

    ensureArray(table?.columns).forEach((column) => {
      if (!column.mutable) {
        return;
      }

      const raw = values[column.key];

      if (column.input === "boolean") {
        normalized[column.key] = Boolean(raw);
        return;
      }

      if (raw === undefined || raw === null || raw === "") {
        normalized[column.key] = null;
        return;
      }

      normalized[column.key] = raw;
    });

    return normalized;
  }

  function startCreateAdminRecord() {
    if (!adminTable.value) {
      return;
    }

    admin.value.selectedRecordId = "";
    admin.value.draftMode = "create";
    admin.value.draftValues = cloneAdminDraft(adminTable.value);
    admin.value.error = "";
  }

  function selectAdminRow(row) {
    admin.value.selectedRecordId = row.id;
    admin.value.draftMode = "edit";
    admin.value.draftValues = cloneAdminDraft(adminTable.value, row);
    admin.value.error = "";
  }

  async function selectAdminTable(tableId) {
    admin.value.selectedTableId = tableId;
    admin.value.selectedRecordId = "";
    await loadAdminTableInternal(tableId);
  }

  async function refreshAdminTable() {
    await loadAdminTableInternal(admin.value.selectedTableId);
  }

  async function saveAdminRecordDraft({
    tableId = admin.value.selectedTableId,
    mode = admin.value.draftMode,
    recordId = admin.value.selectedRecordId,
    values = admin.value.draftValues
  } = {}) {
    const table =
      admin.value.table?.id === tableId
        ? admin.value.table
        : adminTables.value.find((entry) => entry.id === tableId) || adminTable.value;

    if (!isAdmin.value || !table) {
      return;
    }

    admin.value.savingRecord = true;
    admin.value.error = "";

    try {
      const payload = {
        values: normalizeAdminPayloadValues(table, values)
      };
      const endpointId =
        mode === "create"
          ? "admin-db-table-create"
          : "admin-db-record-update";
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition(endpointId, {
          pathParams:
            mode === "create"
              ? { tableId }
              : {
                  tableId,
                  recordId
                },
          body: payload
        })
      );

      if (result?.ok && result.data?.data) {
        const savedId = result.data.data.id || recordId;
        await Promise.all([
          loadAdminOverviewInternal(),
          loadAdminTableInternal(tableId)
        ]);

        if (savedId) {
          const matching = admin.value.rows.find((row) => row.id === savedId);
          if (matching) {
            selectAdminRow(matching);
          }
        }
      }

      return result;
    } catch (error) {
      admin.value.error = String(error.message || error);
      return null;
    } finally {
      admin.value.savingRecord = false;
    }
  }

  async function saveAdminRecord() {
    return saveAdminRecordDraft();
  }

  async function deleteAdminRecord({ tableId = admin.value.selectedTableId, recordId } = {}) {
    if (!isAdmin.value || !tableId || !recordId) {
      return null;
    }

    admin.value.deletingRecord = true;
    admin.value.error = "";

    try {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition("admin-db-record-delete", {
          pathParams: { tableId, recordId }
        })
      );

      if (result?.ok) {
        if (admin.value.selectedRecordId === recordId) {
          admin.value.selectedRecordId = "";
          admin.value.draftMode = "create";
          admin.value.draftValues = cloneAdminDraft(adminTable.value);
        }

        await Promise.all([loadAdminOverviewInternal(), loadAdminTableInternal(tableId)]);
      }

      return result;
    } finally {
      admin.value.deletingRecord = false;
    }
  }

  async function approveAdminRecord({
    tableId = admin.value.selectedTableId,
    recordId,
    refresh = true
  } = {}) {
    if (!isAdmin.value || !tableId || !recordId) {
      return null;
    }

    admin.value.approvingRecord = true;
    admin.value.error = "";

    try {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition("admin-db-record-approve", {
          pathParams: { tableId, recordId }
        })
      );

      if (result?.ok && refresh) {
        await Promise.all([loadAdminOverviewInternal(), loadAdminTableInternal(tableId)]);
      }

      return result;
    } finally {
      admin.value.approvingRecord = false;
    }
  }

  async function loadModerationPrices() {
    return withBusyState("moderation", () => loadModerationPricesInternal());
  }

  async function loadAdminUsers() {
    return withBusyState("admin-users", () => loadAdminUsersInternal());
  }

  async function createAdminUser(payload) {
    return withBusyState("admin-user-create", async () => {
      admin.value.savingUsers = true;

      try {
        const result = await sendPreparedRequest(
          buildPreparedRequestFromDefinition("admin-users-create", {
            body: removeEmptyValues(payload)
          })
        );

        if (result?.ok) {
          await Promise.all([loadAdminOverviewInternal(), loadAdminUsersInternal()]);
        }

        return result;
      } finally {
        admin.value.savingUsers = false;
      }
    });
  }

  async function updateAdminUser(accountId, payload) {
    return withBusyState("admin-user-update", async () => {
      admin.value.savingUsers = true;

      try {
        const result = await sendPreparedRequest(
          buildPreparedRequestFromDefinition("admin-users-update", {
            pathParams: { accountId },
            body: payload
          })
        );

        if (result?.ok) {
          await Promise.all([loadAdminOverviewInternal(), loadAdminUsersInternal()]);
        }

        return result;
      } finally {
        admin.value.savingUsers = false;
      }
    });
  }

  async function applyAdminUserBulkAction(payload) {
    return withBusyState("admin-user-bulk", async () => {
      admin.value.savingUsers = true;

      try {
        const result = await sendPreparedRequest(
          buildPreparedRequestFromDefinition("admin-users-bulk", {
            body: removeEmptyValues(payload)
          })
        );

        if (result?.ok) {
          await Promise.all([loadAdminOverviewInternal(), loadAdminUsersInternal()]);
        }

        return result;
      } finally {
        admin.value.savingUsers = false;
      }
    });
  }

  async function moderatePrice(endpointId, priceId, reason) {
    return withBusyState(endpointId, async () => {
      const result = await sendPreparedRequest(
        buildPreparedRequestFromDefinition(endpointId, {
          pathParams: {
            priceId
          },
          body: {
            reason
          }
        })
      );

      if (result?.ok) {
        await loadModerationPricesInternal();
        await loadAdminOverviewInternal();
      }

      return result;
    });
  }

  async function approveModerationPrice(priceId) {
    return moderatePrice("moderation-verify", priceId, "Approved from admin dashboard");
  }

  async function rejectModerationPrice(priceId) {
    return moderatePrice("moderation-reject", priceId, "Rejected from admin dashboard");
  }

  async function sendExplorerRequest() {
    return withBusyState("explorer", async () => {
      try {
        explorerError.value = "";
        return await sendPreparedRequest(prepareExplorerRequest(), {
          queueOnFailure: workbench.value.explorer.queueOnFailure
        });
      } catch (error) {
        explorerError.value = String(error.message || error);
        return null;
      }
    });
  }

  function queueExplorerRequest() {
    try {
      explorerError.value = "";
      const prepared = prepareExplorerRequest();
      enqueuePreparedRequest(prepared, "queued", "Saved manually from the explorer");
      recordLocalEvent(
        prepared.label,
        "Saved the explorer request to the retry queue."
      );
    } catch (error) {
      explorerError.value = String(error.message || error);
    }
  }

  return {
    workbench,
    queue,
    history,
    health,
    admin,
    browserOnline,
    apiOnline,
    databaseOnline,
    online,
    isAdmin,
    isBusy,
    isFlushingQueue,
    debugToolsEnabled,
    selectedEndpoint,
    endpointGroups,
    filteredEndpoints,
    summaryCards,
    adminSummaryCards,
    statusLights,
    adminTables,
    adminTable,
    explorerError,
    requestMethodOptions,
    requestEndpoint,
    runEndpoint,
    addExplorerHeader,
    removeExplorerHeader,
    resetExplorerDraft,
    handleCapturedCode,
    runHealthCheck,
    fillDemoSession,
    fillAdminSession,
    loginSession,
    loginDemo,
    loginAdmin,
    refreshSession,
    loadProfile,
    logoutSession,
    openPublicView,
    openAdminView,
    listShops,
    listItems,
    lookupCapturedCode,
    loadVariantDetail,
    compareVariant,
    createUploadIntent,
    completeUploadIntent,
    createPurchase,
    submitPrice,
    saveFullCaptureForRetry,
    flushQueue,
    removeQueueEntry,
    retryQueueEntry,
    clearQueue,
    selectAdminTable,
    selectAdminRow,
    startCreateAdminRecord,
    refreshAdminTable,
    saveAdminRecord,
    saveAdminRecordDraft,
    deleteAdminRecord,
    approveAdminRecord,
    loadModerationPrices,
    loadAdminUsers,
    loadAdminBootstrap,
    createAdminUser,
    updateAdminUser,
    applyAdminUserBulkAction,
    approveModerationPrice,
    rejectModerationPrice,
    sendExplorerRequest,
    queueExplorerRequest
  };
}
