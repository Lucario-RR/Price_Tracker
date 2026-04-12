<script setup>
import { computed, onMounted, reactive, watch } from "vue";

import AdminDatabasePanel from "./components/AdminDatabasePanel.vue";
import AdminExplorerPanel from "./components/AdminExplorerPanel.vue";
import AdminModerationPanel from "./components/AdminModerationPanel.vue";
import AdminSettingsPanel from "./components/AdminSettingsPanel.vue";
import QueuePanel from "./components/QueuePanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import ScannerPanel from "./components/ScannerPanel.vue";
import { useWorkbench } from "./composables/useWorkbench";

const model = reactive(useWorkbench());

const ui = reactive({
  menuOpen: false,
  page: "home",
  authMode: "login",
  tab: "overview",
  search: "",
  alertAmount: "1.50",
  alertCurrency: "GBP",
  alertEnabled: true,
  revealDebug: false
});

const feedback = reactive({
  visible: false,
  tone: "info",
  message: "",
  pending: false
});

const signOutState = reactive({
  intentional: false
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
  defaultCurrency: "GBP"
});

const isLoggedIn = computed(() => Boolean(model.workbench.session.accountId));
const selectedShop = computed(
  () => data.shops.find((shop) => shop.id === model.workbench.lookup.shopId) || null
);
const selectedItem = computed(
  () => data.items.find((item) => item.id === model.workbench.lookup.itemId) || null
);
const settingMap = computed(() =>
  Object.fromEntries(data.settings.map((setting) => [setting.key, setting.value]))
);
const maintenanceMode = computed(
  () => settingMap.value["system.maintenanceMode"] === true
);
const hiddenModulesEnabled = computed(
  () => settingMap.value["debug.hiddenModulesEnabled"] !== false
);
const drawerLinks = computed(() =>
  [
    { label: "Home", page: "home" },
    {
      label: isLoggedIn.value ? "Overview" : "Login / Register",
      page: isLoggedIn.value ? "dashboard" : "auth",
      tab: "overview"
    },
    isLoggedIn.value ? { label: "Capture", page: "dashboard", tab: "capture" } : null,
    isLoggedIn.value ? { label: "Activity", page: "dashboard", tab: "activity" } : null,
    model.isAdmin ? { label: "Admin Console", page: "dashboard", tab: "admin" } : null
  ].filter(Boolean)
);

function openPage(page, tab = ui.tab) {
  ui.page = page;
  ui.tab = tab;
  ui.menuOpen = false;

  if (page === "dashboard") {
    refreshDashboard();
  }
}

function formatValue(value) {
  if (value === undefined || value === null) {
    return "Not set";
  }

  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

function clearFeedback() {
  feedback.visible = false;
  feedback.tone = "info";
  feedback.message = "";
  feedback.pending = false;
}

function showFeedback(tone, message, pending = false) {
  feedback.visible = Boolean(message);
  feedback.tone = tone;
  feedback.message = message;
  feedback.pending = pending;
}

function resultErrorMessage(result, fallbackMessage) {
  if (result?.data?.error?.message) {
    return result.data.error.message;
  }

  if (result?.error) {
    return result.error;
  }

  if (result?.status) {
    return `${fallbackMessage} (HTTP ${result.status}).`;
  }

  return fallbackMessage;
}

async function runServerAction({
  pendingMessage,
  successMessage,
  queuedMessage,
  errorMessage,
  task
}) {
  showFeedback("info", pendingMessage, true);

  try {
    const result = await task();

    if (result?.ok) {
      showFeedback("success", successMessage, false);
      return result;
    }

    if (result?.queued) {
      showFeedback(
        "warn",
        queuedMessage || "Request saved locally and will retry when the backend is reachable.",
        false
      );
      return result;
    }

    showFeedback("danger", resultErrorMessage(result, errorMessage), false);
    return result;
  } catch (error) {
    showFeedback("danger", String(error.message || error), false);
    return null;
  }
}

function resetProtectedData() {
  data.profile = null;
  data.purchases = [];
  data.prices = [];
  data.watchlist = [];
  data.alerts = [];
  data.settings = [];
  data.savingSetting = "";
  profileForm.displayName = "";
  profileForm.defaultCurrency = "GBP";
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
    data.profile = null;
    data.purchases = [];
    data.prices = [];
    data.watchlist = [];
    data.alerts = [];
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
      profileForm.displayName = profile.data.data.displayName || "";
      profileForm.defaultCurrency = profile.data.data.defaultCurrency || "GBP";
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
  if (model.isAdmin) await loadSettings();
}

async function afterAuth(nextTab = "overview") {
  await refreshDashboard();
  openPage("dashboard", nextTab);
}

async function login() {
  const result = await runServerAction({
    pendingMessage: "Signing you in...",
    successMessage: "Signed in successfully.",
    errorMessage: "Unable to sign in.",
    task: async () => {
      const response = await model.loginSession();
      if (response?.ok) {
        await afterAuth(model.isAdmin ? "admin" : "overview");
      }
      return response;
    }
  });

  return result;
}

async function registerUser() {
  const result = await runServerAction({
    pendingMessage: "Creating your account...",
    successMessage: "Account created and signed in.",
    errorMessage: "Unable to create the account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register");
      if (response?.ok) {
        await afterAuth("overview");
      }
      return response;
    }
  });

  return result;
}

async function registerAdmin() {
  const result = await runServerAction({
    pendingMessage: "Creating the admin account...",
    successMessage: "Admin account created and signed in.",
    errorMessage: "Unable to create the admin account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register-admin");
      if (response?.ok) {
        await afterAuth("admin");
      }
      return response;
    }
  });

  return result;
}

async function logout() {
  signOutState.intentional = true;
  showFeedback("info", "Signing out and clearing local app data...", true);

  const result = await model.logoutSession();
  openPage("home");
  resetProtectedData();
  ui.revealDebug = false;

  if (result?.ok) {
    showFeedback(
      "success",
      "Signed out. Local cached data was cleared and the backend session cookie was expired.",
      false
    );
  } else {
    showFeedback(
      "warn",
      `Local cached data was cleared, but the backend sign-out could not be confirmed. ${resultErrorMessage(result, "Please refresh if you still appear signed in.")}`,
      false
    );
  }

  signOutState.intentional = false;
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
          defaultCurrency: profileForm.defaultCurrency
        }
      })
  });

  if (result?.ok) {
    model.workbench.session.displayName = profileForm.displayName;
    await loadUserData();
  }
}

async function addWatchlist() {
  if (!model.workbench.lookup.variantId) return;
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
  if (!model.workbench.lookup.variantId) return;
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

watch(() => model.workbench.lookup.itemId, (itemId) => {
  loadVariants(itemId);
});

watch(isLoggedIn, (loggedIn, wasLoggedIn) => {
  if (loggedIn) {
    return;
  }

  resetProtectedData();

  if (wasLoggedIn && !signOutState.intentional) {
    showFeedback(
      "warn",
      "Your session is no longer valid. Protected local data was cleared. Please sign in again.",
      false
    );
  }
});

watch(() => model.isAdmin, (isAdminNow) => {
  if (!isAdminNow) {
    data.settings = [];
    ui.revealDebug = false;
    return;
  }

  loadSettings();
});

onMounted(() => {
  refreshDashboard();
});
</script>

<template>
  <div class="app-shell">
    <div v-if="ui.menuOpen" class="drawer-backdrop" @click="ui.menuOpen = false" />

    <aside :class="['nav-drawer', ui.menuOpen ? 'is-open' : '']">
      <div class="drawer-head">
        <div>
          <p class="eyebrow">PriceTracker</p>
          <h2>Menu</h2>
        </div>
        <button class="ghost-icon-button" @click="ui.menuOpen = false">Close</button>
      </div>

      <button
        v-for="link in drawerLinks"
        :key="link.label"
        class="drawer-link"
        @click="openPage(link.page, link.tab)"
      >
        <strong>{{ link.label }}</strong>
      </button>

      <p class="helper-text">Debug modules stay hidden inside the admin console until revealed.</p>
    </aside>

    <div class="app-main">
      <header class="topbar">
        <div class="topbar-left">
          <button class="menu-button" @click="ui.menuOpen = true">Menu</button>
          <div>
            <p class="brand-kicker">Publish Version</p>
            <h1>PriceTracker</h1>
          </div>
        </div>

        <div class="topbar-status">
          <span
            v-for="light in model.statusLights"
            :key="light.key"
            :class="['mini-status', `tone-${light.tone}`]"
          >
            <span class="mini-dot" /> {{ light.label }}: {{ light.state }}
          </span>
        </div>

        <div class="topbar-auth">
          <template v-if="isLoggedIn">
            <button class="account-chip" @click="openPage('dashboard', 'overview')">
              <strong>{{ model.workbench.session.displayName || model.workbench.session.email }}</strong>
              <span>{{ model.isAdmin ? "Admin" : "User" }}</span>
            </button>
            <button class="ghost-button" :disabled="model.isBusy" @click="logout">{{ feedback.pending ? "Signing out..." : "Logout" }}</button>
          </template>
          <button v-else class="primary-button" @click="openPage('auth')">Login / Register</button>
        </div>
      </header>

      <main class="page-shell">
        <section
          v-if="feedback.visible"
          :class="['feedback-banner', `feedback-${feedback.tone}`, feedback.pending ? 'is-pending' : '']"
        >
          <div>
            <strong>{{ feedback.pending ? "Sending request" : "Status" }}</strong>
            <p>{{ feedback.message }}</p>
          </div>
          <div class="feedback-actions">
            <span v-if="feedback.pending" class="status-pill tone-info">Sending...</span>
            <button v-else class="ghost-button compact-button" @click="clearFeedback">Dismiss</button>
          </div>
        </section>

        <section v-if="ui.page === 'home'" class="stack-lg">
          <section class="hero-card">
            <div>
              <p class="eyebrow">Dashboard-first frontend</p>
              <h2>Scan products, save price evidence, and unlock deeper tools only when the role allows it.</h2>
              <p class="helper-text">
                This version starts on a clean home page, moves into a dedicated login/register screen,
                and keeps admin debugging hidden behind an extra reveal step.
              </p>
              <div class="action-row wrap">
                <button class="primary-button" @click="openPage(isLoggedIn ? 'dashboard' : 'auth')">
                  {{ isLoggedIn ? "Open dashboard" : "Get started" }}
                </button>
                <button class="ghost-button" @click="model.runHealthCheck">Check health</button>
              </div>
            </div>

            <div class="summary-grid">
              <article
                v-for="card in model.summaryCards"
                :key="card.label"
                :class="['summary-card', `summary-${card.tone}`]"
              >
                <span class="summary-label">{{ card.label }}</span>
                <strong class="summary-value">{{ card.value }}</strong>
              </article>
            </div>
          </section>

          <section class="status-grid">
            <article
              v-for="light in model.statusLights"
              :key="light.key"
              :class="['status-card', `summary-${light.tone}`]"
            >
              <div class="status-card-head">
                <span :class="['status-dot', `tone-${light.tone}`]" />
                <strong>{{ light.label }}</strong>
              </div>
              <strong class="summary-value">{{ light.state }}</strong>
              <p class="helper-text">{{ light.detail }}</p>
            </article>
          </section>
        </section>

        <section v-else-if="ui.page === 'auth'" class="auth-layout">
          <article class="panel">
            <div class="tab-strip">
              <button :class="['tab-button', ui.authMode === 'login' ? 'is-active' : '']" @click="ui.authMode = 'login'">Login</button>
              <button :class="['tab-button', ui.authMode === 'register' ? 'is-active' : '']" @click="ui.authMode = 'register'">Register</button>
              <button :class="['tab-button', ui.authMode === 'admin' ? 'is-active' : '']" @click="ui.authMode = 'admin'">Admin Setup</button>
            </div>

            <div class="field-grid two-up">
              <label class="field">
                <span>Email</span>
                <input v-model="model.workbench.session.email" />
              </label>
              <label class="field">
                <span>Password</span>
                <input v-model="model.workbench.session.password" type="password" />
              </label>
              <label v-if="ui.authMode !== 'login'" class="field">
                <span>Display name</span>
                <input v-model="model.workbench.session.displayName" />
              </label>
              <label v-if="ui.authMode !== 'login'" class="field">
                <span>Primary phone</span>
                <input v-model="model.workbench.session.primaryPhone" />
              </label>
            </div>

            <div class="action-row wrap">
              <button v-if="ui.authMode === 'login'" class="primary-button" :disabled="model.isBusy" @click="login">
                {{ model.isBusy ? "Signing in..." : "Login" }}
              </button>
              <button v-else-if="ui.authMode === 'register'" class="primary-button" :disabled="model.isBusy" @click="registerUser">
                {{ model.isBusy ? "Creating account..." : "Create user" }}
              </button>
              <button v-else class="primary-button" :disabled="model.isBusy" @click="registerAdmin">
                {{ model.isBusy ? "Creating admin..." : "Create admin" }}
              </button>
              <button class="ghost-button" :disabled="model.isBusy" @click="model.fillDemoSession">Fill demo</button>
              <button class="ghost-button" :disabled="model.isBusy" @click="model.fillAdminSession">Fill seeded admin</button>
            </div>

            <p class="helper-text">
              <span v-if="ui.authMode === 'admin'">Temporary bootstrap only. Remove or disable this flow later.</span>
              <span v-else>Use this page for the regular login and registration flow.</span>
            </p>
          </article>
        </section>

        <section v-else-if="!isLoggedIn" class="panel empty-state">
          <h2>Sign in to unlock the dashboard</h2>
          <button class="primary-button" @click="openPage('auth')">Open auth page</button>
        </section>

        <section v-else class="stack-lg">
          <section class="dashboard-hero">
            <div>
              <p class="eyebrow">Workspace</p>
              <h2>{{ model.workbench.session.displayName || model.workbench.session.email }}</h2>
              <p class="helper-text">
                {{ model.isAdmin ? "Admin and user tools are available. Debug stays hidden until revealed." : "User tools are unlocked for your account." }}
              </p>
            </div>

            <div class="action-row wrap">
              <button class="primary-button" @click="refreshDashboard">Refresh dashboard</button>
              <button class="ghost-button" @click="model.runHealthCheck">Check health</button>
            </div>
          </section>

          <section class="tab-strip">
            <button :class="['tab-button', ui.tab === 'overview' ? 'is-active' : '']" @click="ui.tab = 'overview'">Overview</button>
            <button :class="['tab-button', ui.tab === 'capture' ? 'is-active' : '']" @click="ui.tab = 'capture'">Capture</button>
            <button :class="['tab-button', ui.tab === 'activity' ? 'is-active' : '']" @click="ui.tab = 'activity'">Activity</button>
            <button v-if="model.isAdmin" :class="['tab-button', ui.tab === 'admin' ? 'is-active' : '']" @click="ui.tab = 'admin'">Admin</button>
          </section>

          <section v-if="ui.tab === 'overview'" class="preview-grid">
            <article class="panel">
              <div class="section-head">
                <div>
                  <p class="section-kicker">Profile</p>
                  <h2>Account details</h2>
                </div>
                <span :class="['status-pill', model.isAdmin ? 'tone-accent' : 'tone-good']">
                  {{ model.isAdmin ? "Admin" : "User" }}
                </span>
              </div>

              <div class="field-grid two-up">
                <label class="field">
                  <span>Display name</span>
                  <input v-model="profileForm.displayName" />
                </label>
                <label class="field">
                  <span>Default currency</span>
                  <input v-model="profileForm.defaultCurrency" />
                </label>
              </div>

              <div class="action-row wrap">
                <button class="primary-button" :disabled="model.isBusy" @click="saveProfile">
                  {{ model.isBusy ? "Saving..." : "Save profile" }}
                </button>
                <button class="ghost-button" @click="loadUserData">Reload</button>
              </div>
            </article>

            <article class="panel">
              <div class="section-head">
                <div>
                  <p class="section-kicker">Current context</p>
                  <h2>Selected capture data</h2>
                </div>
              </div>

              <div class="simple-list">
                <div class="list-row"><strong>Shop</strong><span>{{ selectedShop?.name || "Not selected" }}</span></div>
                <div class="list-row"><strong>Item</strong><span>{{ selectedItem?.name || "Not selected" }}</span></div>
                <div class="list-row"><strong>Variant</strong><span>{{ model.workbench.lookup.variantId || "Not selected" }}</span></div>
                <div class="list-row"><strong>Maintenance mode</strong><span>{{ maintenanceMode ? "On" : "Off" }}</span></div>
              </div>
            </article>
          </section>

          <template v-else-if="ui.tab === 'capture'">
            <section v-if="maintenanceMode" class="notice-card">
              Maintenance mode is on. Capture is visible, but contribution actions should be treated as paused.
            </section>

            <section class="preview-grid">
              <ScannerPanel
                :model="model.workbench.capture"
                :shop-id="model.workbench.lookup.shopId"
                :disabled="model.isBusy || maintenanceMode"
                @capture="model.handleCapturedCode"
                @lookup="model.lookupCapturedCode"
              />

              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">Catalog</p>
                    <h2>Lookup and selection</h2>
                  </div>
                  <button class="ghost-button" :disabled="data.loadingPublic" @click="loadPublicData">Refresh</button>
                </div>

                <div class="field-grid">
                  <label class="field">
                    <span>Search items</span>
                    <input v-model="ui.search" @keyup.enter.prevent="loadPublicData" />
                  </label>
                  <label class="field">
                    <span>Shop</span>
                    <select v-model="model.workbench.lookup.shopId">
                      <option value="">Select</option>
                      <option v-for="shop in data.shops" :key="shop.id" :value="shop.id">{{ shop.name }}</option>
                    </select>
                  </label>
                  <label class="field">
                    <span>Item</span>
                    <select v-model="model.workbench.lookup.itemId">
                      <option value="">Select</option>
                      <option v-for="item in data.items" :key="item.id" :value="item.id">{{ item.name }}</option>
                    </select>
                  </label>
                  <label class="field">
                    <span>Variant</span>
                    <select v-model="model.workbench.lookup.variantId">
                      <option value="">Select</option>
                      <option v-for="variant in data.variants" :key="variant.id" :value="variant.id">
                        {{ variant.variantName || variant.id }}
                      </option>
                    </select>
                  </label>
                </div>

                <div class="action-row wrap">
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.lookupCapturedCode">Lookup code</button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.loadVariantDetail">Variant detail</button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.compareVariant">Compare</button>
                </div>
              </article>
            </section>

            <section class="preview-grid">
              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">Submission</p>
                    <h2>Purchase and price</h2>
                  </div>
                </div>

                <div class="field-grid two-up">
                  <label class="field"><span>Purchase time</span><input v-model="model.workbench.submission.purchaseTime" type="datetime-local" /></label>
                  <label class="field"><span>Recorded at</span><input v-model="model.workbench.submission.recordedAt" type="datetime-local" /></label>
                  <label class="field"><span>Amount</span><input v-model="model.workbench.submission.originalAmount" /></label>
                  <label class="field"><span>Currency</span><input v-model="model.workbench.submission.originalCurrency" /></label>
                  <label class="field full-span"><span>Attachment IDs</span><input v-model="model.workbench.submission.attachmentFileIds" /></label>
                  <label class="field full-span"><span>Notes</span><textarea v-model="model.workbench.submission.priceNotes" rows="4" /></label>
                </div>

                <div class="action-row wrap">
                  <button class="primary-button" :disabled="model.isBusy || maintenanceMode" @click="model.createPurchase">Create purchase</button>
                  <button class="ghost-button" :disabled="model.isBusy || maintenanceMode" @click="model.submitPrice">Submit price</button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.saveFullCaptureForRetry">Save capture</button>
                </div>
              </article>

              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">Files</p>
                    <h2>Receipt helper</h2>
                  </div>
                </div>

                <div class="field-grid two-up">
                  <label class="field"><span>Filename</span><input v-model="model.workbench.submission.fileUpload.filename" /></label>
                  <label class="field"><span>Type</span><input v-model="model.workbench.submission.fileUpload.contentType" /></label>
                  <label class="field"><span>Size</span><input v-model="model.workbench.submission.fileUpload.size" /></label>
                  <label class="field"><span>Purpose</span><input v-model="model.workbench.submission.fileUpload.purpose" /></label>
                </div>

                <div class="action-row wrap">
                  <button class="primary-button" :disabled="model.isBusy" @click="model.createUploadIntent">Create upload intent</button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.completeUploadIntent">Complete upload</button>
                </div>
              </article>
            </section>
          </template>

          <template v-else-if="ui.tab === 'activity'">
            <section class="preview-grid">
              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">My data</p>
                    <h2>Purchases, prices, watchlist, alerts</h2>
                  </div>
                </div>

                <div class="simple-list">
                  <div class="list-row"><strong>Purchases</strong><span>{{ data.purchases.length }}</span></div>
                  <div class="list-row"><strong>Prices</strong><span>{{ data.prices.length }}</span></div>
                  <div class="list-row"><strong>Watchlist</strong><span>{{ data.watchlist.length }}</span></div>
                  <div class="list-row"><strong>Alerts</strong><span>{{ data.alerts.length }}</span></div>
                </div>

                <div class="field-grid two-up">
                  <label class="field"><span>Alert amount</span><input v-model="ui.alertAmount" /></label>
                  <label class="field"><span>Alert currency</span><input v-model="ui.alertCurrency" /></label>
                </div>

                <label class="checkbox-field">
                  <input v-model="ui.alertEnabled" type="checkbox" />
                  <span>Enable new alert</span>
                </label>

                <div class="action-row wrap">
                  <button class="primary-button" :disabled="model.isBusy" @click="addWatchlist">
                    {{ model.isBusy ? "Saving..." : "Add current variant to watchlist" }}
                  </button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="addAlert">
                    {{ model.isBusy ? "Saving..." : "Create alert" }}
                  </button>
                </div>
              </article>

              <QueuePanel
                :queue="model.queue"
                :online="model.online"
                :flushing="model.isFlushingQueue"
                @flush="model.flushQueue"
                @retry="model.retryQueueEntry"
                @remove="model.removeQueueEntry"
                @clear="model.clearQueue"
              />
            </section>

            <ResponsePanel :history="model.history" />
          </template>

          <template v-else-if="model.isAdmin">
            <section class="summary-grid">
              <article
                v-for="card in model.adminSummaryCards"
                :key="card.label"
                :class="['summary-card', `summary-${card.tone}`]"
              >
                <span class="summary-label">{{ card.label }}</span>
                <strong class="summary-value">{{ card.value }}</strong>
              </article>
            </section>

            <section class="preview-grid">
              <AdminModerationPanel
                :prices="model.admin.moderationPrices"
                :loading="model.admin.loadingModeration"
                :disabled="model.isBusy"
                @refresh="model.loadModerationPrices"
                @approve="model.approveModerationPrice"
                @reject="model.rejectModerationPrice"
              />

              <AdminSettingsPanel
                :settings="data.settings"
                :loading="data.loadingSettings"
                :saving-key="data.savingSetting"
                :disabled="model.isBusy"
                @refresh="loadSettings"
                @save="saveSetting"
              />
            </section>

            <section class="preview-grid">
              <AdminDatabasePanel
                :tables="model.adminTables"
                :table="model.adminTable"
                :admin="model.admin"
                :disabled="model.isBusy"
                @select-table="model.selectAdminTable"
                @select-row="model.selectAdminRow"
                @create="model.startCreateAdminRecord"
                @refresh="model.refreshAdminTable"
                @save="model.saveAdminRecord"
              />

              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">Hidden tools</p>
                    <h2>Debug access</h2>
                  </div>
                </div>

                <p class="helper-text">Keep the publish UI clean, then reveal the explorer only when needed.</p>

                <div class="simple-list">
                  <div class="list-row"><strong>Maintenance mode</strong><span>{{ maintenanceMode ? "On" : "Off" }}</span></div>
                  <div class="list-row"><strong>Hidden modules</strong><span>{{ hiddenModulesEnabled ? "Available" : "Disabled" }}</span></div>
                  <div class="list-row"><strong>Banner text</strong><span>{{ formatValue(settingMap["ui.publishBannerText"]) }}</span></div>
                </div>

                <div class="action-row wrap">
                  <button class="primary-button" :disabled="!hiddenModulesEnabled" @click="ui.revealDebug = !ui.revealDebug">
                    {{ ui.revealDebug ? "Hide debug modules" : "Reveal debug modules" }}
                  </button>
                </div>
              </article>
            </section>

            <section v-if="ui.revealDebug && hiddenModulesEnabled" class="preview-grid">
              <AdminExplorerPanel
                :model="model.workbench"
                :endpoint-groups="model.endpointGroups"
                :filtered-endpoints="model.filteredEndpoints"
                :selected-endpoint="model.selectedEndpoint"
                :explorer-error="model.explorerError"
                :busy="model.isBusy"
                :enabled="true"
                @send="model.sendExplorerRequest"
                @queue="model.queueExplorerRequest"
                @reset="model.resetExplorerDraft"
                @add-header="model.addExplorerHeader"
                @remove-header="model.removeExplorerHeader"
              />

              <ResponsePanel :history="model.history" />
            </section>
          </template>

          <article v-else class="panel empty-state">
            <h2>Admin console is locked</h2>
            <button class="primary-button" @click="openPage('auth')">Open auth page</button>
          </article>
        </section>
      </main>
    </div>
  </div>
</template>



