<script setup>
import { reactive } from "vue";

import { SUPPORTED_PROFILE_CURRENCIES } from "../data/currencies";

defineProps({
  profile: {
    type: Object,
    default: null
  },
  loading: {
    type: Boolean,
    default: false
  },
  busy: {
    type: Boolean,
    default: false
  },
  profileForm: {
    type: Object,
    required: true
  },
  emailDraft: {
    type: Object,
    required: true
  },
  phoneDraft: {
    type: Object,
    required: true
  },
  passwordForm: {
    type: Object,
    required: true
  },
  avatarState: {
    type: Object,
    required: true
  },
  cookiePreferences: {
    type: Object,
    required: true
  },
  emails: {
    type: Array,
    required: true
  },
  phones: {
    type: Array,
    required: true
  }
});

defineEmits([
  "refresh",
  "save-profile",
  "pick-avatar",
  "upload-avatar",
  "remove-avatar",
  "add-email",
  "delete-email",
  "verify-email",
  "make-primary-email",
  "add-phone",
  "delete-phone",
  "verify-phone",
  "make-primary-phone",
  "change-password",
  "save-cookie-preferences"
]);

const passwordUi = reactive({
  showCurrent: false,
  showNext: false,
  showConfirm: false
});

function showHeldPassword(key) {
  passwordUi[key] = true;
}

function hideHeldPassword(key) {
  passwordUi[key] = false;
}
</script>

<template>
  <section class="stack-lg">
    <article class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Profile</p>
          <h2>Account settings</h2>
        </div>
        <div class="action-row wrap">
          <span class="status-pill tone-good">{{ profile?.status || "active" }}</span>
          <button class="ghost-button" :disabled="busy || loading" @click="$emit('refresh')">
            Reload
          </button>
        </div>
      </div>

      <div class="field-grid">
        <label class="field">
          <span>Display name</span>
          <input v-model="profileForm.displayName" :disabled="busy" />
        </label>
        <label class="field">
          <span>Default currency</span>
          <select v-model="profileForm.defaultCurrency" :disabled="busy">
            <option
              v-for="currency in SUPPORTED_PROFILE_CURRENCIES"
              :key="currency.code"
              :value="currency.code"
            >
              {{ currency.label }}
            </option>
          </select>
          <small class="field-note">Choose one of the currencies currently available in the database.</small>
        </label>
        <label class="field">
          <span>Locale</span>
          <input v-model="profileForm.locale" :disabled="busy" />
        </label>
        <label class="field">
          <span>Timezone</span>
          <input v-model="profileForm.timezoneName" :disabled="busy" />
        </label>
        <label class="field">
          <span>Bio</span>
          <textarea v-model="profileForm.profileBio" :disabled="busy" rows="4" />
        </label>
      </div>

      <div class="pill-row">
        <span class="status-pill tone-info">
          {{ profile?.security?.passwordSet ? "Password ready" : "Password missing" }}
        </span>
        <span class="status-pill tone-accent">
          {{ profile?.security?.mfaEnabled ? "MFA enabled" : "MFA not enabled" }}
        </span>
        <span class="status-pill tone-good">
          {{ `${profile?.security?.passkeyCount || 0} passkeys` }}
        </span>
      </div>

      <div class="action-row wrap">
        <button :disabled="busy" @click="$emit('save-profile')">
          {{ busy ? "Saving..." : "Save profile" }}
        </button>
      </div>
    </article>

    <article class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Avatar</p>
          <h2>Profile image</h2>
        </div>
        <span class="status-pill tone-info">
          {{ profile?.avatarFilename || profile?.avatarFileId || "No avatar" }}
        </span>
      </div>

      <p class="helper-text">
        Upload uses the current dev file-intent flow, then attaches the completed file as your avatar.
      </p>

      <div class="field-grid">
        <label class="field">
          <span>Selected file</span>
          <input :value="avatarState.selectedFileName || ''" disabled />
        </label>
      </div>

      <div class="action-row wrap">
        <label class="file-button">
          Choose avatar
          <input :disabled="busy" type="file" accept="image/*" @change="$emit('pick-avatar', $event)" />
        </label>
        <button :disabled="busy || !avatarState.selectedFile" @click="$emit('upload-avatar')">
          {{ busy ? "Uploading..." : "Upload avatar" }}
        </button>
        <button class="ghost-button" :disabled="busy || !profile?.avatarFileId" @click="$emit('remove-avatar')">
          Remove avatar
        </button>
      </div>
    </article>

    <article class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Identity</p>
          <h2>Email addresses</h2>
        </div>
        <span class="status-pill tone-accent">{{ emails.length }} stored</span>
      </div>

      <div class="field-grid">
        <label class="field">
          <span>New email</span>
          <input v-model="emailDraft.email" :disabled="busy" />
        </label>
        <label class="field">
          <span>Email role</span>
          <select v-model="emailDraft.emailRole" :disabled="busy">
            <option value="SECONDARY">Secondary</option>
            <option value="PRIMARY">Primary</option>
          </select>
        </label>
        <label class="checkbox-field">
          <input v-model="emailDraft.isLoginEnabled" :disabled="busy" type="checkbox" />
          <span>Allow login with this email</span>
        </label>
      </div>

      <div class="action-row wrap">
        <button :disabled="busy" @click="$emit('add-email')">
          {{ busy ? "Saving..." : "Add email" }}
        </button>
      </div>

      <div class="simple-list">
        <article v-for="email in emails" :key="email.id" class="list-card">
          <div class="list-row"><strong>{{ email.email }}</strong><span>{{ email.emailRole }}</span></div>
          <div class="pill-row">
            <span :class="['status-pill', email.isPrimaryForAccount ? 'tone-good' : 'tone-info']">
              {{ email.isPrimaryForAccount ? "Primary" : "Secondary" }}
            </span>
            <span :class="['status-pill', email.verifiedAt ? 'tone-good' : 'tone-warn']">
              {{ email.verifiedAt ? "Verified" : "Unverified" }}
            </span>
          </div>
          <div class="action-row wrap">
            <button class="ghost-button" :disabled="busy || email.isPrimaryForAccount" @click="$emit('make-primary-email', email.id)">
              Make primary
            </button>
            <button class="ghost-button" :disabled="busy || email.verifiedAt" @click="$emit('verify-email', email.id)">
              Verify
            </button>
            <button class="danger-button" :disabled="busy || email.isPrimaryForAccount" @click="$emit('delete-email', email.id)">
              Delete
            </button>
          </div>
          <p v-if="email.isPrimaryForAccount" class="field-note">
            Make another email primary before deleting this address.
          </p>
        </article>
      </div>
    </article>

    <article class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Identity</p>
          <h2>Phone numbers</h2>
        </div>
        <span class="status-pill tone-accent">{{ phones.length }} stored</span>
      </div>

      <div class="field-grid">
        <label class="field">
          <span>New phone</span>
          <input v-model="phoneDraft.phoneNumber" :disabled="busy" />
        </label>
      </div>

      <div class="action-row wrap">
        <button :disabled="busy" @click="$emit('add-phone')">
          {{ busy ? "Saving..." : "Add phone" }}
        </button>
      </div>

      <div class="simple-list">
        <article v-for="phone in phones" :key="phone.id" class="list-card">
          <div class="list-row"><strong>{{ phone.phoneNumber }}</strong><span>{{ phone.isPrimaryForAccount ? "Primary" : "Secondary" }}</span></div>
          <div class="pill-row">
            <span :class="['status-pill', phone.verifiedAt ? 'tone-good' : 'tone-warn']">
              {{ phone.verifiedAt ? "Verified" : "Unverified" }}
            </span>
          </div>
          <div class="action-row wrap">
            <button class="ghost-button" :disabled="busy || phone.isPrimaryForAccount" @click="$emit('make-primary-phone', phone.id)">
              Make primary
            </button>
            <button class="ghost-button" :disabled="busy || phone.verifiedAt" @click="$emit('verify-phone', phone.id)">
              Verify
            </button>
            <button class="danger-button" :disabled="busy" @click="$emit('delete-phone', phone.id)">
              Delete
            </button>
          </div>
        </article>
      </div>
    </article>

    <article class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Security</p>
          <h2>Password and privacy</h2>
        </div>
      </div>

      <div class="field-grid">
        <label class="field">
          <span>Current password</span>
          <div class="password-field">
            <input
              v-model="passwordForm.currentPassword"
              :disabled="busy"
              :type="passwordUi.showCurrent ? 'text' : 'password'"
            />
            <button
              class="ghost-icon-button password-toggle"
              type="button"
              aria-label="Hold to show current password"
              title="Hold to show current password"
              @pointerdown.prevent="showHeldPassword('showCurrent')"
              @pointerup="hideHeldPassword('showCurrent')"
              @pointerleave="hideHeldPassword('showCurrent')"
              @pointercancel="hideHeldPassword('showCurrent')"
              @blur="hideHeldPassword('showCurrent')"
              @keydown.space.prevent="showHeldPassword('showCurrent')"
              @keyup.space="hideHeldPassword('showCurrent')"
              @keydown.enter.prevent="showHeldPassword('showCurrent')"
              @keyup.enter="hideHeldPassword('showCurrent')"
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
        </label>
        <label class="field">
          <span>New password</span>
          <div class="password-field">
            <input
              v-model="passwordForm.newPassword"
              :disabled="busy"
              :type="passwordUi.showNext ? 'text' : 'password'"
            />
            <button
              class="ghost-icon-button password-toggle"
              type="button"
              aria-label="Hold to show new password"
              title="Hold to show new password"
              @pointerdown.prevent="showHeldPassword('showNext')"
              @pointerup="hideHeldPassword('showNext')"
              @pointerleave="hideHeldPassword('showNext')"
              @pointercancel="hideHeldPassword('showNext')"
              @blur="hideHeldPassword('showNext')"
              @keydown.space.prevent="showHeldPassword('showNext')"
              @keyup.space="hideHeldPassword('showNext')"
              @keydown.enter.prevent="showHeldPassword('showNext')"
              @keyup.enter="hideHeldPassword('showNext')"
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
        </label>
        <label class="field">
          <span>Confirm new password</span>
          <div class="password-field">
            <input
              v-model="passwordForm.confirmPassword"
              :disabled="busy"
              :type="passwordUi.showConfirm ? 'text' : 'password'"
            />
            <button
              class="ghost-icon-button password-toggle"
              type="button"
              aria-label="Hold to show password confirmation"
              title="Hold to show password confirmation"
              @pointerdown.prevent="showHeldPassword('showConfirm')"
              @pointerup="hideHeldPassword('showConfirm')"
              @pointerleave="hideHeldPassword('showConfirm')"
              @pointercancel="hideHeldPassword('showConfirm')"
              @blur="hideHeldPassword('showConfirm')"
              @keydown.space.prevent="showHeldPassword('showConfirm')"
              @keyup.space="hideHeldPassword('showConfirm')"
              @keydown.enter.prevent="showHeldPassword('showConfirm')"
              @keyup.enter="hideHeldPassword('showConfirm')"
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
        </label>
      </div>

      <div class="action-row wrap">
        <button :disabled="busy" @click="$emit('change-password')">
          {{ busy ? "Saving..." : "Change password" }}
        </button>
      </div>

      <div class="field-grid">
        <label class="checkbox-field">
          <input v-model="cookiePreferences.analytics" :disabled="busy" type="checkbox" />
          <span>Analytics cookies</span>
        </label>
        <label class="checkbox-field">
          <input v-model="cookiePreferences.marketing" :disabled="busy" type="checkbox" />
          <span>Marketing cookies</span>
        </label>
        <label class="checkbox-field">
          <input v-model="cookiePreferences.preferences" :disabled="busy" type="checkbox" />
          <span>Preference cookies</span>
        </label>
      </div>

      <div class="action-row wrap">
        <button class="ghost-button" :disabled="busy" @click="$emit('save-cookie-preferences')">
          {{ busy ? "Saving..." : "Save cookie preferences" }}
        </button>
      </div>
    </article>
  </section>
</template>
