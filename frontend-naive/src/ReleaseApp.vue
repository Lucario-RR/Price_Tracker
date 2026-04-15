<script setup>
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, reactive, watch } from "vue";

import QueuePanel from "./components/QueuePanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import ScannerPanel from "./components/ScannerPanel.vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import UserAccountPanel from "./components/UserAccountPanel.vue";
import { appConfig } from "./config";
import { useWorkbench } from "./composables/useWorkbench";

const AdminPortalPage = defineAsyncComponent(() => import("./components/AdminPortalPage.vue"));
const AVATAR_PREVIEW_STORAGE_KEY = `${appConfig.storagePrefix}:avatar-preview`;
const FEEDBACK_AUTO_DISMISS_MS = 3200;

const model = reactive(useWorkbench());

const ui = reactive({
  menuPinned: false,
  page: "home",
  authMode: "login",
  tab: "overview",
  search: "",
  alertAmount: "1.50",
  alertCurrency: "GBP",
  alertEnabled: true
});

const feedback = reactive({
  visible: false,
  tone: "info",
  message: "",
  pending: false
});

const confirmDialog = reactive({
  visible: false,
  title: "Confirm action",
  message: "",
  confirmLabel: "Confirm",
  cancelLabel: "Cancel",
  tone: "danger"
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
  emails: [],
  phones: [],
  cookiePreferences: {
    analytics: false,
    marketing: false,
    preferences: true,
    updatedAt: ""
  },
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

const emailDraft = reactive({
  email: "",
  emailRole: "SECONDARY",
  isLoginEnabled: true
});

const phoneDraft = reactive({
  phoneNumber: ""
});

const passwordForm = reactive({
  currentPassword: "",
  newPassword: "",
  confirmPassword: ""
});

const avatarState = reactive({
  selectedFile: null,
  selectedFileName: ""
});

const avatarPreview = reactive({
  accountId: "",
  fileId: "",
  filename: "",
  dataUrl: ""
});

const authState = reactive({
  confirmPassword: "",
  showPassword: false,
  showConfirmPassword: false
});

let confirmDialogResolver = null;
let feedbackDismissTimer = null;

const isLoggedIn = computed(() => Boolean(model.workbench.session.accountId));
const accountDisplayName = computed(
  () => model.workbench.session.displayName || model.workbench.session.email || "Guest"
);
const accountRoleLabel = computed(() => {
  if (!isLoggedIn.value) {
    return "Guest mode";
  }

  return model.isAdmin ? "Admin account" : "User account";
});
const accountAvatarInitials = computed(() => createAvatarInitials(accountDisplayName.value));
const accountAvatarPreviewUrl = computed(() => {
  const accountId = model.workbench.session.accountId;
  const avatarFileId = data.profile?.avatarFileId;

  if (!accountId || !avatarFileId) {
    return "";
  }

  return avatarPreview.accountId === accountId && avatarPreview.fileId === avatarFileId
    ? avatarPreview.dataUrl
    : "";
});
const accountAvatarLabel = computed(() => {
  if (!isLoggedIn.value) {
    return "Sign in to add your own avatar.";
  }

  if (data.profile?.avatarFilename) {
    return data.profile.avatarFilename;
  }

  if (data.profile?.avatarFileId) {
    return "Avatar attached to this account.";
  }

  return "Add an avatar from your account settings.";
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
const drawerLinks = computed(() =>
  [
    { label: "Home", page: "home", description: "Overview and connection status" },
    {
      label: isLoggedIn.value ? "Dashboard" : "Login / Register",
      page: isLoggedIn.value ? "dashboard" : "auth",
      tab: "overview",
      description: isLoggedIn.value ? "Personal tools and account settings" : "Sign in or create an account"
    },
    isLoggedIn.value
      ? { label: "Capture", page: "dashboard", tab: "capture", description: "Submit prices and purchases" }
      : null,
    isLoggedIn.value
      ? { label: "Activity", page: "dashboard", tab: "activity", description: "Your watchlist, alerts, and queue" }
      : null
  ].filter(Boolean)
);

const feedbackAlertType = computed(() => {
  const t = feedback.tone;
  if (t === "danger") return "error";
  if (t === "warn") return "warning";
  if (t === "success") return "success";
  return "info";
});

function statusTagType(tone) {
  const t = String(tone || "");
  if (t === "good") return "success";
  if (t === "warn") return "warning";
  if (t === "danger") return "error";
  if (t === "accent") return "info";
  if (t === "info") return "info";
  return "default";
}

function openPage(page, tab = ui.tab) {
  if (page === "dashboard") {
    ui.page = "dashboard";
    ui.tab = tab;
    void refreshDashboard();
    return;
  }

  if (page === "admin") {
    if (!model.isAdmin) {
      ui.page = isLoggedIn.value ? "dashboard" : "auth";
      ui.tab = "overview";
      return;
    }

    ui.page = "admin";
    void refreshAdminPage();
    return;
  }

  ui.page = page;
}

function applyProfileToForm(profile) {
  profileForm.displayName = profile?.displayName || "";
  profileForm.defaultCurrency = profile?.defaultCurrency || "GBP";
  profileForm.locale = profile?.locale || "en-GB";
  profileForm.timezoneName = profile?.timezoneName || "Europe/London";
  profileForm.profileBio = profile?.profileBio || "";
}

function createAvatarInitials(value) {
  const source = String(value || "")
    .replace(/@.*$/, "")
    .trim();

  if (!source) {
    return "PT";
  }

  const parts = source.split(/[\s._-]+/).filter(Boolean);
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }

  return `${parts[0][0] || ""}${parts[1][0] || ""}`.toUpperCase();
}

function persistAvatarPreview(nextPreview = {}) {
  avatarPreview.accountId = nextPreview.accountId || "";
  avatarPreview.fileId = nextPreview.fileId || "";
  avatarPreview.filename = nextPreview.filename || "";
  avatarPreview.dataUrl = nextPreview.dataUrl || "";

  if (typeof window === "undefined") {
    return;
  }

  try {
    if (avatarPreview.accountId && avatarPreview.fileId && avatarPreview.dataUrl) {
      window.localStorage.setItem(
        AVATAR_PREVIEW_STORAGE_KEY,
        JSON.stringify({
          accountId: avatarPreview.accountId,
          fileId: avatarPreview.fileId,
          filename: avatarPreview.filename,
          dataUrl: avatarPreview.dataUrl
        })
      );
      return;
    }

    window.localStorage.removeItem(AVATAR_PREVIEW_STORAGE_KEY);
  } catch (_error) {
    // Ignore avatar-preview persistence failures so the rest of the app keeps working.
  }
}

function restoreAvatarPreview() {
  if (typeof window === "undefined") {
    return;
  }

  try {
    const stored = window.localStorage.getItem(AVATAR_PREVIEW_STORAGE_KEY);
    if (!stored) {
      persistAvatarPreview();
      return;
    }

    const parsed = JSON.parse(stored);
    persistAvatarPreview(parsed && typeof parsed === "object" ? parsed : {});
  } catch (_error) {
    persistAvatarPreview();
  }
}

function clearAvatarPreview(accountId = "") {
  if (!accountId || avatarPreview.accountId === accountId) {
    persistAvatarPreview();
  }
}

function syncAvatarPreviewWithProfile(profile) {
  const accountId = model.workbench.session.accountId;
  const avatarFileId = profile?.avatarFileId || "";

  if (!accountId || !avatarFileId) {
    clearAvatarPreview(accountId);
    return;
  }

  if (avatarPreview.accountId !== accountId || avatarPreview.fileId !== avatarFileId) {
    if (avatarPreview.accountId === accountId) {
      persistAvatarPreview();
    }
    return;
  }

  if (profile?.avatarFilename && avatarPreview.filename !== profile.avatarFilename) {
    persistAvatarPreview({
      accountId,
      fileId: avatarFileId,
      filename: profile.avatarFilename,
      dataUrl: avatarPreview.dataUrl
    });
  }
}

function readFileAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = () => resolve(typeof reader.result === "string" ? reader.result : "");
    reader.onerror = () =>
      reject(reader.error || new Error("Unable to read the avatar image."));
    reader.readAsDataURL(file);
  });
}

function clearFeedback() {
  if (feedbackDismissTimer) {
    window.clearTimeout(feedbackDismissTimer);
    feedbackDismissTimer = null;
  }

  feedback.visible = false;
  feedback.tone = "info";
  feedback.message = "";
  feedback.pending = false;
}

function showFeedback(tone, message, pending = false) {
  if (feedbackDismissTimer) {
    window.clearTimeout(feedbackDismissTimer);
    feedbackDismissTimer = null;
  }

  feedback.visible = Boolean(message);
  feedback.tone = tone;
  feedback.message = message;
  feedback.pending = pending;

  if (
    typeof window !== "undefined" &&
    tone === "success" &&
    !pending &&
    message
  ) {
    const expectedTone = tone;
    const expectedMessage = message;

    feedbackDismissTimer = window.setTimeout(() => {
      if (
        feedback.visible &&
        !feedback.pending &&
        feedback.tone === expectedTone &&
        feedback.message === expectedMessage
      ) {
        clearFeedback();
      }
    }, FEEDBACK_AUTO_DISMISS_MS);
  }
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

async function runServerAction({ pendingMessage, successMessage, queuedMessage, errorMessage, task }) {
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

function resetIdentityDrafts() {
  emailDraft.email = "";
  emailDraft.emailRole = "SECONDARY";
  emailDraft.isLoginEnabled = true;
  phoneDraft.phoneNumber = "";
  passwordForm.currentPassword = "";
  passwordForm.newPassword = "";
  passwordForm.confirmPassword = "";
  avatarState.selectedFile = null;
  avatarState.selectedFileName = "";
}

function resetProtectedData() {
  data.profile = null;
  data.purchases = [];
  data.prices = [];
  data.watchlist = [];
  data.alerts = [];
  data.settings = [];
  data.emails = [];
  data.phones = [];
  data.savingSetting = "";
  data.cookiePreferences = {
    analytics: false,
    marketing: false,
    preferences: true,
    updatedAt: ""
  };
  applyProfileToForm(null);
  resetIdentityDrafts();
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
    const [profile, purchases, prices, watchlist, alerts, emails, phones, cookiePreferences] =
      await Promise.all([
        model.requestEndpoint("me-get"),
        model.requestEndpoint("purchase-list"),
        model.requestEndpoint("price-list"),
        model.requestEndpoint("watchlist-list"),
        model.requestEndpoint("alerts-list"),
        model.requestEndpoint("emails-list"),
        model.requestEndpoint("phones-list"),
        model.requestEndpoint("cookie-preferences-get")
      ]);

    if (profile?.ok && profile.data?.data) {
      data.profile = profile.data.data;
      applyProfileToForm(profile.data.data);
      syncAvatarPreviewWithProfile(profile.data.data);
    } else {
      syncAvatarPreviewWithProfile(null);
    }

    if (purchases?.ok && Array.isArray(purchases.data?.data)) data.purchases = purchases.data.data;
    if (prices?.ok && Array.isArray(prices.data?.data)) data.prices = prices.data.data;
    if (watchlist?.ok && Array.isArray(watchlist.data?.data)) data.watchlist = watchlist.data.data;
    if (alerts?.ok && Array.isArray(alerts.data?.data)) data.alerts = alerts.data.data;
    if (emails?.ok && Array.isArray(emails.data?.data)) data.emails = emails.data.data;
    if (phones?.ok && Array.isArray(phones.data?.data)) data.phones = phones.data.data;
    if (cookiePreferences?.ok && cookiePreferences.data?.data) {
      data.cookiePreferences = { ...cookiePreferences.data.data };
    }
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

  if (model.workbench.lookup.itemId) {
    await loadVariants(model.workbench.lookup.itemId);
  }

  if (isLoggedIn.value) {
    await loadUserData();
  }
}

async function refreshAdminPage() {
  if (!model.isAdmin) {
    return;
  }

  await Promise.all([refreshDashboard(), model.loadAdminBootstrap(), loadSettings()]);
}

async function afterAuth() {
  authState.confirmPassword = "";
  authState.showPassword = false;
  authState.showConfirmPassword = false;
  await refreshDashboard();
  ui.page = "dashboard";
  ui.tab = "overview";
}

function togglePinnedMenu() {
  ui.menuPinned = !ui.menuPinned;
}

function showHeldPassword(target, key) {
  target[key] = true;
}

function hideHeldPassword(target, key) {
  target[key] = false;
}

function resolveConfirmation(confirmed) {
  confirmDialog.visible = false;

  if (confirmDialogResolver) {
    const resolve = confirmDialogResolver;
    confirmDialogResolver = null;
    resolve(confirmed);
  }
}

function requestConfirmation(options = {}) {
  if (typeof window === "undefined") {
    return Promise.resolve(true);
  }

  const config =
    typeof options === "string"
      ? { message: options }
      : options && typeof options === "object"
        ? options
        : {};

  if (confirmDialogResolver) {
    resolveConfirmation(false);
  }

  confirmDialog.title = config.title || "Confirm action";
  confirmDialog.message = config.message || "Are you sure you want to continue?";
  confirmDialog.confirmLabel = config.confirmLabel || "Confirm";
  confirmDialog.cancelLabel = config.cancelLabel || "Cancel";
  confirmDialog.tone = config.tone || "danger";
  confirmDialog.visible = true;

  return new Promise((resolve) => {
    confirmDialogResolver = resolve;
  });
}

async function confirmDestructiveAction(message, options = {}) {
  return requestConfirmation({
    title: options.title || "Confirm deletion",
    message,
    confirmLabel: options.confirmLabel || "Delete",
    cancelLabel: options.cancelLabel || "Keep",
    tone: "danger"
  });
}

function isDrawerLinkActive(link) {
  if (ui.page !== link.page) {
    return false;
  }

  if (link.page !== "dashboard") {
    return true;
  }

  return link.tab ? ui.tab === link.tab : true;
}

async function login() {
  return runServerAction({
    pendingMessage: "Signing you in...",
    successMessage: "Signed in successfully.",
    errorMessage: "Unable to sign in.",
    task: async () => {
      const response = await model.loginSession();
      if (response?.ok) {
        await afterAuth();
      }
      return response;
    }
  });
}

async function registerUser() {
  if (model.workbench.session.password !== authState.confirmPassword) {
    showFeedback("warn", "The password confirmation does not match.", false);
    return null;
  }

  return runServerAction({
    pendingMessage: "Creating your account...",
    successMessage: "Account created and signed in.",
    errorMessage: "Unable to create the account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register");
      if (response?.ok) {
        await afterAuth();
      }
      return response;
    }
  });
}

async function registerAdmin() {
  if (model.workbench.session.password !== authState.confirmPassword) {
    showFeedback("warn", "The password confirmation does not match.", false);
    return null;
  }

  return runServerAction({
    pendingMessage: "Creating the admin account...",
    successMessage: "Admin account created and signed in.",
    errorMessage: "Unable to create the admin account.",
    task: async () => {
      const response = await model.runEndpoint("auth-register-admin");
      if (response?.ok) {
        await afterAuth();
      }
      return response;
    }
  });
}

async function logout() {
  signOutState.intentional = true;
  showFeedback("info", "Signing out and clearing local app data...", true);

  const result = await model.logoutSession();
  ui.page = "home";
  ui.tab = "overview";
  resetProtectedData();

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
          defaultCurrency: String(profileForm.defaultCurrency || "")
            .trim()
            .toUpperCase(),
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
    showFeedback("warn", "Select a variant before adding a watchlist item.", false);
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

  if (result?.ok) {
    await loadUserData();
  }
}

async function addAlert() {
  if (!model.workbench.lookup.variantId) {
    showFeedback("warn", "Select a variant before creating an alert.", false);
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

  if (result?.ok) {
    await loadUserData();
  }
}

async function addEmail() {
  const result = await runServerAction({
    pendingMessage: "Adding your email...",
    successMessage: "Email added.",
    errorMessage: "Unable to add the email.",
    task: () =>
      model.runEndpoint("emails-create", {
        body: {
          email: emailDraft.email,
          emailRole: emailDraft.emailRole,
          isLoginEnabled: emailDraft.isLoginEnabled
        }
      })
  });

  if (result?.ok) {
    emailDraft.email = "";
    emailDraft.emailRole = "SECONDARY";
    emailDraft.isLoginEnabled = true;
    await loadUserData();
  }
}

async function deleteEmail(emailId) {
  const targetEmail = data.emails.find((email) => email.id === emailId)?.email || "this email";
  if (
    !(await confirmDestructiveAction(`Delete ${targetEmail} from your account?`, {
      title: "Delete email?"
    }))
  ) {
    return null;
  }

  const result = await runServerAction({
    pendingMessage: "Removing the email...",
    successMessage: "Email removed.",
    errorMessage: "Unable to remove the email.",
    task: () => model.runEndpoint("emails-delete", { pathParams: { emailId } })
  });

  if (result?.ok) {
    data.emails = data.emails.filter((email) => email.id !== emailId);
    await loadUserData();
  }
}

async function verifyEmail(emailId) {
  const result = await runServerAction({
    pendingMessage: "Marking the email as verified...",
    successMessage: "Email verified.",
    errorMessage: "Unable to verify the email.",
    task: () =>
      model.runEndpoint("emails-verify", {
        pathParams: { emailId },
        body: { code: "123456" }
      })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

async function makePrimaryEmail(emailId) {
  const result = await runServerAction({
    pendingMessage: "Updating the primary email...",
    successMessage: "Primary email updated.",
    errorMessage: "Unable to update the primary email.",
    task: () => model.runEndpoint("emails-make-primary", { pathParams: { emailId } })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

async function addPhone() {
  const result = await runServerAction({
    pendingMessage: "Adding your phone number...",
    successMessage: "Phone number added.",
    errorMessage: "Unable to add the phone number.",
    task: () => model.runEndpoint("phones-create", { body: { phoneNumber: phoneDraft.phoneNumber } })
  });

  if (result?.ok) {
    phoneDraft.phoneNumber = "";
    await loadUserData();
  }
}

async function deletePhone(phoneId) {
  const targetPhone =
    data.phones.find((phone) => phone.id === phoneId)?.phoneNumber || "this phone number";
  if (
    !(await confirmDestructiveAction(`Delete ${targetPhone} from your account?`, {
      title: "Delete phone number?"
    }))
  ) {
    return null;
  }

  const result = await runServerAction({
    pendingMessage: "Removing the phone number...",
    successMessage: "Phone number removed.",
    errorMessage: "Unable to remove the phone number.",
    task: () => model.runEndpoint("phones-delete", { pathParams: { phoneId } })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

async function verifyPhone(phoneId) {
  const result = await runServerAction({
    pendingMessage: "Marking the phone as verified...",
    successMessage: "Phone number verified.",
    errorMessage: "Unable to verify the phone number.",
    task: () =>
      model.runEndpoint("phones-verify", {
        pathParams: { phoneId },
        body: { code: "123456" }
      })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

async function makePrimaryPhone(phoneId) {
  const result = await runServerAction({
    pendingMessage: "Updating the primary phone...",
    successMessage: "Primary phone updated.",
    errorMessage: "Unable to update the primary phone.",
    task: () => model.runEndpoint("phones-make-primary", { pathParams: { phoneId } })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

async function changePassword() {
  if (!passwordForm.currentPassword || !passwordForm.newPassword) {
    showFeedback("warn", "Enter both your current and new password.", false);
    return;
  }

  if (passwordForm.newPassword !== passwordForm.confirmPassword) {
    showFeedback("warn", "The new password confirmation does not match.", false);
    return;
  }

  const result = await runServerAction({
    pendingMessage: "Updating your password...",
    successMessage: "Password updated.",
    errorMessage: "Unable to change the password.",
    task: () =>
      model.runEndpoint("auth-password-change", {
        body: {
          currentPassword: passwordForm.currentPassword,
          newPassword: passwordForm.newPassword
        }
      })
  });

  if (result?.ok) {
    passwordForm.currentPassword = "";
    passwordForm.newPassword = "";
    passwordForm.confirmPassword = "";
    await loadUserData();
  }
}

async function saveCookiePreferences() {
  const result = await runServerAction({
    pendingMessage: "Saving cookie preferences...",
    successMessage: "Cookie preferences updated.",
    errorMessage: "Unable to save cookie preferences.",
    task: () =>
      model.runEndpoint("cookie-preferences-update", {
        body: {
          analytics: data.cookiePreferences.analytics,
          marketing: data.cookiePreferences.marketing,
          preferences: data.cookiePreferences.preferences
        }
      })
  });

  if (result?.ok) {
    await loadUserData();
  }
}

function pickAvatar(event) {
  const file = event.target?.files?.[0] || null;
  avatarState.selectedFile = file;
  avatarState.selectedFileName = file?.name || "";
}

async function uploadAvatar() {
  if (!avatarState.selectedFile) {
    showFeedback("warn", "Choose an image before uploading an avatar.", false);
    return;
  }

  const file = avatarState.selectedFile;
  let uploadedAvatarFileId = "";
  const result = await runServerAction({
    pendingMessage: "Uploading and attaching your avatar...",
    successMessage: "Avatar updated.",
    errorMessage: "Unable to update the avatar.",
    task: async () => {
      const uploadIntent = await model.runEndpoint("file-upload-intent", {
        body: {
          filename: file.name,
          contentType: file.type || "application/octet-stream",
          size: file.size || 0,
          purpose: "PROFILE_AVATAR",
          checksumSha256: ""
        }
      });

      if (!uploadIntent?.ok) {
        return uploadIntent;
      }

      const fileId = uploadIntent.data?.data?.fileId;
      uploadedAvatarFileId = fileId || "";
      const completed = await model.runEndpoint("file-upload-complete", {
        pathParams: { fileId }
      });

      if (!completed?.ok) {
        return completed;
      }

      return model.runEndpoint("me-avatar-update", {
        body: { fileId }
      });
    }
  });

  if (result?.ok) {
    const previewDataUrl = await readFileAsDataUrl(file).catch(() => "");
    if (previewDataUrl && uploadedAvatarFileId && model.workbench.session.accountId) {
      persistAvatarPreview({
        accountId: model.workbench.session.accountId,
        fileId: uploadedAvatarFileId,
        filename: file.name,
        dataUrl: previewDataUrl
      });
    }
    avatarState.selectedFile = null;
    avatarState.selectedFileName = "";
    await loadUserData();
  }
}

async function removeAvatar() {
  if (
    !(await confirmDestructiveAction("Remove your current avatar?", {
      title: "Remove avatar?",
      confirmLabel: "Remove"
    }))
  ) {
    return null;
  }

  const result = await runServerAction({
    pendingMessage: "Removing your avatar...",
    successMessage: "Avatar removed.",
    errorMessage: "Unable to remove the avatar.",
    task: () => model.runEndpoint("me-avatar-delete")
  });

  if (result?.ok) {
    clearAvatarPreview(model.workbench.session.accountId);
    await loadUserData();
  }
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

async function createAdminUser(payload) {
  return runServerAction({
    pendingMessage: "Creating the user account...",
    successMessage: "User created.",
    errorMessage: "Unable to create the user account.",
    task: async () => {
      const result = await model.createAdminUser(payload);
      if (result?.ok) {
        await Promise.all([
          model.loadAdminUsers(),
          model.requestEndpoint("admin-overview", {}, { recordHistory: false })
        ]);
      }
      return result;
    }
  });
}

async function updateAdminUser(accountId, payload) {
  return runServerAction({
    pendingMessage: "Saving user changes...",
    successMessage: "User updated.",
    errorMessage: "Unable to update the user.",
    task: async () => {
      const result = await model.updateAdminUser(accountId, payload);
      if (result?.ok) {
        await Promise.all([
          model.loadAdminUsers(),
          model.requestEndpoint("admin-overview", {}, { recordHistory: false })
        ]);
      }
      return result;
    }
  });
}

async function applyAdminUserAction(payload) {
  if (!Array.isArray(payload.accountIds) || !payload.accountIds.length) {
    showFeedback("warn", "Select at least one user before applying an admin action.", false);
    return null;
  }

  return runServerAction({
    pendingMessage: "Applying the admin user action...",
    successMessage: "Admin user action completed.",
    errorMessage: "Unable to apply the admin user action.",
    task: async () => {
      const result = await model.applyAdminUserBulkAction(payload);
      if (result?.ok) {
        await Promise.all([
          model.loadAdminUsers(),
          model.requestEndpoint("admin-overview", {}, { recordHistory: false })
        ]);
      }
      return result;
    }
  });
}

async function saveAdminRecordDraft(payload) {
  const tableLabel =
    model.adminTables.find((table) => table.id === payload.tableId)?.label || "record";

  return runServerAction({
    pendingMessage: `Saving ${tableLabel.toLowerCase()} changes...`,
    successMessage: `${tableLabel} saved.`,
    errorMessage: `Unable to save the ${tableLabel.toLowerCase()} record.`,
    task: () => model.saveAdminRecordDraft(payload)
  });
}

async function deleteAdminRecord(payload) {
  const tableLabel =
    model.adminTables.find((table) => table.id === payload.tableId)?.label || "record";

  return runServerAction({
    pendingMessage: `Deleting the ${tableLabel.toLowerCase()} record...`,
    successMessage: `${tableLabel} record updated.`,
    errorMessage: `Unable to delete the ${tableLabel.toLowerCase()} record.`,
    task: () => model.deleteAdminRecord(payload)
  });
}

async function approveAdminRecord(payload) {
  const tableLabel =
    model.adminTables.find((table) => table.id === payload.tableId)?.label || "record";

  return runServerAction({
    pendingMessage: `Approving the ${tableLabel.toLowerCase()} record...`,
    successMessage: `${tableLabel} approved.`,
    errorMessage: `Unable to approve the ${tableLabel.toLowerCase()} record.`,
    task: () => model.approveAdminRecord(payload)
  });
}

async function bulkDeleteAdminRecords({ tableId, recordIds = [] }) {
  const uniqueRecordIds = Array.from(new Set(recordIds.filter(Boolean)));
  if (!tableId || !uniqueRecordIds.length) {
    showFeedback("warn", "Select at least one record before deleting.", false);
    return null;
  }

  const tableLabel =
    model.adminTables.find((table) => table.id === tableId)?.label || "record";
  const pluralSuffix = uniqueRecordIds.length === 1 ? "" : "s";

  return runServerAction({
    pendingMessage: `Deleting ${uniqueRecordIds.length} ${tableLabel.toLowerCase()} record${pluralSuffix}...`,
    successMessage: `${tableLabel} selection updated.`,
    errorMessage: `Unable to delete the selected ${tableLabel.toLowerCase()} records.`,
    task: async () => {
      let lastResult = null;

      for (const recordId of uniqueRecordIds) {
        lastResult = await model.deleteAdminRecord({ tableId, recordId });
        if (!lastResult?.ok) {
          return lastResult;
        }
      }

      return (
        lastResult || {
          ok: true,
          status: 200,
          data: { data: null }
        }
      );
    }
  });
}

async function bulkApproveAdminRecords({ tableId, recordIds = [] }) {
  const uniqueRecordIds = Array.from(new Set(recordIds.filter(Boolean)));
  if (!tableId || !uniqueRecordIds.length) {
    showFeedback("warn", "Select at least one pending record before approving.", false);
    return null;
  }

  const tableLabel =
    model.adminTables.find((table) => table.id === tableId)?.label || "record";
  const pluralSuffix = uniqueRecordIds.length === 1 ? "" : "s";

  return runServerAction({
    pendingMessage: `Approving ${uniqueRecordIds.length} ${tableLabel.toLowerCase()} record${pluralSuffix}...`,
    successMessage: `${tableLabel} selection approved.`,
    errorMessage: `Unable to approve the selected ${tableLabel.toLowerCase()} records.`,
    task: async () => {
      let lastResult = null;

      for (const recordId of uniqueRecordIds) {
        lastResult = await model.approveAdminRecord({ tableId, recordId, refresh: false });
        if (!lastResult?.ok) {
          return lastResult;
        }
      }

      await Promise.all([
        model.requestEndpoint("admin-overview", {}, { recordHistory: false }),
        model.refreshAdminTable()
      ]);

      return (
        lastResult || {
          ok: true,
          status: 200,
          data: { data: null }
        }
      );
    }
  });
}

function exitAdminMode() {
  ui.page = isLoggedIn.value ? "dashboard" : "home";
  ui.tab = "overview";
}

watch(
  () => model.workbench.lookup.itemId,
  (itemId) => {
    void loadVariants(itemId);
  }
);

watch(isLoggedIn, (loggedIn, wasLoggedIn) => {
  if (loggedIn) {
    return;
  }

  resetProtectedData();
  clearAvatarPreview();

  if (ui.page === "dashboard" || ui.page === "admin") {
    ui.page = "home";
    ui.tab = "overview";
  }

  if (wasLoggedIn && !signOutState.intentional) {
    showFeedback(
      "warn",
      "Your session is no longer valid. Protected local data was cleared. Please sign in again.",
      false
    );
  }
});

watch(
  () => model.isAdmin,
  (isAdminNow) => {
    if (!isAdminNow) {
      data.settings = [];

      if (ui.page === "admin") {
        ui.page = isLoggedIn.value ? "dashboard" : "auth";
        ui.tab = "overview";
      }
    }
  }
);

onMounted(() => {
  restoreAvatarPreview();
  void refreshDashboard();
});

onBeforeUnmount(() => {
  clearFeedback();
});
</script>

<template>
  <ConfirmDialog
    :visible="confirmDialog.visible"
    :title="confirmDialog.title"
    :message="confirmDialog.message"
    :confirm-label="confirmDialog.confirmLabel"
    :cancel-label="confirmDialog.cancelLabel"
    :tone="confirmDialog.tone"
    @confirm="resolveConfirmation(true)"
    @cancel="resolveConfirmation(false)"
  />

  <div v-if="feedback.visible" class="ptfb">
    <n-alert :type="feedbackAlertType" :title="feedback.pending ? '请求中' : '提示'" closable @close="clearFeedback">
      <n-space v-if="feedback.pending" align="center" size="small">
        <n-spin size="small" />
        <span>{{ feedback.message }}</span>
      </n-space>
      <span v-else>{{ feedback.message }}</span>
    </n-alert>
  </div>

  <AdminPortalPage
    v-if="ui.page === 'admin' && model.isAdmin"
    :model="model"
    :settings="data.settings"
    :loading-settings="data.loadingSettings"
    :saving-setting="data.savingSetting"
    :disabled="model.isBusy"
    :feedback="feedback"
    :hidden-modules-enabled="hiddenModulesEnabled"
    :maintenance-mode="maintenanceMode"
    :refresh-all="refreshAdminPage"
    :refresh-settings="loadSettings"
    :create-user="createAdminUser"
    :update-user="updateAdminUser"
    :apply-user-action="applyAdminUserAction"
    :save-setting="saveSetting"
    :save-admin-record="saveAdminRecordDraft"
    :delete-admin-record="deleteAdminRecord"
    :approve-admin-record="approveAdminRecord"
    :bulk-delete-admin-records="bulkDeleteAdminRecords"
    :bulk-approve-admin-records="bulkApproveAdminRecords"
    :confirm-action="requestConfirmation"
    @exit="exitAdminMode"
  />

  <n-layout v-else position="absolute" style="inset: 0; height: 100vh" has-sider>
    <n-layout-sider
      v-if="ui.menuPinned"
      bordered
      show-trigger
      collapse-mode="width"
      :width="260"
      :native-scrollbar="false"
      content-style="padding: 12px"
    >
      <n-space vertical size="large">
        <n-text strong>导航</n-text>
        <n-button
          v-for="link in drawerLinks"
          :key="link.label"
          block
          quaternary
          :type="isDrawerLinkActive(link) ? 'primary' : 'default'"
          @click="openPage(link.page, link.tab)"
        >
          <n-space vertical align="start" size="small">
            <n-text>{{ link.label }}</n-text>
            <n-text depth="3" style="font-size: 12px">{{ link.description }}</n-text>
          </n-space>
        </n-button>
        <n-text depth="3" style="font-size: 12px">管理员入口在顶部；仅管理员可进入独立后台。</n-text>
      </n-space>
    </n-layout-sider>

    <n-layout>
      <n-layout-header bordered style="height: 64px; padding: 0 20px; display: flex; align-items: center; gap: 16px; flex-wrap: wrap">
        <n-button quaternary size="small" @click="togglePinnedMenu">
          {{ ui.menuPinned ? "收起侧栏" : "固定侧栏" }}
        </n-button>
        <n-space vertical size="small">
          <n-text depth="3" style="font-size: 11px">发布版</n-text>
          <n-text tag="h1" style="margin: 0; font-size: 1.2rem">PriceTracker</n-text>
        </n-space>
        <n-space size="small" style="flex: 1; justify-content: center" wrap>
          <n-tag v-for="light in model.statusLights" :key="light.key" size="small" :type="statusTagType(light.tone)">
            {{ light.label }}: {{ light.state }}
          </n-tag>
        </n-space>
        <n-space>
          <template v-if="isLoggedIn">
            <n-button quaternary @click="openPage('dashboard', 'overview')">
              <n-space align="center" size="small">
                <n-avatar round size="small" :src="accountAvatarPreviewUrl || undefined">
                  {{ accountAvatarInitials }}
                </n-avatar>
                <n-space vertical size="small">
                  <n-text strong style="font-size: 13px">{{ accountDisplayName }}</n-text>
                  <n-text depth="3" style="font-size: 11px">{{ accountRoleLabel }}</n-text>
                </n-space>
              </n-space>
            </n-button>
            <n-button v-if="model.isAdmin" size="small" @click="openPage('admin')">管理后台</n-button>
            <n-button size="small" :disabled="model.isBusy" @click="logout">{{ feedback.pending ? "退出中…" : "退出" }}</n-button>
          </template>
          <n-button v-else type="primary" size="small" @click="openPage('auth')">登录 / 注册</n-button>
        </n-space>
      </n-layout-header>

      <n-layout-content content-style="padding: 20px 24px 32px; overflow: auto; max-width: 1280px; margin: 0 auto; width: 100%">
        <n-space v-if="ui.page === 'home'" vertical size="large" style="width: 100%">
          <n-card title="从这里开始" :segmented="{ content: true }">
            <n-grid :cols="2" :x-gap="16" responsive="screen">
              <n-gi span="2 m:1">
                <n-text depth="2" style="display: block; margin-bottom: 12px">
                  公共首页按角色解锁工具；侧栏可固定；管理员使用独立门户。
                </n-text>
                <n-space>
                  <n-button type="primary" @click="openPage(isLoggedIn ? 'dashboard' : 'auth')">
                    {{ isLoggedIn ? "打开工作台" : "开始使用" }}
                  </n-button>
                  <n-button quaternary @click="model.runHealthCheck">健康检查</n-button>
                </n-space>
              </n-gi>
              <n-gi span="2 m:1">
                <n-card v-if="isLoggedIn" size="small" embedded title="当前账户">
                  <n-space align="center">
                    <n-avatar round :src="accountAvatarPreviewUrl || undefined" :size="48">{{ accountAvatarInitials }}</n-avatar>
                    <n-space vertical>
                      <n-text strong>{{ accountDisplayName }}</n-text>
                      <n-text depth="3">{{ accountAvatarLabel }}</n-text>
                      <n-space>
                        <n-tag size="small" :type="accountAvatarPreviewUrl ? 'success' : 'info'">
                          {{ accountAvatarPreviewUrl ? "头像预览就绪" : "首字母头像" }}
                        </n-tag>
                        <n-tag size="small" type="warning">{{ accountRoleLabel }}</n-tag>
                      </n-space>
                    </n-space>
                  </n-space>
                </n-card>
                <n-grid :cols="2" :x-gap="12" :y-gap="12" style="margin-top: 12px">
                  <n-gi v-for="card in model.summaryCards" :key="card.label" span="2 m:1">
                    <n-card size="small" embedded>
                      <n-statistic :label="card.label" :value="card.value" />
                    </n-card>
                  </n-gi>
                </n-grid>
              </n-gi>
            </n-grid>
          </n-card>

          <n-grid :cols="3" :x-gap="12" responsive="screen">
            <n-gi v-for="light in model.statusLights" :key="light.key" span="3 m:1">
              <n-card size="small" embedded>
                <n-statistic :label="light.label" :value="light.state">
                  <template #suffix>
                    <n-tag size="tiny" :type="statusTagType(light.tone)">状态</n-tag>
                  </template>
                </n-statistic>
                <n-text depth="3" style="display: block; margin-top: 8px">{{ light.detail }}</n-text>
              </n-card>
            </n-gi>
          </n-grid>
        </n-space>

        <n-card v-else-if="ui.page === 'auth'" title="账户" :segmented="{ content: true }">
          <n-tabs v-model:value="ui.authMode" type="line">
            <n-tab-pane name="login" tab="登录" />
            <n-tab-pane name="register" tab="注册" />
            <n-tab-pane name="admin" tab="管理员引导" />
          </n-tabs>

          <n-form style="margin-top: 16px" label-placement="top">
            <n-form-item label="邮箱">
              <n-input v-model:value="model.workbench.session.email" placeholder="登录邮箱" />
            </n-form-item>
            <n-form-item label="密码">
              <n-input
                v-model:value="model.workbench.session.password"
                type="password"
                show-password-on="mousedown"
                :placeholder="ui.authMode === 'login' ? '登录密码' : '至少 12 位'"
              />
            </n-form-item>
            <template v-if="ui.authMode !== 'login'">
              <n-form-item label="显示名称">
                <n-input v-model:value="model.workbench.session.displayName" />
              </n-form-item>
              <n-form-item label="确认密码">
                <n-input v-model:value="authState.confirmPassword" type="password" show-password-on="mousedown" />
              </n-form-item>
              <n-form-item label="手机（可选）">
                <n-input v-model:value="model.workbench.session.primaryPhone" />
              </n-form-item>
            </template>
          </n-form>

          <n-space style="margin-top: 16px" wrap>
            <n-button v-if="ui.authMode === 'login'" type="primary" :disabled="model.isBusy" @click="login">
              {{ model.isBusy ? "登录中…" : "登录" }}
            </n-button>
            <n-button v-else-if="ui.authMode === 'register'" type="primary" :disabled="model.isBusy" @click="registerUser">
              {{ model.isBusy ? "创建中…" : "注册用户" }}
            </n-button>
            <n-button v-else type="primary" :disabled="model.isBusy" @click="registerAdmin">
              {{ model.isBusy ? "创建中…" : "创建管理员" }}
            </n-button>
            <n-button quaternary :disabled="model.isBusy" @click="model.fillDemoSession">填入演示账号</n-button>
            <n-button quaternary :disabled="model.isBusy" @click="model.fillAdminSession">填入种子管理员</n-button>
          </n-space>

          <n-alert v-if="ui.authMode === 'admin'" type="warning" title="临时能力" style="margin-top: 16px">
            上线前请关闭或限制管理员自助注册。
          </n-alert>
        </n-card>

        <n-card v-else-if="!isLoggedIn" title="需要登录">
          <n-empty description="请先登录以使用工作台">
            <template #extra>
              <n-button type="primary" @click="openPage('auth')">去登录</n-button>
            </template>
          </n-empty>
        </n-card>

        <n-space v-else vertical size="large" style="width: 100%">
          <n-card size="small" embedded>
            <n-space justify="space-between" align="center" wrap>
              <n-space vertical>
                <n-text depth="3">工作台</n-text>
                <n-text tag="h2" style="margin: 0; font-size: 1.1rem">
                  {{ model.workbench.session.displayName || model.workbench.session.email }}
                </n-text>
                <n-text depth="3">
                  {{ model.isAdmin ? "日常用户工具在此；深度运维请进管理后台。" : "已解锁用户功能。" }}
                </n-text>
              </n-space>
              <n-space>
                <n-button @click="refreshDashboard">刷新</n-button>
                <n-button quaternary @click="model.runHealthCheck">健康检查</n-button>
                <n-button v-if="model.isAdmin" quaternary @click="openPage('admin')">管理后台</n-button>
              </n-space>
            </n-space>
          </n-card>

          <n-tabs v-model:value="ui.tab" type="segment">
            <n-tab-pane name="overview" tab="总览" />
            <n-tab-pane name="capture" tab="采集" />
            <n-tab-pane name="activity" tab="活动" />
          </n-tabs>

          <template v-if="ui.tab === 'overview'">
            <UserAccountPanel
              :profile="data.profile"
              :loading="data.loadingUser"
              :busy="model.isBusy"
              :profile-form="profileForm"
              :email-draft="emailDraft"
              :phone-draft="phoneDraft"
              :password-form="passwordForm"
              :avatar-state="avatarState"
              :cookie-preferences="data.cookiePreferences"
              :emails="data.emails"
              :phones="data.phones"
              @refresh="loadUserData"
              @save-profile="saveProfile"
              @pick-avatar="pickAvatar"
              @upload-avatar="uploadAvatar"
              @remove-avatar="removeAvatar"
              @add-email="addEmail"
              @delete-email="deleteEmail"
              @verify-email="verifyEmail"
              @make-primary-email="makePrimaryEmail"
              @add-phone="addPhone"
              @delete-phone="deletePhone"
              @verify-phone="verifyPhone"
              @make-primary-phone="makePrimaryPhone"
              @change-password="changePassword"
              @save-cookie-preferences="saveCookiePreferences"
            />

            <n-card title="当前上下文" size="small" embedded style="margin-top: 16px">
              <n-descriptions bordered size="small" :column="1">
                <n-descriptions-item label="商店">{{ selectedShop?.name || '未选择' }}</n-descriptions-item>
                <n-descriptions-item label="商品">{{ selectedItem?.name || '未选择' }}</n-descriptions-item>
                <n-descriptions-item label="变体 ID">{{ model.workbench.lookup.variantId || '未选择' }}</n-descriptions-item>
                <n-descriptions-item label="维护模式">{{ maintenanceMode ? '开' : '关' }}</n-descriptions-item>
                <n-descriptions-item label="头像">{{ data.profile?.avatarFilename || data.profile?.avatarFileId || '无' }}</n-descriptions-item>
              </n-descriptions>
            </n-card>
          </template>

          <template v-else-if="ui.tab === 'capture'">
            <n-alert v-if="maintenanceMode" type="warning" title="维护模式" style="margin-bottom: 12px">
              维护中，提交类操作请谨慎。
            </n-alert>

            <n-space vertical size="large">
              <ScannerPanel
                :model="model.workbench.capture"
                :shop-id="model.workbench.lookup.shopId"
                :disabled="model.isBusy || maintenanceMode"
                @capture="model.handleCapturedCode"
                @lookup="model.lookupCapturedCode"
              />

              <n-card title="目录与选择" :segmented="{ content: true }">
                <template #header-extra>
                  <n-button size="small" quaternary :disabled="data.loadingPublic" @click="loadPublicData">刷新目录</n-button>
                </template>
                <n-grid :cols="2" :x-gap="12" responsive="screen">
                  <n-gi span="2">
                    <n-form-item label="搜索商品">
                      <n-input v-model:value="ui.search" placeholder="回车刷新" @keyup.enter.prevent="loadPublicData" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="商店">
                      <n-select
                        v-model:value="model.workbench.lookup.shopId"
                        :options="[{ label: '请选择', value: '' }, ...data.shops.map((s) => ({ label: s.name, value: s.id }))]"
                        filterable
                      />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="商品">
                      <n-select
                        v-model:value="model.workbench.lookup.itemId"
                        :options="[{ label: '请选择', value: '' }, ...data.items.map((i) => ({ label: i.name, value: i.id }))]"
                        filterable
                      />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2">
                    <n-form-item label="变体">
                      <n-select
                        v-model:value="model.workbench.lookup.variantId"
                        :options="[{ label: '请选择', value: '' }, ...data.variants.map((v) => ({ label: v.variantName || v.id, value: v.id }))]"
                        filterable
                      />
                    </n-form-item>
                  </n-gi>
                </n-grid>
                <n-space style="margin-top: 8px" wrap>
                  <n-button quaternary :disabled="model.isBusy" @click="model.lookupCapturedCode">查询编码</n-button>
                  <n-button quaternary :disabled="model.isBusy" @click="model.loadVariantDetail">变体详情</n-button>
                  <n-button quaternary :disabled="model.isBusy" @click="model.compareVariant">对比</n-button>
                </n-space>
              </n-card>

              <n-card title="采购与价格" :segmented="{ content: true }">
                <n-grid :cols="2" :x-gap="12" responsive="screen">
                  <n-gi span="2 m:1">
                    <n-form-item label="采购时间">
                      <input v-model="model.workbench.submission.purchaseTime" class="native-dt" type="datetime-local" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="记录时间">
                      <input v-model="model.workbench.submission.recordedAt" class="native-dt" type="datetime-local" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="金额">
                      <n-input v-model:value="model.workbench.submission.originalAmount" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="货币">
                      <n-input v-model:value="model.workbench.submission.originalCurrency" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2">
                    <n-form-item label="附件 ID">
                      <n-input v-model:value="model.workbench.submission.attachmentFileIds" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2">
                    <n-form-item label="备注">
                      <n-input v-model:value="model.workbench.submission.priceNotes" type="textarea" :rows="3" />
                    </n-form-item>
                  </n-gi>
                </n-grid>
                <n-space style="margin-top: 8px" wrap>
                  <n-button type="primary" :disabled="model.isBusy || maintenanceMode" @click="model.createPurchase">创建采购</n-button>
                  <n-button :disabled="model.isBusy || maintenanceMode" @click="model.submitPrice">提交价格</n-button>
                  <n-button quaternary :disabled="model.isBusy" @click="model.saveFullCaptureForRetry">保存捕获</n-button>
                </n-space>
              </n-card>

              <n-card title="收据文件（演示字段）" :segmented="{ content: true }">
                <n-grid :cols="2" :x-gap="12" responsive="screen">
                  <n-gi span="2 m:1">
                    <n-form-item label="文件名">
                      <n-input v-model:value="model.workbench.submission.fileUpload.filename" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="类型">
                      <n-input v-model:value="model.workbench.submission.fileUpload.contentType" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="大小">
                      <n-input v-model:value="model.workbench.submission.fileUpload.size" />
                    </n-form-item>
                  </n-gi>
                  <n-gi span="2 m:1">
                    <n-form-item label="用途">
                      <n-input v-model:value="model.workbench.submission.fileUpload.purpose" />
                    </n-form-item>
                  </n-gi>
                </n-grid>
                <n-space style="margin-top: 8px">
                  <n-button :disabled="model.isBusy" @click="model.createUploadIntent">创建上传意图</n-button>
                  <n-button quaternary :disabled="model.isBusy" @click="model.completeUploadIntent">完成上传</n-button>
                </n-space>
              </n-card>
            </n-space>
          </template>

          <template v-else-if="ui.tab === 'activity'">
            <n-space vertical size="large">
              <n-card title="我的数据概览" :segmented="{ content: true }">
                <n-descriptions bordered size="small" :column="2">
                  <n-descriptions-item label="采购">{{ data.purchases.length }}</n-descriptions-item>
                  <n-descriptions-item label="价格">{{ data.prices.length }}</n-descriptions-item>
                  <n-descriptions-item label="关注">{{ data.watchlist.length }}</n-descriptions-item>
                  <n-descriptions-item label="提醒">{{ data.alerts.length }}</n-descriptions-item>
                </n-descriptions>
                <n-grid :cols="2" :x-gap="12" style="margin-top: 12px">
                  <n-gi>
                    <n-form-item label="提醒金额">
                      <n-input v-model:value="ui.alertAmount" />
                    </n-form-item>
                  </n-gi>
                  <n-gi>
                    <n-form-item label="货币">
                      <n-input v-model:value="ui.alertCurrency" />
                    </n-form-item>
                  </n-gi>
                </n-grid>
                <n-checkbox v-model:checked="ui.alertEnabled">新建提醒时默认启用</n-checkbox>
                <n-space style="margin-top: 12px">
                  <n-button :disabled="model.isBusy" @click="addWatchlist">{{ model.isBusy ? '保存中…' : '关注当前变体' }}</n-button>
                  <n-button quaternary :disabled="model.isBusy" @click="addAlert">{{ model.isBusy ? '保存中…' : '创建提醒' }}</n-button>
                </n-space>
              </n-card>

              <QueuePanel
                :queue="model.queue"
                :online="model.online"
                :flushing="model.isFlushingQueue"
                :confirm-action="requestConfirmation"
                @flush="model.flushQueue"
                @retry="model.retryQueueEntry"
                @remove="model.removeQueueEntry"
                @clear="model.clearQueue"
              />

              <ResponsePanel :history="model.history" />
            </n-space>
          </template>
        </n-space>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<style scoped>
.ptfb {
  position: fixed;
  top: 12px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 4000;
  width: min(560px, 92vw);
}
.native-dt {
  width: 100%;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  font: inherit;
}
</style>

