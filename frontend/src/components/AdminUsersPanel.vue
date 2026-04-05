<script setup>
import { computed, reactive, watch } from "vue";

import { SUPPORTED_PROFILE_CURRENCIES } from "../data/currencies";

const props = defineProps({
  users: {
    type: Array,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  saving: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  refreshUsers: {
    type: Function,
    required: true
  },
  createUser: {
    type: Function,
    required: true
  },
  updateUser: {
    type: Function,
    required: true
  },
  applyBulkAction: {
    type: Function,
    required: true
  },
  confirmAction: {
    type: Function,
    required: true
  }
});

const ui = reactive({
  search: "",
  page: 1,
  pageSize: 10,
  detailOpen: false,
  editorOpen: false,
  editorMode: "create",
  detailUser: null,
  editingUserId: "",
  editorError: ""
});

const bulkForm = reactive({
  selectedIds: [],
  status: "active",
  reason: "Administrative action"
});

const editor = reactive({
  email: "",
  password: "StrongPassword!234",
  passwordConfirm: "StrongPassword!234",
  displayName: "",
  primaryPhone: "",
  roleLevel: "user",
  accountStatus: "active",
  defaultCurrency: "GBP",
  locale: "en-GB",
  timezoneName: "Europe/London",
  profileBio: "",
  showPassword: false,
  showPasswordConfirm: false
});

function showHeldPassword(key) {
  editor[key] = true;
}

function hideHeldPassword(key) {
  editor[key] = false;
}

const filteredUsers = computed(() => {
  const search = ui.search.trim().toLowerCase();

  return props.users.filter((user) => {
    if (!search) {
      return true;
    }

    return [
      user.displayName,
      user.primaryEmail,
      user.primaryPhone,
      user.status,
      Array.isArray(user.roles) ? user.roles.join(" ") : ""
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(search));
  });
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil(filteredUsers.value.length / ui.pageSize))
);
const paginatedUsers = computed(() => {
  const start = (ui.page - 1) * ui.pageSize;
  return filteredUsers.value.slice(start, start + ui.pageSize);
});
const allVisibleSelected = computed(
  () =>
    paginatedUsers.value.length > 0 &&
    paginatedUsers.value.every((user) => bulkForm.selectedIds.includes(user.id))
);

function relocatePageForPageSize(nextPageSize, previousPageSize) {
  const nextSize = Math.max(1, Number(nextPageSize) || 1);
  const previousSize = Math.max(1, Number(previousPageSize) || nextSize);
  const firstVisibleIndex = Math.max(0, (ui.page - 1) * previousSize);
  const nextPage = Math.floor(firstVisibleIndex / nextSize) + 1;
  ui.page = Math.min(totalPages.value, Math.max(1, nextPage));
}

watch(filteredUsers, () => {
  if (ui.page > totalPages.value) {
    ui.page = totalPages.value;
  }
});

watch(
  () => ui.pageSize,
  (nextPageSize, previousPageSize) => {
    if (nextPageSize === previousPageSize) {
      return;
    }

    relocatePageForPageSize(nextPageSize, previousPageSize);
  }
);

watch(
  () => props.users,
  (users) => {
    const allowed = new Set(users.map((user) => user.id));
    bulkForm.selectedIds = bulkForm.selectedIds.filter((id) => allowed.has(id));
  },
  { deep: true }
);

function resetEditor() {
  editor.email = "";
  editor.password = "StrongPassword!234";
  editor.passwordConfirm = "StrongPassword!234";
  editor.displayName = "";
  editor.primaryPhone = "";
  editor.roleLevel = "user";
  editor.accountStatus = "active";
  editor.defaultCurrency = "GBP";
  editor.locale = "en-GB";
  editor.timezoneName = "Europe/London";
  editor.profileBio = "";
  editor.showPassword = false;
  editor.showPasswordConfirm = false;
}

function roleLabel(user) {
  return Array.isArray(user.roles) && user.roles.length ? user.roles.join(", ") : "user";
}

function dateLabel(value) {
  return value ? new Date(value).toLocaleString() : "Not available";
}

function openCreateModal() {
  ui.editorMode = "create";
  ui.editingUserId = "";
  ui.editorError = "";
  resetEditor();
  ui.editorOpen = true;
}

function openEditModal(user) {
  ui.editorMode = "edit";
  ui.editingUserId = user.id;
  ui.editorError = "";
  editor.email = user.primaryEmail || "";
  editor.password = "";
  editor.passwordConfirm = "";
  editor.displayName = user.displayName || "";
  editor.primaryPhone = user.primaryPhone || "";
  editor.roleLevel = Array.isArray(user.roles) && user.roles.includes("admin") ? "admin" : "user";
  editor.accountStatus = user.status || "active";
  editor.defaultCurrency = user.defaultCurrency || "GBP";
  editor.locale = user.locale || "en-GB";
  editor.timezoneName = user.timezoneName || "Europe/London";
  editor.profileBio = user.profileBio || "";
  ui.editorOpen = true;
}

function openDetailModal(user) {
  ui.detailUser = user;
  ui.detailOpen = true;
}

function userLabel(user) {
  return user?.displayName || user?.primaryEmail || user?.id || "this user";
}

function toggleUser(userId) {
  if (bulkForm.selectedIds.includes(userId)) {
    bulkForm.selectedIds = bulkForm.selectedIds.filter((id) => id !== userId);
    return;
  }

  bulkForm.selectedIds = [...bulkForm.selectedIds, userId];
}

function toggleVisibleUsers() {
  if (allVisibleSelected.value) {
    const visibleIds = new Set(paginatedUsers.value.map((user) => user.id));
    bulkForm.selectedIds = bulkForm.selectedIds.filter((id) => !visibleIds.has(id));
    return;
  }

  bulkForm.selectedIds = Array.from(
    new Set([...bulkForm.selectedIds, ...paginatedUsers.value.map((user) => user.id)])
  );
}

async function submitEditor() {
  if (ui.editorMode === "create") {
    if (editor.password !== editor.passwordConfirm) {
      ui.editorError = "The password confirmation does not match.";
      return;
    }

    const result = await props.createUser({
      email: editor.email,
      password: editor.password,
      displayName: editor.displayName,
      primaryPhone: editor.primaryPhone,
      roleCodes: editor.roleLevel === "admin" ? ["user", "admin"] : ["user"],
      accountStatus: editor.accountStatus
    });

    if (result?.ok) {
      ui.editorOpen = false;
      ui.editorError = "";
      resetEditor();
    }
    return;
  }

  const existingUser = props.users.find((user) => user.id === ui.editingUserId);
  if (
    editor.accountStatus === "deleted" &&
    existingUser?.status !== "deleted" &&
    !(await props.confirmAction({
      title: "Save deleted status?",
      message: `Save ${userLabel(existingUser)} with deleted status?`,
      confirmLabel: "Save as deleted",
      cancelLabel: "Keep editing",
      tone: "danger"
    }))
  ) {
    return;
  }

  const result = await props.updateUser(ui.editingUserId, {
    displayName: editor.displayName,
    primaryEmail: editor.email,
    primaryPhone: editor.primaryPhone,
    roleCodes: editor.roleLevel === "admin" ? ["user", "admin"] : ["user"],
    accountStatus: editor.accountStatus,
    defaultCurrency: String(editor.defaultCurrency || "")
      .trim()
      .toUpperCase(),
    locale: editor.locale,
    timezoneName: editor.timezoneName,
    profileBio: editor.profileBio
  });

  if (result?.ok) {
    ui.editorOpen = false;
    ui.detailOpen = false;
    ui.editorError = "";
  }
}

async function runBulkAction(action) {
  if (!bulkForm.selectedIds.length) {
    return;
  }

  if (
    action === "delete" &&
    !(await props.confirmAction({
      title: "Delete selected users?",
      message: `Delete ${bulkForm.selectedIds.length} selected user(s)?`,
      confirmLabel: "Delete users",
      cancelLabel: "Keep users",
      tone: "danger"
    }))
  ) {
    return;
  }

  if (
    action === "set-status" &&
    bulkForm.status === "deleted" &&
    !(await props.confirmAction({
      title: "Mark selected users as deleted?",
      message: `Change ${bulkForm.selectedIds.length} selected user(s) to deleted status?`,
      confirmLabel: "Mark deleted",
      cancelLabel: "Keep current status",
      tone: "danger"
    }))
  ) {
    return;
  }

  await props.applyBulkAction({
    accountIds: bulkForm.selectedIds,
    action,
    status: action === "set-status" ? bulkForm.status : undefined,
    reason: bulkForm.reason
  });
}

async function runSingleAction(userId, action, status) {
  const targetUser = props.users.find((user) => user.id === userId);

  if (
    action === "delete" &&
    !(await props.confirmAction({
      title: "Delete user?",
      message: `Delete ${userLabel(targetUser)}?`,
      confirmLabel: "Delete user",
      cancelLabel: "Keep user",
      tone: "danger"
    }))
  ) {
    return;
  }

  if (
    action === "set-status" &&
    status === "deleted" &&
    !(await props.confirmAction({
      title: "Mark user as deleted?",
      message: `Change ${userLabel(targetUser)} to deleted status?`,
      confirmLabel: "Mark deleted",
      cancelLabel: "Keep current status",
      tone: "danger"
    }))
  ) {
    return;
  }

  await props.applyBulkAction({
    accountIds: [userId],
    action,
    status,
    reason: bulkForm.reason
  });
}
</script>

<template>
  <article class="panel admin-module">
    <div class="section-head">
      <div>
        <p class="section-kicker">User management</p>
        <h2>Accounts, roles, and status</h2>
      </div>
      <div class="action-row wrap">
        <button class="ghost-button" :disabled="disabled || loading || saving" @click="refreshUsers">
          Refresh
        </button>
        <button :disabled="disabled || saving" @click="openCreateModal">Add new user</button>
      </div>
    </div>

    <div class="admin-toolbar">
      <label class="field">
        <span>Search users</span>
        <input v-model="ui.search" :disabled="disabled" placeholder="Search by name, email, role, or status" />
      </label>
      <label class="field compact-field">
        <span>Rows per page</span>
        <select v-model.number="ui.pageSize" :disabled="disabled">
          <option :value="10">10</option>
          <option :value="20">20</option>
          <option :value="25">25</option>
          <option :value="50">50</option>
          <option :value="100">100</option>
        </select>
      </label>
    </div>

    <article class="admin-subpanel">
      <div class="inline-head">
        <h3>Selected users</h3>
        <span class="status-pill tone-info">{{ bulkForm.selectedIds.length }} selected</span>
      </div>

      <div class="admin-toolbar">
        <label class="field">
          <span>Reason</span>
          <input v-model="bulkForm.reason" :disabled="disabled || saving" />
        </label>
        <label class="field compact-field">
          <span>Status</span>
          <select v-model="bulkForm.status" :disabled="disabled || saving">
            <option value="active">Active</option>
            <option value="review">Review</option>
            <option value="disabled">Disabled</option>
            <option value="suspended">Suspended</option>
            <option value="deleted">Deleted</option>
          </select>
        </label>
      </div>

      <div class="action-row wrap">
        <button class="ghost-button" :disabled="disabled || saving || !paginatedUsers.length" @click="toggleVisibleUsers">
          {{ allVisibleSelected ? "Clear visible selection" : "Select visible rows" }}
        </button>
        <button :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('freeze')">
          Freeze
        </button>
        <button class="ghost-button" :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('activate')">
          Activate
        </button>
        <button class="ghost-button" :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('set-status')">
          Change status
        </button>
        <button class="danger-button" :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('delete')">
          Delete
        </button>
      </div>
    </article>

    <div class="table-wrap">
      <table class="admin-table">
        <thead>
          <tr>
            <th class="select-col">Select</th>
            <th>User</th>
            <th>Email</th>
            <th>Role</th>
            <th>Status</th>
            <th>Last active</th>
            <th class="actions-col">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in paginatedUsers" :key="user.id" @click="openDetailModal(user)">
            <td class="select-col" @click.stop>
              <input
                :checked="bulkForm.selectedIds.includes(user.id)"
                :disabled="disabled || saving"
                type="checkbox"
                @change="toggleUser(user.id)"
              />
            </td>
            <td>
              <strong>{{ user.displayName || user.primaryEmail }}</strong>
              <div class="table-subtext">{{ user.id }}</div>
            </td>
            <td>{{ user.primaryEmail }}</td>
            <td>{{ roleLabel(user) }}</td>
            <td><span class="status-pill tone-info">{{ user.status }}</span></td>
            <td>{{ dateLabel(user.lastActiveAt) }}</td>
            <td class="actions-col" @click.stop>
              <div class="action-row wrap">
                <button class="ghost-button compact-button" :disabled="disabled || saving" @click="openDetailModal(user)">
                  View
                </button>
                <button class="ghost-button compact-button" :disabled="disabled || saving" @click="openEditModal(user)">
                  Edit
                </button>
              </div>
            </td>
          </tr>
          <tr v-if="!paginatedUsers.length">
            <td colspan="7" class="empty-table">
              {{ loading ? "Loading users..." : "No users matched the current filters." }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="admin-pagination">
      <span class="helper-text">Page {{ ui.page }} of {{ totalPages }}</span>
      <div class="action-row wrap">
        <button class="ghost-button compact-button" :disabled="ui.page <= 1" @click="ui.page -= 1">
          Previous
        </button>
        <button class="ghost-button compact-button" :disabled="ui.page >= totalPages" @click="ui.page += 1">
          Next
        </button>
      </div>
    </div>

    <div v-if="ui.detailOpen && ui.detailUser" class="admin-modal-backdrop" @click.self="ui.detailOpen = false">
      <article class="admin-modal">
        <div class="section-head">
          <div>
            <p class="section-kicker">User detail</p>
            <h2>{{ ui.detailUser.displayName || ui.detailUser.primaryEmail }}</h2>
          </div>
          <div class="action-row wrap">
            <button
              class="ghost-button compact-button"
              :disabled="disabled || saving"
              @click="openEditModal(ui.detailUser)"
            >
              Edit user
            </button>
            <button class="ghost-button compact-button" @click="ui.detailOpen = false">Close</button>
          </div>
        </div>

        <div class="simple-list">
          <div class="list-row"><strong>Email</strong><span>{{ ui.detailUser.primaryEmail }}</span></div>
          <div class="list-row"><strong>Phone</strong><span>{{ ui.detailUser.primaryPhone || "Not set" }}</span></div>
          <div class="list-row"><strong>Roles</strong><span>{{ roleLabel(ui.detailUser) }}</span></div>
          <div class="list-row"><strong>Scopes</strong><span>{{ ui.detailUser.scopes?.join(", ") || "None" }}</span></div>
          <div class="list-row"><strong>Status</strong><span>{{ ui.detailUser.status }}</span></div>
          <div class="list-row"><strong>Locale</strong><span>{{ ui.detailUser.locale }}</span></div>
          <div class="list-row"><strong>Timezone</strong><span>{{ ui.detailUser.timezoneName }}</span></div>
          <div class="list-row"><strong>Currency</strong><span>{{ ui.detailUser.defaultCurrency }}</span></div>
          <div class="list-row"><strong>Email count</strong><span>{{ ui.detailUser.emailCount }}</span></div>
          <div class="list-row"><strong>Phone count</strong><span>{{ ui.detailUser.phoneCount }}</span></div>
          <div class="list-row"><strong>Created</strong><span>{{ dateLabel(ui.detailUser.createdAt) }}</span></div>
          <div class="list-row"><strong>Last active</strong><span>{{ dateLabel(ui.detailUser.lastActiveAt) }}</span></div>
          <div class="list-row"><strong>Suspended until</strong><span>{{ dateLabel(ui.detailUser.suspendedUntil) }}</span></div>
          <div class="list-row"><strong>Deleted at</strong><span>{{ dateLabel(ui.detailUser.deletedAt) }}</span></div>
          <div class="list-row"><strong>Password</strong><span>{{ ui.detailUser.security?.passwordSet ? "Set" : "Not set" }}</span></div>
          <div class="list-row"><strong>MFA</strong><span>{{ ui.detailUser.security?.mfaEnabled ? "Enabled" : "Disabled" }}</span></div>
          <div class="list-row"><strong>Passkeys</strong><span>{{ ui.detailUser.security?.passkeyCount || 0 }}</span></div>
          <div class="list-row"><strong>Bio</strong><span>{{ ui.detailUser.profileBio || "No bio" }}</span></div>
        </div>

        <div class="action-row wrap">
          <button :disabled="disabled || saving" @click="openEditModal(ui.detailUser)">Edit user</button>
          <button
            class="ghost-button"
            :disabled="disabled || saving || ui.detailUser.status === 'active'"
            @click="runSingleAction(ui.detailUser.id, 'activate')"
          >
            Activate user
          </button>
          <button
            class="ghost-button"
            :disabled="disabled || saving || ui.detailUser.status === 'suspended'"
            @click="runSingleAction(ui.detailUser.id, 'freeze')"
          >
            Freeze user
          </button>
          <button
            class="danger-button"
            :disabled="disabled || saving || ui.detailUser.status === 'deleted'"
            @click="runSingleAction(ui.detailUser.id, 'delete')"
          >
            Delete user
          </button>
        </div>
      </article>
    </div>

    <div v-if="ui.editorOpen" class="admin-modal-backdrop" @click.self="ui.editorOpen = false">
      <article class="admin-modal">
        <div class="section-head">
          <div>
            <p class="section-kicker">{{ ui.editorMode === "create" ? "Add user" : "Edit user" }}</p>
            <h2>{{ ui.editorMode === "create" ? "Create account" : "Update account" }}</h2>
          </div>
          <button class="ghost-button compact-button" @click="ui.editorOpen = false">Close</button>
        </div>

        <div class="field-grid">
          <label class="field">
            <span>Email</span>
            <input v-model="editor.email" :disabled="disabled || saving" />
            <small class="field-note">Required. This becomes the user's primary sign-in email.</small>
          </label>
          <label v-if="ui.editorMode === 'create'" class="field">
            <span>Password</span>
            <div class="password-field">
              <input
                v-model="editor.password"
                :disabled="disabled || saving"
                :type="editor.showPassword ? 'text' : 'password'"
              />
              <button
                class="ghost-icon-button password-toggle"
                type="button"
                aria-label="Hold to show password"
                title="Hold to show password"
                @pointerdown.prevent="showHeldPassword('showPassword')"
                @pointerup="hideHeldPassword('showPassword')"
                @pointerleave="hideHeldPassword('showPassword')"
                @pointercancel="hideHeldPassword('showPassword')"
                @blur="hideHeldPassword('showPassword')"
                @keydown.space.prevent="showHeldPassword('showPassword')"
                @keyup.space="hideHeldPassword('showPassword')"
                @keydown.enter.prevent="showHeldPassword('showPassword')"
                @keyup.enter="hideHeldPassword('showPassword')"
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
            <small class="field-note">Required. Use at least 12 characters for the first password.</small>
          </label>
          <label v-if="ui.editorMode === 'create'" class="field">
            <span>Confirm password</span>
            <div class="password-field">
              <input
                v-model="editor.passwordConfirm"
                :disabled="disabled || saving"
                :type="editor.showPasswordConfirm ? 'text' : 'password'"
              />
              <button
                class="ghost-icon-button password-toggle"
                type="button"
                aria-label="Hold to show password confirmation"
                title="Hold to show password confirmation"
                @pointerdown.prevent="showHeldPassword('showPasswordConfirm')"
                @pointerup="hideHeldPassword('showPasswordConfirm')"
                @pointerleave="hideHeldPassword('showPasswordConfirm')"
                @pointercancel="hideHeldPassword('showPasswordConfirm')"
                @blur="hideHeldPassword('showPasswordConfirm')"
                @keydown.space.prevent="showHeldPassword('showPasswordConfirm')"
                @keyup.space="hideHeldPassword('showPasswordConfirm')"
                @keydown.enter.prevent="showHeldPassword('showPasswordConfirm')"
                @keyup.enter="hideHeldPassword('showPasswordConfirm')"
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
            <small class="field-note">Required. Enter the same password again to validate it.</small>
          </label>
          <label class="field">
            <span>Display name</span>
            <input v-model="editor.displayName" :disabled="disabled || saving" />
            <small class="field-note">Required. This name is shown in the UI and admin lists.</small>
          </label>
          <label class="field">
            <span>Primary phone</span>
            <input v-model="editor.primaryPhone" :disabled="disabled || saving" />
            <small class="field-note">Optional. Include the country code if one is available.</small>
          </label>
          <label class="field">
            <span>Role</span>
            <select v-model="editor.roleLevel" :disabled="disabled || saving">
              <option value="user">User</option>
              <option value="admin">Admin</option>
            </select>
          </label>
          <label class="field">
            <span>Status</span>
            <select v-model="editor.accountStatus" :disabled="disabled || saving">
              <option value="active">Active</option>
              <option value="review">Review</option>
              <option value="disabled">Disabled</option>
              <option value="suspended">Suspended</option>
              <option value="deleted">Deleted</option>
            </select>
          </label>
          <label class="field">
            <span>Currency</span>
            <select v-model="editor.defaultCurrency" :disabled="disabled || saving">
              <option
                v-for="currency in SUPPORTED_PROFILE_CURRENCIES"
                :key="currency.code"
                :value="currency.code"
              >
                {{ currency.label }}
              </option>
            </select>
            <small class="field-note">Use a currency code that already exists in the database.</small>
          </label>
          <label class="field">
            <span>Locale</span>
            <input v-model="editor.locale" :disabled="disabled || saving" />
          </label>
          <label class="field">
            <span>Timezone</span>
            <input v-model="editor.timezoneName" :disabled="disabled || saving" />
          </label>
          <label class="field full-span">
            <span>Bio</span>
            <textarea v-model="editor.profileBio" :disabled="disabled || saving" rows="4" />
          </label>
        </div>

        <p v-if="ui.editorError" class="helper-text">{{ ui.editorError }}</p>

        <div class="action-row wrap">
          <button :disabled="disabled || saving" @click="submitEditor">
            {{ saving ? "Saving..." : ui.editorMode === "create" ? "Create user" : "Save changes" }}
          </button>
        </div>
      </article>
    </div>
  </article>
</template>
