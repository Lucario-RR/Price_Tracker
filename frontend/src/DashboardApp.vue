<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";

import { useWorkbench } from "./composables/useWorkbench";
import {
  listCameraDevices,
  scanImageFile,
  startLiveScanner,
  stopLiveScanner
} from "./lib/scanner";

const FEEDBACK_AUTO_DISMISS_MS = 3200;

const model = reactive(useWorkbench());
const videoRef = ref(null);
const cameraDevices = ref([]);
const isScanning = ref(false);
const scanError = ref("");
const loadingCameras = ref(false);
const lastFingerprint = ref("");

const ui = reactive({
  page: "overview",
  authMode: "login",
  search: "",
  sidebarCollapsed: false,
  alertAmount: "1.50",
  alertCurrency: "GBP",
  alertEnabled: true,
  showExplorer: false,
  feedback: {
    visible: false,
    tone: "info",
    message: "",
    pending: false
  }
});

const data = reactive({
  shops: [],
  items: [],
  variants: [],
  profile: null,
  purchases: [],
  prices: [],
  watchlist: [],
  alerts: [],
  settings: [],
  loadingPublic: false,
  loadingUser: false,
  loadingSettings: false,
  savingSetting: ""
});

const profileForm = reactive({
  displayName: "",
  defaultCurrency: "GBP",
  locale: "en-GB",
  timezoneName: "Europe/London",
  profileBio: ""
});

let feedbackDismissTimer = null;

const isLoggedIn = computed(() => Boolean(model.workbench.session.accountId));
const accountName = computed(
  () => model.workbench.session.displayName || model.workbench.session.email || "Guest"
);
const accountInitials = computed(() => {
  const source = accountName.value.replace(/@.*$/, "").trim();
  if (!source) return "PT";

  const parts = source.split(/[\s._-]+/).filter(Boolean);
  return parts.length === 1
    ? parts[0].slice(0, 2).toUpperCase()
    : `${parts[0][0] || ""}${parts[1][0] || ""}`.toUpperCase();
});
const accountRole = computed(() => {
  if (!isLoggedIn.value) return "Guest";
  return model.isAdmin ? "Admin" : "User";
});
const selectedShop = computed(
  () => data.shops.find((shop) => shop.id === model.workbench.lookup.shopId) || null
);
const selectedItem = computed(
  () => data.items.find((item) => item.id === model.workbench.lookup.itemId) || null
);
const settingMap = computed(() =>
  Object.fromEntries(data.settings.map((setting) => [setting.key, setting.value]))
);
const maintenanceMode = computed(() => settingMap.value["system.maintenanceMode"] === true);
const hiddenModulesEnabled = computed(
  () => settingMap.value["debug.hiddenModulesEnabled"] !== false
);
const canUseProtectedTools = computed(() => isLoggedIn.value && !maintenanceMode.value);

const pageMeta = computed(() => {
  const meta = {
    overview: ["Overview", "i-lucide-layout-dashboard", "System health, catalog readiness, and latest activity."],
    capture: ["Capture", "i-lucide-scan-barcode", "Scan, select, and submit prices with receipt context."],
    activity: ["Activity", "i-lucide-activity", "Queue recovery, watchlists, alerts, and API responses."],
    account: ["Account", "i-lucide-user-round-cog", "Profile, identity, and local session details."],
    admin: ["Admin", "i-lucide-shield-check", "Operations summary, moderation, settings, and debug tools."],
    auth: [ui.authMode === "login" ? "Login" : "Register", "i-lucide-log-in", "Access the dashboard."]
  };
  const [title, icon, description] = meta[ui.page] || meta.overview;
  return { title, icon, description };
});

const navigationGroups = computed(() => [
  [
    { label: "Dashboard", type: "label" },
    {
      label: "Overview",
      icon: "i-lucide-layout-dashboard",
      active: ui.page === "overview",
      onSelect: () => openPage("overview")
    },
    {
      label: "Capture",
      icon: "i-lucide-scan-barcode",
      active: ui.page === "capture",
      badge: model.workbench.capture.recentCaptures.length || undefined,
      onSelect: () => openPage("capture")
    },
    {
      label: "Activity",
      icon: "i-lucide-activity",
      active: ui.page === "activity",
      badge: model.queue.length || undefined,
      onSelect: () => openPage("activity")
    },
    {
      label: "Account",
      icon: "i-lucide-user-round-cog",
      active: ui.page === "account",
      onSelect: () => openPage(isLoggedIn.value ? "account" : "auth")
    }
  ],
  [
    { label: "Operations", type: "label" },
    {
      label: "Admin",
      icon: "i-lucide-shield-check",
      active: ui.page === "admin",
      badge: model.isAdmin ? "Live" : undefined,
      disabled: !model.isAdmin,
      onSelect: () => openPage("admin")
    }
  ]
]);

const statusCards = computed(() =>
  model.statusLights.map((light) => ({ ...light, color: toneColor(light.tone) }))
);
const summaryCards = computed(() => [
  ...model.summaryCards.map((card) => ({
    label: card.label,
    value: card.value,
    icon: summaryIcon(card.tone),
    color: toneColor(card.tone)
  })),
  {
    label: "Role",
    value: accountRole.value,
    icon: "i-lucide-badge-check",
    color: model.isAdmin ? "primary" : "neutral"
  }
]);
const authModeTabs = computed(() => [
  { label: "Login", value: "login", icon: "i-lucide-log-in" },
  { label: "Register", value: "register", icon: "i-lucide-user-plus" },
  { label: "Admin Setup", value: "admin", icon: "i-lucide-shield-plus" }
]);
const dashboardTabs = computed(() =>
  [
    { label: "Overview", value: "overview", icon: "i-lucide-layout-dashboard" },
    { label: "Capture", value: "capture", icon: "i-lucide-scan-barcode" },
    { label: "Activity", value: "activity", icon: "i-lucide-activity" },
    { label: "Account", value: "account", icon: "i-lucide-user" },
    model.isAdmin ? { label: "Admin", value: "admin", icon: "i-lucide-shield" } : null
  ].filter(Boolean)
);
const historyRows = computed(() => model.history.slice(0, 8));
const latestResponse = computed(() => model.history[0] || null);
const queuePreview = computed(() => model.queue.slice(0, 5));
const adminSettingsPreview = computed(() => data.settings.slice(0, 4));
const adminModerationPreview = computed(() => model.admin.moderationPrices.slice(0, 4));

function toneColor(tone) {
  return {
    good: "success",
    warn: "warning",
    info: "info",
    accent: "primary",
    quiet: "neutral",
    danger: "error"
  }[tone] || "neutral";
}

function summaryIcon(tone) {
  return {
    good: "i-lucide-check-circle",
    warn: "i-lucide-alert-triangle",
    info: "i-lucide-info",
    accent: "i-lucide-sparkles",
    quiet: "i-lucide-circle"
  }[tone] || "i-lucide-chart-no-axes-column";
}

function openPage(page) {
  if (page === "capture") void loadPublicData();
  if (page === "activity" || page === "account") void refreshDashboard();
  if (page === "admin") {
    if (!model.isAdmin) return;
    void refreshAdminPage();
  }
  ui.page = page;
}

function formatValue(value) {
  if (value === undefined || value === null || value === "") return "Not set";
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

function clearFeedback() {
  if (feedbackDismissTimer) {
    window.clearTimeout(feedbackDismissTimer);
    feedbackDismissTimer = null;
  }
  ui.feedback.visible = false;
  ui.feedback.tone = "info";
  ui.feedback.message = "";
  ui.feedback.pending = false;
}

function showFeedback(tone, message, pending = false) {
  if (feedbackDismissTimer) {
    window.clearTimeout(feedbackDismissTimer);
    feedbackDismissTimer = null;
  }
  ui.feedback.visible = Boolean(message);
  ui.feedback.tone = tone;
  ui.feedback.message = message;
  ui.feedback.pending = pending;
  if (typeof window !== "undefined" && tone === "success" && !pending && message) {
    const expectedMessage = message;
    feedbackDismissTimer = window.setTimeout(() => {
      if (ui.feedback.visible && !ui.feedback.pending && ui.feedback.message === expectedMessage) {
        clearFeedback();
      }
    }, FEEDBACK_AUTO_DISMISS_MS);
  }
}

function resultErrorMessage(result, fallbackMessage) {
  if (result?.data?.error?.message) return result.data.error.message;
  if (result?.error) return result.error;
  if (result?.status) return `${fallbackMessage} (HTTP ${result.status}).`;
  return fallbackMessage;
}

async function runServerAction({ pendingMessage, successMessage, queuedMessage, errorMessage, task }) {
  showFeedback("info", pendingMessage, true);
  try {
    const result = await task();
    if (result?.ok) {
      showFeedback("success", successMessage, false);
      return result;
    }
    if (result?.queued) {
      showFeedback("warning", queuedMessage || "Request saved locally and will retry.", false);
      return result;
    }
    showFeedback("error", resultErrorMessage(result, errorMessage), false);
    return result;
  } catch (error) {
    showFeedback("error", String(error.message || error), false);
    return null;
  }
}

function applyProfileToForm(profile) {
  profileForm.displayName = profile?.displayName || "";
  profileForm.defaultCurrency = profile?.defaultCurrency || "GBP";
  profileForm.locale = profile?.locale || "en-GB";
  profileForm.timezoneName = profile?.timezoneName || "Europe/London";
  profileForm.profileBio = profile?.profileBio || "";
}

function resetProtectedData() {
  data.profile = null;
  data.purchases = [];
  data.prices = [];
  data.watchlist = [];
  data.alerts = [];
  data.settings = [];
  data.savingSetting = "";
  applyProfileToForm(null);
}

async function loadPublicData() {
  data.loadingPublic = true;
  try {
    const [shopsResult, itemsResult] = await Promise.all([
      model.requestEndpoint("shops-list"),
      model.requestEndpoint("items-list", { query: { q: ui.search.trim() } })
    ]);
    if (shopsResult?.ok && Array.isArray(shopsResult.data?.data)) {
      data.shops = shopsResult.data.data;
    }
    if (itemsResult?.ok && Array.isArray(itemsResult.data?.data)) {
      data.items = itemsResult.data.data;
    }
  } finally {
    data.loadingPublic = false;
  }
}

async function loadVariants(itemId = model.workbench.lookup.itemId) {
  if (!itemId) {
    data.variants = [];
    model.workbench.lookup.variantId = "";
    return;
  }
  const result = await model.requestEndpoint("item-variants", {
    pathParams: { itemId }
  });
  if (result?.ok && Array.isArray(result.data?.data)) {
    data.variants = result.data.data;
    if (!data.variants.some((variant) => variant.id === model.workbench.lookup.variantId)) {
      model.workbench.lookup.variantId = data.variants[0]?.id || "";
    }
  }
}

async function loadUserData() {
  if (!isLoggedIn.value) {
    resetProtectedData();
    return;
  }
  data.loadingUser = true;
  try {
    const [profile, purchases, prices, watchlist, alerts] = await Promise.all([
      model.requestEndpoint("me-get"),
      model.requestEndpoint("purchase-list"),
      model.requestEndpoint("price-list"),
      model.requestEndpoint("watchlist-list"),
      model.requestEndpoint("alerts-list")
    ]);
    if (profile?.ok && profile.data?.data) {
      data.profile = profile.data.data;
      applyProfileToForm(profile.data.data);
    }
    if (purchases?.ok && Array.isArray(purchases.data?.data)) data.purchases = purchases.data.data;
    if (prices?.ok && Array.isArray(prices.data?.data)) data.prices = prices.data.data;
    if (watchlist?.ok && Array.isArray(watchlist.data?.data)) data.watchlist = watchlist.data.data;
    if (alerts?.ok && Array.isArray(alerts.data?.data)) data.alerts = alerts.data.data;
  } finally {
    data.loadingUser = false;
  }
}

async function loadSettings() {
  if (!model.isAdmin) {
    data.settings = [];
    return;
  }
  data.loadingSettings = true;
  try {
    const result = await model.requestEndpoint("admin-settings-list");
    if (result?.ok && Array.isArray(result.data?.data)) {
      data.settings = result.data.data;
    }
  } finally {
    data.loadingSettings = false;
  }
}

async function refreshDashboard() {
  await loadPublicData();
  if (model.workbench.lookup.itemId) await loadVariants(model.workbench.lookup.itemId);
  if (isLoggedIn.value) await loadUserData();
}

async function refreshAdminPage() {
  if (!model.isAdmin) return;
  await Promise.all([refreshDashboard(), model.loadAdminBootstrap(), loadSettings()]);
}

async function afterAuth() {
  await refreshDashboard();
  openPage("overview");
}

async function login() {
  return runServerAction({
    pendingMessage: "Signing you in...",
    successMessage: "Signed in successfully.",
    errorMessage: "Unable to sign in.",
    task: async () => {
      const response = await model.loginSession();
      if (response?.ok) await afterAuth();
      return response;
    }
  });
}

async function registerUser() {
  return runServerAction({
    pendingMessage: "Creating your account...",
    successMessage: "Account created and signed in.",
    errorMessage: "Unable to create the account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register");
      if (response?.ok) await afterAuth();
      return response;
    }
  });
}

async function registerAdmin() {
  return runServerAction({
    pendingMessage: "Creating the admin account...",
    successMessage: "Admin account created and signed in.",
    errorMessage: "Unable to create the admin account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register-admin");
      if (response?.ok) {
        await afterAuth();
        openPage("admin");
      }
      return response;
    }
  });
}

async function logout() {
  showFeedback("info", "Signing out and clearing local app data...", true);
  const result = await model.logoutSession();
  resetProtectedData();
  ui.showExplorer = false;
  openPage("overview");
  if (result?.ok) {
    showFeedback("success", "Signed out. Local cached data was cleared.", false);
  } else {
    showFeedback("warning", `Local data was cleared, but sign-out could not be confirmed. ${resultErrorMessage(result, "Refresh if you still appear signed in.")}`, false);
  }
}

async function saveProfile() {
  const result = await runServerAction({
    pendingMessage: "Saving your profile...",
    successMessage: "Profile updated.",
    errorMessage: "Unable to save the profile.",
    task: () =>
      model.runEndpoint("me-update", {
        body: {
          displayName: profileForm.displayName,
          defaultCurrency: String(profileForm.defaultCurrency || "").trim().toUpperCase(),
          locale: profileForm.locale,
          timezoneName: profileForm.timezoneName,
          profileBio: profileForm.profileBio
        }
      })
  });
  if (result?.ok) {
    model.workbench.session.displayName = profileForm.displayName;
    await loadUserData();
  }
}

async function addWatchlist() {
  if (!model.workbench.lookup.variantId) {
    showFeedback("warning", "Select a variant before adding it to your watchlist.", false);
    return;
  }
  const result = await runServerAction({
    pendingMessage: "Adding this variant to your watchlist...",
    successMessage: "Variant added to your watchlist.",
    errorMessage: "Unable to add the watchlist item.",
    task: () =>
      model.runEndpoint("watchlist-create", {
        body: { itemVariantId: model.workbench.lookup.variantId }
      })
  });
  if (result?.ok) await loadUserData();
}

async function addAlert() {
  if (!model.workbench.lookup.variantId) {
    showFeedback("warning", "Select a variant before creating an alert.", false);
    return;
  }
  const result = await runServerAction({
    pendingMessage: "Creating your alert...",
    successMessage: "Alert created.",
    errorMessage: "Unable to create the alert.",
    task: () =>
      model.runEndpoint("alert-create", {
        body: {
          itemVariantId: model.workbench.lookup.variantId,
          targetFinalAmount: ui.alertAmount,
          currency: ui.alertCurrency,
          isEnabled: ui.alertEnabled
        }
      })
  });
  if (result?.ok) await loadUserData();
}

async function saveSetting(setting, value) {
  data.savingSetting = setting.key;
  try {
    const result = await runServerAction({
      pendingMessage: `Saving ${setting.key}...`,
      successMessage: `${setting.key} updated.`,
      errorMessage: `Unable to update ${setting.key}.`,
      task: () =>
        model.runEndpoint("admin-settings-update", {
          pathParams: { settingKey: setting.key },
          body: { value }
        })
    });
    if (result?.ok && result.data?.data) {
      data.settings = data.settings.map((entry) =>
        entry.key === setting.key ? result.data.data : entry
      );
    }
  } finally {
    data.savingSetting = "";
  }
}

async function loadCameras() {
  loadingCameras.value = true;
  scanError.value = "";
  try {
    cameraDevices.value = await listCameraDevices();
    if (!model.workbench.capture.selectedDeviceId && cameraDevices.value[0]?.deviceId) {
      model.workbench.capture.selectedDeviceId = cameraDevices.value[0].deviceId;
    }
  } catch (error) {
    scanError.value = String(error.message || error);
  } finally {
    loadingCameras.value = false;
  }
}

function registerCapture(payload) {
  const fingerprint = `${payload.source}:${payload.format}:${payload.text}`;
  if (lastFingerprint.value === fingerprint) return;

  lastFingerprint.value = fingerprint;
  model.workbench.capture.lookupCode = payload.text;
  model.workbench.capture.manualCode = payload.text;
  model.workbench.capture.recentCaptures = [
    payload,
    ...model.workbench.capture.recentCaptures.filter(
      (entry) => entry.text !== payload.text || entry.source !== payload.source
    )
  ].slice(0, 6);
  model.handleCapturedCode(payload);
}

async function startScanning() {
  if (maintenanceMode.value) return;
  scanError.value = "";
  await nextTick();
  if (!videoRef.value) return;

  try {
    await startLiveScanner({
      deviceId: model.workbench.capture.selectedDeviceId,
      videoElement: videoRef.value,
      onResult: registerCapture,
      onError: (error) => {
        scanError.value = String(error.message || error);
      }
    });
    isScanning.value = true;
  } catch (error) {
    isScanning.value = false;
    scanError.value = String(error.message || error);
  }
}

function stopScanning() {
  stopLiveScanner();
  isScanning.value = false;
}

async function handleImageSelection(event) {
  const [file] = event.target.files || [];
  if (!file) return;

  scanError.value = "";
  try {
    registerCapture(await scanImageFile(file));
  } catch (error) {
    scanError.value = String(error.message || error);
  } finally {
    event.target.value = "";
  }
}

function useManualCode() {
  const value = model.workbench.capture.manualCode?.trim();
  if (!value) {
    scanError.value = "Type a code manually before saving it.";
    return;
  }

  scanError.value = "";
  registerCapture({
    text: value,
    format: "MANUAL",
    source: "manual-entry",
    capturedAt: new Date().toISOString()
  });
}

function runAuthAction() {
  if (ui.authMode === "login") return login();
  return ui.authMode === "admin" ? registerAdmin() : registerUser();
}

watch(
  () => model.workbench.lookup.itemId,
  (itemId) => {
    void loadVariants(itemId);
  }
);

watch(isLoggedIn, (loggedIn, wasLoggedIn) => {
  if (loggedIn) return;
  resetProtectedData();
  if (wasLoggedIn) {
    showFeedback(
      "warning",
      "Your session is no longer valid. Protected local data was cleared.",
      false
    );
  }
});

watch(
  () => model.isAdmin,
  (isAdminNow) => {
    if (!isAdminNow) {
      data.settings = [];
      ui.showExplorer = false;
      return;
    }
    void loadSettings();
  }
);

onMounted(() => {
  void refreshDashboard();
  void model.runHealthCheck();
});

onBeforeUnmount(() => {
  stopScanning();
  if (feedbackDismissTimer) window.clearTimeout(feedbackDismissTimer);
});
</script>

<template>
  <UDashboardGroup class="min-h-screen bg-default">
    <UDashboardSidebar
      id="price-tracker-sidebar"
      v-model:collapsed="ui.sidebarCollapsed"
      collapsible
      resizable
      :default-size="18"
      :min-size="14"
      :max-size="24"
      class="bg-elevated/50"
    >
      <template #header>
        <div class="flex min-w-0 items-center gap-3 px-2 py-1.5">
          <div class="grid size-9 shrink-0 place-items-center rounded-lg bg-primary text-primary-foreground font-semibold">
            PT
          </div>
          <div v-if="!ui.sidebarCollapsed" class="min-w-0">
            <p class="truncate text-sm font-semibold text-highlighted">PriceTracker</p>
            <p class="truncate text-xs text-muted">Nuxt UI dashboard</p>
          </div>
        </div>
      </template>

      <UNavigationMenu
        orientation="vertical"
        :items="navigationGroups"
        class="min-h-0 flex-1"
      />

      <template #footer>
        <div class="grid gap-3">
          <USeparator />
          <div class="flex items-center gap-3">
            <UAvatar :text="accountInitials" size="md" />
            <div v-if="!ui.sidebarCollapsed" class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium text-highlighted">{{ accountName }}</p>
              <p class="truncate text-xs text-muted">{{ accountRole }}</p>
            </div>
            <UDashboardSidebarCollapse />
          </div>
        </div>
      </template>
    </UDashboardSidebar>

    <UDashboardPanel id="price-tracker-main" class="min-w-0">
      <template #header>
        <UDashboardNavbar :title="pageMeta.title" :icon="pageMeta.icon">
          <template #right>
            <div class="hidden min-w-72 md:block">
              <UInput
                v-model="ui.search"
                icon="i-lucide-search"
                placeholder="Search catalog"
                @keyup.enter.prevent="loadPublicData"
              />
            </div>
            <UButton
              color="neutral"
              variant="ghost"
              icon="i-lucide-refresh-cw"
              :loading="model.isBusy || data.loadingPublic"
              @click="refreshDashboard"
            />
            <UButton
              v-if="isLoggedIn"
              color="neutral"
              variant="outline"
              icon="i-lucide-log-out"
              label="Logout"
              @click="logout"
            />
            <UButton
              v-else
              icon="i-lucide-log-in"
              label="Login"
              @click="openPage('auth')"
            />
          </template>
        </UDashboardNavbar>

        <UDashboardToolbar>
          <template #default>
            <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
              <UBadge
                v-for="light in statusCards"
                :key="light.key"
                :color="light.color"
                variant="subtle"
                class="gap-1"
              >
                <span class="size-1.5 rounded-full bg-current" />
                {{ light.label }}: {{ light.state }}
              </UBadge>
              <UBadge v-if="maintenanceMode" color="warning" variant="subtle">
                Maintenance mode
              </UBadge>
            </div>
            <UTabs
              v-model="ui.page"
              :items="dashboardTabs"
              :content="false"
              size="sm"
              variant="link"
              class="hidden lg:block"
              @update:model-value="openPage"
            />
          </template>
        </UDashboardToolbar>
      </template>

      <template #body>
        <div class="dashboard-body">
          <UAlert
            v-if="ui.feedback.visible"
            :color="ui.feedback.tone"
            variant="subtle"
            :icon="ui.feedback.pending ? 'i-lucide-loader-circle' : 'i-lucide-info'"
            :title="ui.feedback.pending ? 'Working' : 'Status'"
            :description="ui.feedback.message"
            :actions="ui.feedback.pending ? [] : [{ label: 'Dismiss', color: 'neutral', variant: 'ghost', onClick: clearFeedback }]"
          />

          <section v-if="ui.page === 'overview'" class="dashboard-stack">
            <div class="dashboard-grid stats-grid">
              <UCard v-for="card in summaryCards" :key="card.label" class="min-w-0">
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <p class="text-sm text-muted">{{ card.label }}</p>
                    <p class="mt-2 truncate text-2xl font-semibold text-highlighted">{{ card.value }}</p>
                  </div>
                  <UButton :color="card.color" variant="subtle" :icon="card.icon" square />
                </div>
              </UCard>
            </div>

            <div class="dashboard-grid overview-grid">
              <UCard>
                <template #header>
                  <div class="flex items-center justify-between gap-3">
                    <div>
                      <h2 class="font-semibold text-highlighted">System health</h2>
                      <p class="text-sm text-muted">Browser, API, and database reachability.</p>
                    </div>
                    <UButton
                      color="neutral"
                      variant="outline"
                      icon="i-lucide-heart-pulse"
                      label="Check"
                      :loading="model.isBusy"
                      @click="model.runHealthCheck"
                    />
                  </div>
                </template>

                <div class="grid gap-3">
                  <div
                    v-for="light in statusCards"
                    :key="light.key"
                    class="flex items-start justify-between gap-4 rounded-lg border border-default p-3"
                  >
                    <div>
                      <p class="font-medium text-highlighted">{{ light.label }}</p>
                      <p class="text-sm text-muted">{{ light.detail }}</p>
                    </div>
                    <UBadge :color="light.color" variant="subtle">{{ light.state }}</UBadge>
                  </div>
                </div>
              </UCard>

              <UCard>
                <template #header>
                  <div class="flex items-center justify-between gap-3">
                    <div>
                      <h2 class="font-semibold text-highlighted">Capture context</h2>
                      <p class="text-sm text-muted">Current shop, item, and variant selections.</p>
                    </div>
                    <UButton icon="i-lucide-scan-barcode" label="Capture" @click="openPage('capture')" />
                  </div>
                </template>

                <div class="grid gap-3">
                  <div class="context-row"><span>Shop</span><strong>{{ selectedShop?.name || "Not selected" }}</strong></div>
                  <div class="context-row"><span>Item</span><strong>{{ selectedItem?.name || "Not selected" }}</strong></div>
                  <div class="context-row"><span>Variant</span><strong>{{ model.workbench.lookup.variantId || "Not selected" }}</strong></div>
                  <div class="context-row"><span>Queued writes</span><strong>{{ model.queue.length }}</strong></div>
                </div>
              </UCard>
            </div>

            <UCard>
              <template #header>
                <div class="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <h2 class="font-semibold text-highlighted">Recent responses</h2>
                    <p class="text-sm text-muted">The latest client and API events.</p>
                  </div>
                  <UButton color="neutral" variant="outline" icon="i-lucide-activity" label="Activity" @click="openPage('activity')" />
                </div>
              </template>

              <div v-if="historyRows.length" class="overflow-x-auto">
                <table class="dashboard-table">
                  <thead>
                    <tr><th>Action</th><th>Status</th><th>Started</th></tr>
                  </thead>
                  <tbody>
                    <tr v-for="entry in historyRows" :key="entry.id">
                      <td>
                        <span class="font-medium text-highlighted">{{ entry.label }}</span>
                        <code class="mt-1 block text-xs text-muted">{{ entry.url }}</code>
                      </td>
                      <td>
                        <UBadge :color="entry.ok ? 'success' : 'warning'" variant="subtle">
                          {{ entry.status || (entry.ok ? "OK" : "Local") }}
                        </UBadge>
                      </td>
                      <td class="text-muted">{{ new Date(entry.startedAt).toLocaleString() }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <UAlert
                v-else
                color="neutral"
                variant="subtle"
                icon="i-lucide-inbox"
                title="No responses yet"
                description="Run a health check or load catalog data to start the event stream."
              />
            </UCard>
          </section>

          <section v-else-if="ui.page === 'auth'" class="auth-panel">
            <UCard class="w-full max-w-2xl">
              <template #header>
                <div>
                  <h2 class="text-xl font-semibold text-highlighted">{{ pageMeta.title }}</h2>
                  <p class="text-sm text-muted">{{ pageMeta.description }}</p>
                </div>
              </template>

              <div class="grid gap-5">
                <UTabs v-model="ui.authMode" :items="authModeTabs" :content="false" />

                <div class="form-grid">
                  <UFormField label="Email">
                    <UInput v-model="model.workbench.session.email" autocomplete="email" />
                  </UFormField>
                  <UFormField label="Password">
                    <UInput
                      v-model="model.workbench.session.password"
                      type="password"
                      autocomplete="current-password"
                    />
                  </UFormField>
                  <UFormField v-if="ui.authMode !== 'login'" label="Display name">
                    <UInput v-model="model.workbench.session.displayName" />
                  </UFormField>
                  <UFormField v-if="ui.authMode !== 'login'" label="Primary phone">
                    <UInput v-model="model.workbench.session.primaryPhone" placeholder="+447700900123" />
                  </UFormField>
                </div>
              </div>

              <template #footer>
                <div class="flex flex-wrap gap-2">
                  <UButton
                    :loading="model.isBusy"
                    :icon="ui.authMode === 'login' ? 'i-lucide-log-in' : 'i-lucide-user-plus'"
                    :label="ui.authMode === 'login' ? 'Login' : ui.authMode === 'admin' ? 'Create admin' : 'Create user'"
                    @click="runAuthAction"
                  />
                  <UButton color="neutral" variant="outline" icon="i-lucide-user" label="Fill demo" @click="model.fillDemoSession" />
                  <UButton color="neutral" variant="outline" icon="i-lucide-shield" label="Fill admin" @click="model.fillAdminSession" />
                </div>
              </template>
            </UCard>
          </section>

          <section v-else-if="ui.page === 'capture'" class="dashboard-stack">
            <UAlert
              v-if="maintenanceMode"
              color="warning"
              variant="subtle"
              icon="i-lucide-construction"
              title="Maintenance mode is on"
              description="Capture remains visible, but contribution actions should be treated as paused."
            />

            <div class="dashboard-grid capture-grid">
              <UCard>
                <template #header>
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <h2 class="font-semibold text-highlighted">Scanner</h2>
                      <p class="text-sm text-muted">Live camera, image upload, or manual entry.</p>
                    </div>
                    <UButton
                      color="neutral"
                      variant="outline"
                      icon="i-lucide-camera"
                      label="Refresh cameras"
                      :loading="loadingCameras"
                      @click="loadCameras"
                    />
                  </div>
                </template>

                <div class="grid gap-4">
                  <UFormField label="Camera device">
                    <select v-model="model.workbench.capture.selectedDeviceId" class="select-input">
                      <option value="">Default camera</option>
                      <option v-for="device in cameraDevices" :key="device.deviceId" :value="device.deviceId">
                        {{ device.label || device.deviceId }}
                      </option>
                    </select>
                  </UFormField>

                  <div class="scanner-frame">
                    <video ref="videoRef" muted playsinline />
                    <div v-if="!isScanning" class="scanner-idle">
                      <UIcon name="i-lucide-scan-barcode" class="size-10" />
                      <span>Camera preview</span>
                    </div>
                  </div>

                  <div class="flex flex-wrap gap-2">
                    <UButton icon="i-lucide-play" label="Start camera" :disabled="maintenanceMode || isScanning" @click="startScanning" />
                    <UButton color="neutral" variant="outline" icon="i-lucide-square" label="Stop" :disabled="!isScanning" @click="stopScanning" />
                    <label class="inline-flex">
                      <UButton as="span" color="neutral" variant="outline" icon="i-lucide-image" label="Scan image" />
                      <input accept="image/*" type="file" class="sr-only" @change="handleImageSelection">
                    </label>
                  </div>

                  <div class="form-grid">
                    <UFormField label="Current code">
                      <UInput v-model="model.workbench.capture.lookupCode" />
                    </UFormField>
                    <UFormField label="Manual code">
                      <UInput v-model="model.workbench.capture.manualCode" @keyup.enter.prevent="useManualCode" />
                    </UFormField>
                  </div>

                  <div class="flex flex-wrap gap-2">
                    <UButton color="neutral" variant="outline" icon="i-lucide-keyboard" label="Use manual code" @click="useManualCode" />
                    <UButton
                      color="neutral"
                      variant="outline"
                      icon="i-lucide-search-check"
                      label="Lookup code"
                      :disabled="!model.workbench.capture.lookupCode || !model.workbench.lookup.shopId"
                      @click="model.lookupCapturedCode"
                    />
                  </div>

                  <UAlert
                    v-if="scanError"
                    color="error"
                    variant="subtle"
                    icon="i-lucide-triangle-alert"
                    title="Scanner error"
                    :description="scanError"
                  />

                  <div v-if="model.workbench.capture.recentCaptures.length" class="grid gap-2">
                    <div
                      v-for="capture in model.workbench.capture.recentCaptures"
                      :key="capture.capturedAt"
                      class="flex items-center justify-between gap-3 rounded-lg border border-default p-3"
                    >
                      <div class="min-w-0">
                        <p class="truncate font-medium text-highlighted">{{ capture.text }}</p>
                        <p class="text-xs text-muted">{{ new Date(capture.capturedAt).toLocaleString() }}</p>
                      </div>
                      <UBadge color="info" variant="subtle">{{ capture.format }}</UBadge>
                    </div>
                  </div>
                </div>
              </UCard>

              <div class="grid gap-4">
                <UCard>
                  <template #header>
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div>
                        <h2 class="font-semibold text-highlighted">Catalog</h2>
                        <p class="text-sm text-muted">Select a shop, item, and variant.</p>
                      </div>
                      <UButton color="neutral" variant="outline" icon="i-lucide-refresh-cw" label="Refresh" :loading="data.loadingPublic" @click="loadPublicData" />
                    </div>
                  </template>

                  <div class="form-grid">
                    <UFormField label="Search items">
                      <UInput v-model="ui.search" @keyup.enter.prevent="loadPublicData" />
                    </UFormField>
                    <UFormField label="Shop">
                      <select v-model="model.workbench.lookup.shopId" class="select-input">
                        <option value="">Select shop</option>
                        <option v-for="shop in data.shops" :key="shop.id" :value="shop.id">{{ shop.name }}</option>
                      </select>
                    </UFormField>
                    <UFormField label="Item">
                      <select v-model="model.workbench.lookup.itemId" class="select-input">
                        <option value="">Select item</option>
                        <option v-for="item in data.items" :key="item.id" :value="item.id">{{ item.name }}</option>
                      </select>
                    </UFormField>
                    <UFormField label="Variant">
                      <select v-model="model.workbench.lookup.variantId" class="select-input">
                        <option value="">Select variant</option>
                        <option v-for="variant in data.variants" :key="variant.id" :value="variant.id">
                          {{ variant.variantName || variant.id }}
                        </option>
                      </select>
                    </UFormField>
                  </div>
                </UCard>

                <UCard>
                  <template #header>
                    <div>
                      <h2 class="font-semibold text-highlighted">Purchase and price</h2>
                      <p class="text-sm text-muted">Submit current evidence to the API.</p>
                    </div>
                  </template>

                  <div class="form-grid">
                    <UFormField label="Purchase time">
                      <UInput v-model="model.workbench.submission.purchaseTime" type="datetime-local" />
                    </UFormField>
                    <UFormField label="Recorded at">
                      <UInput v-model="model.workbench.submission.recordedAt" type="datetime-local" />
                    </UFormField>
                    <UFormField label="Amount">
                      <UInput v-model="model.workbench.submission.originalAmount" />
                    </UFormField>
                    <UFormField label="Currency">
                      <UInput v-model="model.workbench.submission.originalCurrency" />
                    </UFormField>
                    <UFormField class="sm:col-span-2" label="Attachment IDs">
                      <UInput v-model="model.workbench.submission.attachmentFileIds" />
                    </UFormField>
                    <UFormField class="sm:col-span-2" label="Notes">
                      <UTextarea v-model="model.workbench.submission.priceNotes" :rows="4" />
                    </UFormField>
                  </div>

                  <template #footer>
                    <div class="flex flex-wrap gap-2">
                      <UButton
                        icon="i-lucide-shopping-bag"
                        label="Create purchase"
                        :disabled="!canUseProtectedTools"
                        :loading="model.isBusy"
                        @click="model.createPurchase"
                      />
                      <UButton
                        color="neutral"
                        variant="outline"
                        icon="i-lucide-send"
                        label="Submit price"
                        :disabled="!canUseProtectedTools"
                        :loading="model.isBusy"
                        @click="model.submitPrice"
                      />
                      <UButton color="neutral" variant="ghost" icon="i-lucide-save" label="Save capture" @click="model.saveFullCaptureForRetry" />
                    </div>
                  </template>
                </UCard>
              </div>
            </div>
          </section>

          <section v-else-if="ui.page === 'activity'" class="dashboard-stack">
            <div class="dashboard-grid activity-grid">
              <UCard>
                <template #header>
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <h2 class="font-semibold text-highlighted">Retry queue</h2>
                      <p class="text-sm text-muted">Saved writes that can be retried when the API returns.</p>
                    </div>
                    <div class="flex gap-2">
                      <UButton
                        icon="i-lucide-rotate-cw"
                        label="Flush"
                        :loading="model.isFlushingQueue"
                        :disabled="!model.queue.length || !model.online"
                        @click="model.flushQueue"
                      />
                      <UButton color="neutral" variant="outline" icon="i-lucide-trash-2" label="Clear" :disabled="!model.queue.length" @click="model.clearQueue" />
                    </div>
                  </div>
                </template>

                <div v-if="queuePreview.length" class="grid gap-3">
                  <div v-for="entry in queuePreview" :key="entry.id" class="rounded-lg border border-default p-3">
                    <div class="flex flex-wrap items-start justify-between gap-3">
                      <div class="min-w-0">
                        <p class="font-medium text-highlighted">{{ entry.label }}</p>
                        <code class="mt-1 block truncate text-xs text-muted">{{ entry.method }} {{ entry.path }}</code>
                      </div>
                      <div class="flex gap-2">
                        <UButton color="neutral" variant="outline" size="xs" label="Retry" @click="model.retryQueueEntry(entry.id)" />
                        <UButton color="error" variant="ghost" size="xs" label="Remove" @click="model.removeQueueEntry(entry.id)" />
                      </div>
                    </div>
                  </div>
                </div>
                <UAlert
                  v-else
                  color="neutral"
                  variant="subtle"
                  icon="i-lucide-circle-check"
                  title="Queue is clear"
                  description="Offline-safe writes will appear here when they are saved locally."
                />
              </UCard>

              <UCard>
                <template #header>
                  <div>
                    <h2 class="font-semibold text-highlighted">Watchlist and alerts</h2>
                    <p class="text-sm text-muted">Create activity from the currently selected variant.</p>
                  </div>
                </template>

                <div class="grid gap-4">
                  <div class="dashboard-grid stats-grid compact">
                    <div class="mini-stat"><span>Purchases</span><strong>{{ data.purchases.length }}</strong></div>
                    <div class="mini-stat"><span>Prices</span><strong>{{ data.prices.length }}</strong></div>
                    <div class="mini-stat"><span>Watchlist</span><strong>{{ data.watchlist.length }}</strong></div>
                    <div class="mini-stat"><span>Alerts</span><strong>{{ data.alerts.length }}</strong></div>
                  </div>

                  <div class="form-grid">
                    <UFormField label="Alert amount">
                      <UInput v-model="ui.alertAmount" />
                    </UFormField>
                    <UFormField label="Alert currency">
                      <UInput v-model="ui.alertCurrency" />
                    </UFormField>
                  </div>

                  <UCheckbox v-model="ui.alertEnabled" label="Enable new alert" />

                  <div class="flex flex-wrap gap-2">
                    <UButton icon="i-lucide-bookmark-plus" label="Add watchlist" :disabled="!isLoggedIn" @click="addWatchlist" />
                    <UButton color="neutral" variant="outline" icon="i-lucide-bell-plus" label="Create alert" :disabled="!isLoggedIn" @click="addAlert" />
                  </div>
                </div>
              </UCard>
            </div>

            <UCard>
              <template #header>
                <div class="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <h2 class="font-semibold text-highlighted">Response detail</h2>
                    <p class="text-sm text-muted">Latest API response payload.</p>
                  </div>
                  <UBadge color="neutral" variant="subtle">{{ model.history.length }} events</UBadge>
                </div>
              </template>

              <pre v-if="latestResponse" class="response-pre">{{ latestResponse.prettyResponse || latestResponse.rawText || "No body" }}</pre>
              <UAlert
                v-else
                color="neutral"
                variant="subtle"
                icon="i-lucide-inbox"
                title="No response selected"
                description="Send a request to inspect the payload here."
              />
            </UCard>
          </section>

          <section v-else-if="ui.page === 'account'" class="dashboard-stack">
            <UAlert
              v-if="!isLoggedIn"
              color="warning"
              variant="subtle"
              icon="i-lucide-lock"
              title="Login required"
              description="Sign in before editing account details."
              :actions="[{ label: 'Open login', icon: 'i-lucide-log-in', onClick: () => openPage('auth') }]"
            />

            <div v-else class="dashboard-grid account-grid">
              <UCard>
                <template #header>
                  <div class="flex items-center gap-4">
                    <UAvatar :text="accountInitials" size="xl" />
                    <div>
                      <h2 class="text-lg font-semibold text-highlighted">{{ accountName }}</h2>
                      <p class="text-sm text-muted">{{ accountRole }} account</p>
                    </div>
                  </div>
                </template>

                <div class="form-grid">
                  <UFormField label="Display name">
                    <UInput v-model="profileForm.displayName" />
                  </UFormField>
                  <UFormField label="Default currency">
                    <UInput v-model="profileForm.defaultCurrency" />
                  </UFormField>
                  <UFormField label="Locale">
                    <UInput v-model="profileForm.locale" />
                  </UFormField>
                  <UFormField label="Timezone">
                    <UInput v-model="profileForm.timezoneName" />
                  </UFormField>
                  <UFormField class="sm:col-span-2" label="Bio">
                    <UTextarea v-model="profileForm.profileBio" :rows="4" />
                  </UFormField>
                </div>

                <template #footer>
                  <div class="flex flex-wrap gap-2">
                    <UButton icon="i-lucide-save" label="Save profile" :loading="model.isBusy" @click="saveProfile" />
                    <UButton color="neutral" variant="outline" icon="i-lucide-refresh-cw" label="Reload" :loading="data.loadingUser" @click="loadUserData" />
                  </div>
                </template>
              </UCard>

              <UCard>
                <template #header>
                  <div>
                    <h2 class="font-semibold text-highlighted">Session</h2>
                    <p class="text-sm text-muted">Current local identity values.</p>
                  </div>
                </template>

                <div class="grid gap-3">
                  <div class="context-row"><span>Email</span><strong>{{ model.workbench.session.email || "Not set" }}</strong></div>
                  <div class="context-row"><span>Account ID</span><strong>{{ model.workbench.session.accountId || "Not set" }}</strong></div>
                  <div class="context-row"><span>Roles</span><strong>{{ model.workbench.session.roles.join(", ") || "None loaded" }}</strong></div>
                  <div class="context-row"><span>API base</span><strong>{{ model.workbench.workspace.apiBaseUrl }}</strong></div>
                </div>
              </UCard>
            </div>
          </section>

          <section v-else-if="ui.page === 'admin'" class="dashboard-stack">
            <UAlert
              v-if="!model.isAdmin"
              color="warning"
              variant="subtle"
              icon="i-lucide-shield-alert"
              title="Admin access required"
              description="Sign in with an admin account to use operations tools."
            />

            <template v-else>
              <div class="dashboard-grid stats-grid">
                <UCard v-for="card in model.adminSummaryCards" :key="card.label">
                  <div class="flex items-start justify-between gap-3">
                    <div>
                      <p class="text-sm text-muted">{{ card.label }}</p>
                      <p class="mt-2 text-2xl font-semibold text-highlighted">{{ card.value }}</p>
                    </div>
                    <UBadge :color="toneColor(card.tone)" variant="subtle">{{ card.tone }}</UBadge>
                  </div>
                </UCard>
              </div>

              <div class="dashboard-grid admin-grid">
                <UCard>
                  <template #header>
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div>
                        <h2 class="font-semibold text-highlighted">Moderation</h2>
                        <p class="text-sm text-muted">Recent price submissions awaiting review.</p>
                      </div>
                      <UButton
                        color="neutral"
                        variant="outline"
                        icon="i-lucide-refresh-cw"
                        label="Refresh"
                        :loading="model.admin.loadingModeration"
                        @click="model.loadModerationPrices"
                      />
                    </div>
                  </template>

                  <div v-if="adminModerationPreview.length" class="grid gap-3">
                    <div v-for="price in adminModerationPreview" :key="price.id" class="rounded-lg border border-default p-3">
                      <div class="flex flex-wrap items-start justify-between gap-3">
                        <div>
                          <p class="font-medium text-highlighted">{{ price.itemName || price.id }}</p>
                          <p class="text-sm text-muted">{{ price.finalAmount || price.originalAmount }} {{ price.currency || price.originalCurrency }}</p>
                        </div>
                        <div class="flex gap-2">
                          <UButton size="xs" icon="i-lucide-check" label="Approve" @click="model.approveModerationPrice(price.id)" />
                          <UButton size="xs" color="error" variant="outline" icon="i-lucide-x" label="Reject" @click="model.rejectModerationPrice(price.id)" />
                        </div>
                      </div>
                    </div>
                  </div>
                  <UAlert
                    v-else
                    color="neutral"
                    variant="subtle"
                    icon="i-lucide-inbox"
                    title="No moderation rows loaded"
                    description="Refresh admin data to inspect pending submissions."
                  />
                </UCard>

                <UCard>
                  <template #header>
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div>
                        <h2 class="font-semibold text-highlighted">Settings</h2>
                        <p class="text-sm text-muted">Quick view of system controls.</p>
                      </div>
                      <UButton color="neutral" variant="outline" icon="i-lucide-refresh-cw" label="Refresh" :loading="data.loadingSettings" @click="loadSettings" />
                    </div>
                  </template>

                  <div v-if="adminSettingsPreview.length" class="grid gap-3">
                    <div v-for="setting in adminSettingsPreview" :key="setting.key" class="rounded-lg border border-default p-3">
                      <div class="flex flex-wrap items-start justify-between gap-3">
                        <div class="min-w-0">
                          <p class="truncate font-medium text-highlighted">{{ setting.key }}</p>
                          <p class="text-sm text-muted">{{ formatValue(setting.value) }}</p>
                        </div>
                        <UButton
                          v-if="typeof setting.value === 'boolean'"
                          size="xs"
                          color="neutral"
                          variant="outline"
                          :label="setting.value ? 'Turn off' : 'Turn on'"
                          :loading="data.savingSetting === setting.key"
                          @click="saveSetting(setting, !setting.value)"
                        />
                      </div>
                    </div>
                  </div>
                  <UAlert
                    v-else
                    color="neutral"
                    variant="subtle"
                    icon="i-lucide-settings"
                    title="No settings loaded"
                    description="Refresh admin settings to view persisted controls."
                  />
                </UCard>
              </div>

              <UCard>
                <template #header>
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <h2 class="font-semibold text-highlighted">Debug explorer</h2>
                      <p class="text-sm text-muted">Hidden API tools stay behind admin access.</p>
                    </div>
                    <UButton
                      :disabled="!hiddenModulesEnabled"
                      :icon="ui.showExplorer ? 'i-lucide-eye-off' : 'i-lucide-eye'"
                      :label="ui.showExplorer ? 'Hide explorer' : 'Reveal explorer'"
                      @click="ui.showExplorer = !ui.showExplorer"
                    />
                  </div>
                </template>

                <UAlert
                  v-if="!hiddenModulesEnabled"
                  color="warning"
                  variant="subtle"
                  icon="i-lucide-lock"
                  title="Hidden modules disabled"
                  description="Enable debug.hiddenModulesEnabled to reveal the explorer."
                />

                <div v-else-if="ui.showExplorer" class="dashboard-grid explorer-grid">
                  <div class="grid gap-3">
                    <UFormField label="Group">
                      <select v-model="model.workbench.explorer.selectedGroup" class="select-input">
                        <option v-for="group in model.endpointGroups" :key="group" :value="group">{{ group }}</option>
                      </select>
                    </UFormField>
                    <UFormField label="Search endpoint">
                      <UInput v-model="model.workbench.explorer.search" />
                    </UFormField>
                    <div class="endpoint-list">
                      <UButton
                        v-for="endpoint in model.filteredEndpoints"
                        :key="endpoint.id"
                        color="neutral"
                        :variant="endpoint.id === model.selectedEndpoint.id ? 'soft' : 'ghost'"
                        class="justify-start"
                        @click="model.workbench.explorer.selectedEndpointId = endpoint.id"
                      >
                        <span class="truncate">{{ endpoint.method }} {{ endpoint.label }}</span>
                      </UButton>
                    </div>
                  </div>

                  <div class="grid gap-3">
                    <div>
                      <h3 class="font-semibold text-highlighted">{{ model.selectedEndpoint.label }}</h3>
                      <p class="text-sm text-muted">{{ model.selectedEndpoint.description }}</p>
                    </div>

                    <UTextarea v-model="model.workbench.explorer.bodyText" :rows="10" placeholder="JSON body" />

                    <UAlert
                      v-if="model.explorerError"
                      color="error"
                      variant="subtle"
                      icon="i-lucide-triangle-alert"
                      title="Explorer error"
                      :description="model.explorerError"
                    />

                    <div class="flex flex-wrap gap-2">
                      <UButton icon="i-lucide-send" label="Send request" :loading="model.isBusy" @click="model.sendExplorerRequest" />
                      <UButton color="neutral" variant="outline" icon="i-lucide-save" label="Queue request" @click="model.queueExplorerRequest" />
                      <UButton color="neutral" variant="ghost" icon="i-lucide-rotate-ccw" label="Reset" @click="model.resetExplorerDraft" />
                    </div>
                  </div>
                </div>

                <UAlert
                  v-else
                  color="neutral"
                  variant="subtle"
                  icon="i-lucide-eye"
                  title="Explorer hidden"
                  description="Reveal it when you need low-level API diagnostics."
                />
              </UCard>
            </template>
          </section>
        </div>
      </template>
    </UDashboardPanel>
  </UDashboardGroup>
</template>
