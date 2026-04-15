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
  <n-card title="用户管理" :segmented="{ content: true }">
    <template #header-extra>
      <n-space>
        <n-button size="small" quaternary :disabled="disabled || loading || saving" @click="refreshUsers">刷新</n-button>
        <n-button size="small" type="primary" :disabled="disabled || saving" @click="openCreateModal">新建用户</n-button>
      </n-space>
    </template>

    <n-space align="center" wrap style="margin-bottom: 12px">
      <n-input v-model:value="ui.search" :disabled="disabled" clearable placeholder="搜索姓名、邮箱、角色或状态" style="min-width: 220px" />
      <n-select
        v-model:value="ui.pageSize"
        :disabled="disabled"
        style="width: 120px"
        :options="[
          { label: '10 / 页', value: 10 },
          { label: '20 / 页', value: 20 },
          { label: '25 / 页', value: 25 },
          { label: '50 / 页', value: 50 },
          { label: '100 / 页', value: 100 }
        ]"
      />
    </n-space>

    <n-card size="small" embedded title="批量操作" style="margin-bottom: 16px">
      <n-space vertical>
        <n-space align="center" wrap>
          <n-tag size="small">已选 {{ bulkForm.selectedIds.length }}</n-tag>
          <n-input v-model:value="bulkForm.reason" :disabled="disabled || saving" placeholder="原因" style="min-width: 200px" />
          <n-select
            v-model:value="bulkForm.status"
            :disabled="disabled || saving"
            style="width: 160px"
            :options="[
              { label: 'active', value: 'active' },
              { label: 'review', value: 'review' },
              { label: 'disabled', value: 'disabled' },
              { label: 'suspended', value: 'suspended' },
              { label: 'deleted', value: 'deleted' }
            ]"
          />
        </n-space>
        <n-space wrap>
          <n-button quaternary :disabled="disabled || saving || !paginatedUsers.length" @click="toggleVisibleUsers">
            {{ allVisibleSelected ? "取消本页全选" : "选择本页" }}
          </n-button>
          <n-button :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('freeze')">冻结</n-button>
          <n-button quaternary :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('activate')">激活</n-button>
          <n-button quaternary :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('set-status')">改状态</n-button>
          <n-button type="error" tertiary :disabled="disabled || saving || !bulkForm.selectedIds.length" @click="runBulkAction('delete')">删除</n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-spin :show="loading">
      <n-empty v-if="!paginatedUsers.length" :description="loading ? '加载中…' : '没有匹配用户'" />
      <n-space v-else vertical>
        <n-card v-for="user in paginatedUsers" :key="user.id" size="small" embedded>
          <n-space justify="space-between" align="start">
            <n-space align="start">
              <n-checkbox
                :checked="bulkForm.selectedIds.includes(user.id)"
                :disabled="disabled || saving"
                @update:checked="
                  (checked) => {
                    const has = bulkForm.selectedIds.includes(user.id);
                    if (checked && !has) bulkForm.selectedIds.push(user.id);
                    if (!checked && has) {
                      bulkForm.selectedIds = bulkForm.selectedIds.filter((id) => id !== user.id);
                    }
                  }
                "
              />
              <n-space vertical size="small">
                <n-text strong>{{ user.displayName || user.primaryEmail }}</n-text>
                <n-text depth="3" style="font-size: 12px">{{ user.id }}</n-text>
                <n-text depth="2">{{ user.primaryEmail }}</n-text>
                <n-space>
                  <n-tag size="small">{{ roleLabel(user) }}</n-tag>
                  <n-tag type="info" size="small">{{ user.status }}</n-tag>
                </n-space>
                <n-text depth="3" style="font-size: 12px">最近活跃：{{ dateLabel(user.lastActiveAt) }}</n-text>
              </n-space>
            </n-space>
            <n-space vertical>
              <n-button size="tiny" quaternary :disabled="disabled || saving" @click="openDetailModal(user)">详情</n-button>
              <n-button size="tiny" quaternary :disabled="disabled || saving" @click="openEditModal(user)">编辑</n-button>
            </n-space>
          </n-space>
        </n-card>
      </n-space>
    </n-spin>

    <n-space justify="center" style="margin-top: 16px">
      <n-pagination v-model:page="ui.page" :page-count="totalPages" simple />
    </n-space>

    <n-modal v-model:show="ui.detailOpen" preset="card" style="width: min(720px, 94vw)" :title="ui.detailUser ? (ui.detailUser.displayName || ui.detailUser.primaryEmail) : ''">
      <template v-if="ui.detailUser">
        <n-descriptions bordered size="small" :column="1">
          <n-descriptions-item label="邮箱">{{ ui.detailUser.primaryEmail }}</n-descriptions-item>
          <n-descriptions-item label="手机">{{ ui.detailUser.primaryPhone || '—' }}</n-descriptions-item>
          <n-descriptions-item label="角色">{{ roleLabel(ui.detailUser) }}</n-descriptions-item>
          <n-descriptions-item label="权限">{{ ui.detailUser.scopes?.join(', ') || '无' }}</n-descriptions-item>
          <n-descriptions-item label="状态">{{ ui.detailUser.status }}</n-descriptions-item>
          <n-descriptions-item label="简介">{{ ui.detailUser.profileBio || '无' }}</n-descriptions-item>
        </n-descriptions>
        <n-space style="margin-top: 16px" wrap>
          <n-button :disabled="disabled || saving" @click="openEditModal(ui.detailUser)">编辑</n-button>
          <n-button quaternary :disabled="disabled || saving || ui.detailUser.status === 'active'" @click="runSingleAction(ui.detailUser.id, 'activate')">激活</n-button>
          <n-button quaternary :disabled="disabled || saving || ui.detailUser.status === 'suspended'" @click="runSingleAction(ui.detailUser.id, 'freeze')">冻结</n-button>
          <n-button type="error" tertiary :disabled="disabled || saving || ui.detailUser.status === 'deleted'" @click="runSingleAction(ui.detailUser.id, 'delete')">删除</n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal v-model:show="ui.editorOpen" preset="card" style="width: min(640px, 94vw)" :title="ui.editorMode === 'create' ? '新建用户' : '编辑用户'">
      <n-form label-placement="top">
        <n-form-item label="邮箱">
          <n-input v-model:value="editor.email" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item v-if="ui.editorMode === 'create'" label="密码">
          <n-input v-model:value="editor.password" type="password" show-password-on="mousedown" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item v-if="ui.editorMode === 'create'" label="确认密码">
          <n-input v-model:value="editor.passwordConfirm" type="password" show-password-on="mousedown" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="显示名称">
          <n-input v-model:value="editor.displayName" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="手机">
          <n-input v-model:value="editor.primaryPhone" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="角色">
          <n-select
            v-model:value="editor.roleLevel"
            :disabled="disabled || saving"
            :options="[
              { label: '用户', value: 'user' },
              { label: '管理员', value: 'admin' }
            ]"
          />
        </n-form-item>
        <n-form-item label="状态">
          <n-select
            v-model:value="editor.accountStatus"
            :disabled="disabled || saving"
            :options="[
              { label: 'active', value: 'active' },
              { label: 'review', value: 'review' },
              { label: 'disabled', value: 'disabled' },
              { label: 'suspended', value: 'suspended' },
              { label: 'deleted', value: 'deleted' }
            ]"
          />
        </n-form-item>
        <n-form-item label="货币">
          <n-select v-model:value="editor.defaultCurrency" :options="SUPPORTED_PROFILE_CURRENCIES.map((c) => ({ label: c.label, value: c.code }))" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="Locale">
          <n-input v-model:value="editor.locale" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="时区">
          <n-input v-model:value="editor.timezoneName" :disabled="disabled || saving" />
        </n-form-item>
        <n-form-item label="简介">
          <n-input v-model:value="editor.profileBio" type="textarea" :rows="4" :disabled="disabled || saving" />
        </n-form-item>
        <n-alert v-if="ui.editorError" type="error" :title="ui.editorError" />
        <n-button type="primary" :disabled="disabled || saving" style="margin-top: 12px" @click="submitEditor">
          {{ saving ? '保存中…' : ui.editorMode === 'create' ? '创建' : '保存' }}
        </n-button>
      </n-form>
    </n-modal>
  </n-card>
</template>

