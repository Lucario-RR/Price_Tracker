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

  <transition name="feedback-drop">
    <div
      v-if="feedback.visible"
      class="feedback-layer"
      aria-live="polite"
      aria-atomic="true"
    >
      <section
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
    </div>
  </transition>

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

  <div v-else :class="['app-shell', ui.menuPinned ? 'has-sidebar' : '']">
    <aside v-if="ui.menuPinned" class="nav-drawer panel">
      <div class="drawer-head">
        <div>
          <p class="eyebrow">PriceTracker</p>
          <h2>Navigation</h2>
        </div>
        <button class="ghost-icon-button" @click="togglePinnedMenu">Hide</button>
      </div>

      <button
        v-for="link in drawerLinks"
        :key="link.label"
        :class="['drawer-link', isDrawerLinkActive(link) ? 'is-active' : '']"
        @click="openPage(link.page, link.tab)"
      >
        <strong>{{ link.label }}</strong>
        <span>{{ link.description }}</span>
      </button>

      <p class="helper-text">
        Admin tools stay outside this menu. Only signed-in admins can open the separate admin portal.
      </p>
    </aside>

    <div class="app-main">
      <header class="topbar">
        <div class="topbar-left">
          <button class="menu-button" @click="togglePinnedMenu">
            {{ ui.menuPinned ? "Hide menu" : "Keep menu open" }}
          </button>
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
              <span class="avatar-shell account-chip-avatar" aria-hidden="true">
                <img
                  v-if="accountAvatarPreviewUrl"
                  :src="accountAvatarPreviewUrl"
                  :alt="`${accountDisplayName} avatar`"
                />
                <span v-else>{{ accountAvatarInitials }}</span>
              </span>
              <span class="account-chip-copy">
                <strong>{{ accountDisplayName }}</strong>
                <span>{{ accountRoleLabel }}</span>
              </span>
            </button>
            <button v-if="model.isAdmin" class="ghost-button" @click="openPage('admin')">
              Admin mode
            </button>
            <button class="ghost-button" :disabled="model.isBusy" @click="logout">
              {{ feedback.pending ? "Signing out..." : "Sign out" }}
            </button>
          </template>
          <button v-else class="primary-button" @click="openPage('auth')">Login / Register</button>
        </div>
      </header>

      <main class="page-shell">
        <section v-if="ui.page === 'home'" class="stack-lg">
          <section class="hero-card">
            <div>
              <p class="eyebrow">Dashboard-first frontend</p>
              <h2>Start from the public home page, then unlock only the tools that match the signed-in role.</h2>
              <p class="helper-text">
                The left menu can stay pinned for daily use, regular users keep a focused workspace, and admin operations now live in a separate portal.
              </p>
              <div class="action-row wrap">
                <button class="primary-button" @click="openPage(isLoggedIn ? 'dashboard' : 'auth')">
                  {{ isLoggedIn ? "Open dashboard" : "Get started" }}
                </button>
                <button class="ghost-button" @click="model.runHealthCheck">Check health</button>
              </div>
            </div>

            <div class="home-side-stack">
              <article v-if="isLoggedIn" class="home-account-card">
                <div class="home-account-head">
                  <span class="avatar-shell home-account-avatar" aria-hidden="true">
                    <img
                      v-if="accountAvatarPreviewUrl"
                      :src="accountAvatarPreviewUrl"
                      :alt="`${accountDisplayName} avatar`"
                    />
                    <span v-else>{{ accountAvatarInitials }}</span>
                  </span>
                  <div class="home-account-copy">
                    <p class="eyebrow">Signed in</p>
                    <h3>{{ accountDisplayName }}</h3>
                    <p class="helper-text">{{ accountAvatarLabel }}</p>
                  </div>
                </div>

                <div class="pill-row">
                  <span :class="['status-pill', accountAvatarPreviewUrl ? 'tone-good' : 'tone-info']">
                    {{ accountAvatarPreviewUrl ? "Avatar preview ready" : "Avatar initials fallback" }}
                  </span>
                  <span class="status-pill tone-accent">{{ accountRoleLabel }}</span>
                </div>
              </article>

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

            <div class="field-grid">
              <label class="field">
                <span>Email</span>
                <input v-model="model.workbench.session.email" />
                <small class="field-note">Required. Use your sign-in email address.</small>
              </label>
              <label class="field">
                <span>Password</span>
                <div class="password-field">
                  <input
                    v-model="model.workbench.session.password"
                    :type="authState.showPassword ? 'text' : 'password'"
                  />
                  <button
                    class="ghost-icon-button password-toggle"
                    type="button"
                    aria-label="Hold to show password"
                    title="Hold to show password"
                    @pointerdown.prevent="showHeldPassword(authState, 'showPassword')"
                    @pointerup="hideHeldPassword(authState, 'showPassword')"
                    @pointerleave="hideHeldPassword(authState, 'showPassword')"
                    @pointercancel="hideHeldPassword(authState, 'showPassword')"
                    @blur="hideHeldPassword(authState, 'showPassword')"
                    @keydown.space.prevent="showHeldPassword(authState, 'showPassword')"
                    @keyup.space="hideHeldPassword(authState, 'showPassword')"
                    @keydown.enter.prevent="showHeldPassword(authState, 'showPassword')"
                    @keyup.enter="hideHeldPassword(authState, 'showPassword')"
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <path
                        d="M2.2 12s3.6-6 9.8-6 9.8 6 9.8 6-3.6 6-9.8 6-9.8-6-9.8-6Z"
                        fill="none"
                        stroke="currentColor"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.8"
                      />
                      <circle
                        cx="12"
                        cy="12"
                        r="3.1"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                      />
                    </svg>
                  </button>
                </div>
                <small class="field-note">
                  {{ ui.authMode === "login" ? "Required for sign-in." : "Required. Use at least 12 characters." }}
                </small>
              </label>
              <label v-if="ui.authMode !== 'login'" class="field">
                <span>Display name</span>
                <input v-model="model.workbench.session.displayName" />
                <small class="field-note">Required. This name is shown in the dashboard.</small>
              </label>
              <label v-if="ui.authMode !== 'login'" class="field">
                <span>Confirm password</span>
                <div class="password-field">
                  <input
                    v-model="authState.confirmPassword"
                    :type="authState.showConfirmPassword ? 'text' : 'password'"
                  />
                  <button
                    class="ghost-icon-button password-toggle"
                    type="button"
                    aria-label="Hold to show password confirmation"
                    title="Hold to show password confirmation"
                    @pointerdown.prevent="showHeldPassword(authState, 'showConfirmPassword')"
                    @pointerup="hideHeldPassword(authState, 'showConfirmPassword')"
                    @pointerleave="hideHeldPassword(authState, 'showConfirmPassword')"
                    @pointercancel="hideHeldPassword(authState, 'showConfirmPassword')"
                    @blur="hideHeldPassword(authState, 'showConfirmPassword')"
                    @keydown.space.prevent="showHeldPassword(authState, 'showConfirmPassword')"
                    @keyup.space="hideHeldPassword(authState, 'showConfirmPassword')"
                    @keydown.enter.prevent="showHeldPassword(authState, 'showConfirmPassword')"
                    @keyup.enter="hideHeldPassword(authState, 'showConfirmPassword')"
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <path
                        d="M2.2 12s3.6-6 9.8-6 9.8 6 9.8 6-3.6 6-9.8 6-9.8-6-9.8-6Z"
                        fill="none"
                        stroke="currentColor"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.8"
                      />
                      <circle
                        cx="12"
                        cy="12"
                        r="3.1"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                      />
                    </svg>
                  </button>
                </div>
                <small class="field-note">Required. Enter the same password again for validation.</small>
              </label>
              <label v-if="ui.authMode !== 'login'" class="field">
                <span>Primary phone</span>
                <input v-model="model.workbench.session.primaryPhone" />
                <small class="field-note">Optional. Include the country code if you want phone login later.</small>
              </label>
            </div>

            <div class="action-row wrap">
              <button v-if="ui.authMode === 'login'" :disabled="model.isBusy" @click="login">
                {{ model.isBusy ? "Signing in..." : "Login" }}
              </button>
              <button v-else-if="ui.authMode === 'register'" :disabled="model.isBusy" @click="registerUser">
                {{ model.isBusy ? "Creating account..." : "Create user" }}
              </button>
              <button v-else :disabled="model.isBusy" @click="registerAdmin">
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
                {{ model.isAdmin ? "Your everyday user tools stay here, while the admin portal is opened separately when you need operations work." : "User tools are unlocked for your account." }}
              </p>
            </div>

            <div class="action-row wrap">
              <button @click="refreshDashboard">Refresh dashboard</button>
              <button class="ghost-button" @click="model.runHealthCheck">Check health</button>
              <button v-if="model.isAdmin" class="ghost-button" @click="openPage('admin')">
                Admin mode
              </button>
            </div>
          </section>

          <section class="tab-strip">
            <button :class="['tab-button', ui.tab === 'overview' ? 'is-active' : '']" @click="ui.tab = 'overview'">Overview</button>
            <button :class="['tab-button', ui.tab === 'capture' ? 'is-active' : '']" @click="ui.tab = 'capture'">Capture</button>
            <button :class="['tab-button', ui.tab === 'activity' ? 'is-active' : '']" @click="ui.tab = 'activity'">Activity</button>
          </section>

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
                <div class="list-row"><strong>Avatar</strong><span>{{ data.profile?.avatarFilename || data.profile?.avatarFileId || "Not set" }}</span></div>
              </div>
            </article>
          </template>

          <template v-else-if="ui.tab === 'capture'">
            <section v-if="maintenanceMode" class="notice-card">
              Maintenance mode is on. Capture is still visible, but contribution actions should be treated as paused.
            </section>

            <section class="stack-lg">
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

              <article class="panel">
                <div class="section-head">
                  <div>
                    <p class="section-kicker">Submission</p>
                    <h2>Purchase and price</h2>
                  </div>
                </div>

                <div class="field-grid">
                  <label class="field"><span>Purchase time</span><input v-model="model.workbench.submission.purchaseTime" type="datetime-local" /></label>
                  <label class="field"><span>Recorded at</span><input v-model="model.workbench.submission.recordedAt" type="datetime-local" /></label>
                  <label class="field"><span>Amount</span><input v-model="model.workbench.submission.originalAmount" /></label>
                  <label class="field"><span>Currency</span><input v-model="model.workbench.submission.originalCurrency" /></label>
                  <label class="field"><span>Attachment IDs</span><input v-model="model.workbench.submission.attachmentFileIds" /></label>
                  <label class="field"><span>Notes</span><textarea v-model="model.workbench.submission.priceNotes" rows="4" /></label>
                </div>

                <div class="action-row wrap">
                  <button :disabled="model.isBusy || maintenanceMode" @click="model.createPurchase">Create purchase</button>
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

                <div class="field-grid">
                  <label class="field"><span>Filename</span><input v-model="model.workbench.submission.fileUpload.filename" /></label>
                  <label class="field"><span>Type</span><input v-model="model.workbench.submission.fileUpload.contentType" /></label>
                  <label class="field"><span>Size</span><input v-model="model.workbench.submission.fileUpload.size" /></label>
                  <label class="field"><span>Purpose</span><input v-model="model.workbench.submission.fileUpload.purpose" /></label>
                </div>

                <div class="action-row wrap">
                  <button :disabled="model.isBusy" @click="model.createUploadIntent">Create upload intent</button>
                  <button class="ghost-button" :disabled="model.isBusy" @click="model.completeUploadIntent">Complete upload</button>
                </div>
              </article>
            </section>
          </template>

          <template v-else-if="ui.tab === 'activity'">
            <section class="stack-lg">
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

                <div class="field-grid">
                  <label class="field"><span>Alert amount</span><input v-model="ui.alertAmount" /></label>
                  <label class="field"><span>Alert currency</span><input v-model="ui.alertCurrency" /></label>
                </div>

                <label class="checkbox-field">
                  <input v-model="ui.alertEnabled" type="checkbox" />
                  <span>Enable new alert</span>
                </label>

                <div class="action-row wrap">
                  <button :disabled="model.isBusy" @click="addWatchlist">
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
                :confirm-action="requestConfirmation"
                @flush="model.flushQueue"
                @retry="model.retryQueueEntry"
                @remove="model.removeQueueEntry"
                @clear="model.clearQueue"
              />

              <ResponsePanel :history="model.history" />
            </section>
          </template>
        </section>
      </main>
    </div>
  </div>
</template>
