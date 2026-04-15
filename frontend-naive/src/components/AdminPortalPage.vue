<script setup>
import { computed, onMounted, reactive } from "vue";

import AdminEntityTablePanel from "./AdminEntityTablePanel.vue";
import AdminExplorerPanel from "./AdminExplorerPanel.vue";
import AdminModerationPanel from "./AdminModerationPanel.vue";
import AdminSettingsPanel from "./AdminSettingsPanel.vue";
import AdminUsersPanel from "./AdminUsersPanel.vue";
import ResponsePanel from "./ResponsePanel.vue";

const props = defineProps({
  model: {
    type: Object,
    required: true
  },
  settings: {
    type: Array,
    required: true
  },
  loadingSettings: {
    type: Boolean,
    default: false
  },
  savingSetting: {
    type: String,
    default: ""
  },
  disabled: {
    type: Boolean,
    default: false
  },
  feedback: {
    type: Object,
    required: true
  },
  hiddenModulesEnabled: {
    type: Boolean,
    default: true
  },
  maintenanceMode: {
    type: Boolean,
    default: false
  },
  refreshAll: {
    type: Function,
    required: true
  },
  refreshSettings: {
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
  applyUserAction: {
    type: Function,
    required: true
  },
  saveSetting: {
    type: Function,
    required: true
  },
  saveAdminRecord: {
    type: Function,
    required: true
  },
  deleteAdminRecord: {
    type: Function,
    required: true
  },
  approveAdminRecord: {
    type: Function,
    required: true
  },
  bulkDeleteAdminRecords: {
    type: Function,
    required: true
  },
  bulkApproveAdminRecords: {
    type: Function,
    required: true
  },
  confirmAction: {
    type: Function,
    required: true
  }
});

const emit = defineEmits(["exit"]);

const ui = reactive({
  section: "dashboard"
});

const nonDebugSettings = computed(() =>
  props.settings.filter((setting) => !String(setting.key || "").startsWith("debug."))
);
const debugSettings = computed(() =>
  props.settings.filter((setting) => String(setting.key || "").startsWith("debug."))
);
const currentTableId = computed(() =>
  ui.section.startsWith("table:") ? ui.section.slice("table:".length) : ""
);
const currentTable = computed(() =>
  currentTableId.value
    ? props.model.adminTables.find((table) => table.id === currentTableId.value) ||
      props.model.admin.table ||
      null
    : null
);
const currentRows = computed(() =>
  currentTableId.value === props.model.admin.selectedTableId ? props.model.admin.rows : []
);
const navigationGroups = computed(() => [
  {
    label: "Overview",
    items: [{ key: "dashboard", label: "Admin Dashboard" }]
  },
  {
    label: "Operations",
    items: [
      { key: "users", label: "User management" },
      { key: "moderation", label: "Approvals" }
    ]
  },
  {
    label: "Catalog",
    items: [
      { key: "table:categories", label: "Category" },
      { key: "table:brands", label: "Brand" },
      { key: "table:units", label: "Unit" },
      { key: "table:retailers", label: "Retailer" },
      { key: "table:shops", label: "Shop" },
      { key: "table:items", label: "Item" },
      { key: "table:item-variants", label: "Item variant" },
      { key: "table:discount-types", label: "Discount type" }
    ]
  },
  {
    label: "System",
    items: [
      { key: "settings", label: "Settings" },
      { key: "debug", label: "Debug" }
    ]
  }
]);

const dashboardCards = computed(() => [
  {
    key: "users",
    label: "Users",
    value: String(props.model.admin.overview?.accountCount || props.model.admin.users.length || 0),
    note: "Review accounts, open user detail windows, and update roles or status.",
    section: "users",
    tone: "info"
  },
  {
    key: "approvals",
    label: "Approvals",
    value: String(props.model.admin.overview?.pendingModerationCount || 0),
    note: "Review pending submissions before they move into the live catalog.",
    section: "moderation",
    tone:
      (props.model.admin.overview?.pendingModerationCount || 0) > 0 ? "warn" : "good"
  },
  {
    key: "categories",
    label: "Category",
    value: String(props.model.admin.overview?.categoryCount || 0),
    note: "Manage category records in a dedicated table workflow.",
    section: "table:categories",
    tone: "quiet"
  },
  {
    key: "brands",
    label: "Brands",
    value: String(props.model.admin.overview?.brandCount || 0),
    note: "Review brand records linked to catalog items and variants.",
    section: "table:brands",
    tone: "quiet"
  },
  {
    key: "units",
    label: "Unit",
    value: String(props.model.admin.overview?.unitCount || 0),
    note: "Review conversion units and symbols used by the catalog.",
    section: "table:units",
    tone: "quiet"
  },
  {
    key: "retailers",
    label: "Retailer",
    value: String(props.model.admin.overview?.retailerCount || 0),
    note: "Keep retailer records clean before shops and items depend on them.",
    section: "table:retailers",
    tone: "quiet"
  },
  {
    key: "items",
    label: "Items",
    value: String(props.model.admin.overview?.itemCount || 0),
    note: "Open the item table for add, edit, archive, and status review.",
    section: "table:items",
    tone: "accent"
  },
  {
    key: "variants",
    label: "Item variants",
    value: String(props.model.admin.overview?.itemVariantCount || 0),
    note: "Inspect the current variant records that sit under each item.",
    section: "table:item-variants",
    tone: "accent"
  },
  {
    key: "shops",
    label: "Shops",
    value: String(props.model.admin.overview?.shopCount || 0),
    note: "Manage retailer and shop records used during capture and lookup.",
    section: "table:shops",
    tone: "good"
  },
  {
    key: "discount-types",
    label: "Discount types",
    value: String(props.model.admin.overview?.discountTypeCount || 0),
    note: "Maintain discount labels used in price and purchase submissions.",
    section: "table:discount-types",
    tone: "quiet"
  },
  {
    key: "settings",
    label: "Settings",
    value: String(props.model.admin.overview?.systemSettingCount || 0),
    note: "Open categorized settings without exposing the raw debug tools.",
    section: "settings",
    tone: "quiet"
  },
  {
    key: "debug",
    label: "Debug",
    value: props.hiddenModulesEnabled ? "Enabled" : "Hidden",
    note: "Keep debugging and explorer tools separated from the daily admin flow.",
    section: "debug",
    tone: props.hiddenModulesEnabled ? "warn" : "quiet"
  }
]);

async function openSection(section) {
  ui.section = section;

  if (section === "dashboard") {
    if (!props.model.admin.overview) {
      await props.refreshAll();
    }
    return;
  }

  if (section === "users") {
    if (!props.model.admin.users.length) {
      await props.model.loadAdminUsers();
    }
    return;
  }

  if (section === "moderation") {
    if (!props.model.admin.moderationPrices.length) {
      await props.model.loadModerationPrices();
    }
    return;
  }

  if (section === "settings" || section === "debug") {
    if (!props.settings.length) {
      await props.refreshSettings();
    }
    return;
  }

  if (section.startsWith("table:")) {
    const tableId = section.slice("table:".length);
    if (tableId && props.model.admin.selectedTableId !== tableId) {
      await props.model.selectAdminTable(tableId);
      return;
    }

    if (!props.model.admin.rows.length) {
      await props.model.refreshAdminTable();
    }
  }
}

onMounted(() => {
  void props.refreshAll();
});
</script>

<template>
  <n-layout has-sider position="absolute" style="inset: 0; height: 100vh">
    <n-layout-sider
      bordered
      show-trigger
      collapse-mode="width"
      :collapsed-width="64"
      :width="260"
      :native-scrollbar="false"
      content-style="padding: 12px"
    >
      <n-space vertical size="large">
        <n-space justify="space-between" align="center">
          <n-text strong>管理后台</n-text>
          <n-tag :type="maintenanceMode ? 'warning' : 'success'" size="small">
            {{ maintenanceMode ? "维护中" : "正常" }}
          </n-tag>
        </n-space>

        <n-descriptions label-placement="left" size="small" :column="1" bordered>
          <n-descriptions-item label="用户">
            {{ model.workbench.session.displayName || model.workbench.session.email }}
          </n-descriptions-item>
          <n-descriptions-item label="API">
            {{ model.apiOnline ? "可达" : "不可用" }}
          </n-descriptions-item>
          <n-descriptions-item label="数据库">
            {{ model.databaseOnline ? "已连接" : "不可用" }}
          </n-descriptions-item>
        </n-descriptions>

        <n-scrollbar style="max-height: calc(100vh - 280px)">
          <n-space vertical size="medium">
            <template v-for="group in navigationGroups" :key="group.label">
              <n-text depth="3" style="font-size: 11px; letter-spacing: 0.08em">{{ group.label }}</n-text>
              <n-space vertical size="small">
                <n-button
                  v-for="item in group.items"
                  :key="item.key"
                  block
                  quaternary
                  :type="ui.section === item.key ? 'primary' : 'default'"
                  size="small"
                  @click="openSection(item.key)"
                >
                  {{ item.label }}
                </n-button>
              </n-space>
            </template>
          </n-space>
        </n-scrollbar>

        <n-space vertical>
          <n-button quaternary block :disabled="disabled" @click="refreshAll">刷新门户</n-button>
          <n-button type="error" secondary block @click="$emit('exit')">退出管理模式</n-button>
        </n-space>
      </n-space>
    </n-layout-sider>

    <n-layout content-style="height: 100vh; display: flex; flex-direction: column">
      <n-layout-header bordered style="padding: 16px 20px">
        <n-space justify="space-between" align="center" wrap>
          <n-space vertical>
            <n-text tag="h1" style="margin: 0; font-size: 1.25rem">日常运维工作台</n-text>
            <n-text depth="3">用户前台与此管理界面分离。</n-text>
          </n-space>
          <n-space size="small" wrap>
            <n-tag v-for="light in model.statusLights" :key="light.key" size="small" type="default">
              {{ light.label }}: {{ light.state }}
            </n-tag>
          </n-space>
        </n-space>
      </n-layout-header>

      <n-layout-content content-style="padding: 16px 20px 24px; overflow: auto">
        <section v-if="ui.section === 'dashboard'">
          <n-card title="总览" :segmented="{ content: true }">
            <template #header-extra>
              <n-button size="small" quaternary :disabled="disabled" @click="refreshAll">刷新计数</n-button>
            </template>
            <n-text depth="3" style="display: block; margin-bottom: 16px">
              点击下方卡片进入各模块。
            </n-text>
            <n-grid :cols="2" :x-gap="12" :y-gap="12" responsive="screen">
              <n-gi v-for="card in dashboardCards" :key="card.key" span="2 m:1">
                <n-card size="small" hoverable embedded @click="openSection(card.section)">
                  <n-statistic :label="card.label" :value="card.value" />
                  <n-text depth="3" style="display: block; margin-top: 8px">{{ card.note }}</n-text>
                </n-card>
              </n-gi>
            </n-grid>
          </n-card>
        </section>

        <AdminUsersPanel
          v-else-if="ui.section === 'users'"
          :users="model.admin.users"
          :loading="model.admin.loadingUsers"
          :saving="model.admin.savingUsers"
          :disabled="disabled"
          :refresh-users="model.loadAdminUsers"
          :create-user="createUser"
          :update-user="updateUser"
          :apply-bulk-action="applyUserAction"
          :confirm-action="confirmAction"
        />

        <AdminModerationPanel
          v-else-if="ui.section === 'moderation'"
          :prices="model.admin.moderationPrices"
          :loading="model.admin.loadingModeration"
          :disabled="disabled"
          :refresh-prices="model.loadModerationPrices"
          :approve-price="model.approveModerationPrice"
          :reject-price="model.rejectModerationPrice"
        />

        <AdminSettingsPanel
          v-else-if="ui.section === 'settings'"
          title="系统设置"
          description="与调试项分离，便于日常运维。"
          :settings="nonDebugSettings"
          :loading="loadingSettings"
          :saving-key="savingSetting"
          :disabled="disabled"
          :refresh-settings="refreshSettings"
          :save-setting="saveSetting"
        />

        <n-space v-else-if="ui.section === 'debug'" vertical size="large">
          <AdminSettingsPanel
            title="调试开关"
            description="调试相关配置单独存放。"
            :settings="debugSettings"
            :loading="loadingSettings"
            :saving-key="savingSetting"
            :disabled="disabled"
            :refresh-settings="refreshSettings"
            :save-setting="saveSetting"
          />

          <n-card title="调试模块" size="small" embedded>
            <n-tag type="info" size="small">
              {{ hiddenModulesEnabled ? "管理员可用" : "已由设置关闭" }}
            </n-tag>
          </n-card>

          <AdminExplorerPanel
            :model="model.workbench"
            :endpoint-groups="model.endpointGroups"
            :filtered-endpoints="model.filteredEndpoints"
            :selected-endpoint="model.selectedEndpoint"
            :explorer-error="model.explorerError"
            :busy="model.isBusy"
            :enabled="hiddenModulesEnabled"
            @send="model.sendExplorerRequest"
            @queue="model.queueExplorerRequest"
            @reset="model.resetExplorerDraft"
            @add-header="model.addExplorerHeader"
            @remove-header="model.removeExplorerHeader"
          />

          <ResponsePanel :history="model.history" />
        </n-space>

        <AdminEntityTablePanel
          v-else
          :table="currentTable"
          :rows="currentRows"
          :lookups="model.admin.lookups"
          :error="model.admin.error"
          :loading="model.admin.loadingTable"
          :saving="model.admin.savingRecord || model.admin.deletingRecord || model.admin.approvingRecord"
          :disabled="disabled"
          :refresh-table="model.refreshAdminTable"
          :save-record="saveAdminRecord"
          :delete-record="deleteAdminRecord"
          :approve-record="approveAdminRecord"
          :bulk-delete-records="bulkDeleteAdminRecords"
          :bulk-approve-records="bulkApproveAdminRecords"
          :confirm-action="confirmAction"
        />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>
