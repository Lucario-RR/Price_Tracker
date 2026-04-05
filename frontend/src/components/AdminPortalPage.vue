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
  <div class="admin-portal">
    <aside class="admin-sidebar panel">
      <div class="admin-sidebar-head">
        <div>
          <p class="eyebrow">Admin mode</p>
          <h2>Operations</h2>
        </div>
        <span :class="['status-pill', maintenanceMode ? 'tone-warn' : 'tone-good']">
          {{ maintenanceMode ? "Maintenance on" : "Maintenance off" }}
        </span>
      </div>

      <div class="simple-list">
        <div class="list-row">
          <strong>Signed in</strong>
          <span>{{ model.workbench.session.displayName || model.workbench.session.email }}</span>
        </div>
        <div class="list-row">
          <strong>Backend</strong>
          <span>{{ model.apiOnline ? "Reachable" : "Unavailable" }}</span>
        </div>
        <div class="list-row">
          <strong>Database</strong>
          <span>{{ model.databaseOnline ? "Connected" : "Unavailable" }}</span>
        </div>
      </div>

      <div v-for="group in navigationGroups" :key="group.label" class="admin-nav-group">
        <p class="admin-nav-label">{{ group.label }}</p>
        <button
          v-for="item in group.items"
          :key="item.key"
          :class="['admin-portal-link', ui.section === item.key ? 'is-active' : '']"
          @click="openSection(item.key)"
        >
          {{ item.label }}
        </button>
      </div>

      <div class="action-row wrap">
        <button class="ghost-button" :disabled="disabled" @click="refreshAll">
          Refresh portal
        </button>
        <button class="danger-button" @click="$emit('exit')">Return to normal mode</button>
      </div>
    </aside>

    <section class="admin-portal-main">
      <header class="admin-portal-header panel">
        <div>
          <p class="section-kicker">Separated admin portal</p>
          <h1>Daily management workspace</h1>
          <p class="helper-text">
            Operational modules live here, while normal user mode stays focused on public and personal tools.
          </p>
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
      </header>

      <section v-if="ui.section === 'dashboard'" class="stack-lg">
        <article class="panel">
          <div class="section-head">
            <div>
              <p class="section-kicker">Dashboard</p>
              <h2>Jump into the admin modules</h2>
            </div>
            <button class="ghost-button" :disabled="disabled" @click="refreshAll">
              Refresh counts
            </button>
          </div>

          <p class="helper-text">
            The dashboard is for visibility only. Open a module card to edit users, catalog tables, approvals, or settings.
          </p>

          <div class="admin-jump-grid">
            <button
              v-for="card in dashboardCards"
              :key="card.key"
              :class="['admin-jump-card', `summary-${card.tone}`]"
              @click="openSection(card.section)"
            >
              <span class="summary-label">{{ card.label }}</span>
              <strong class="summary-value">{{ card.value }}</strong>
              <span class="helper-text">{{ card.note }}</span>
            </button>
          </div>
        </article>
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
        title="System setting"
        description="System controls are separated from debug so daily admin work stays focused."
        :settings="nonDebugSettings"
        :loading="loadingSettings"
        :saving-key="savingSetting"
        :disabled="disabled"
        :refresh-settings="refreshSettings"
        :save-setting="saveSetting"
      />

      <section v-else-if="ui.section === 'debug'" class="stack-lg">
        <AdminSettingsPanel
          title="Debug setting"
          description="Debug controls stay in their own section and are not mixed into the regular admin pages."
          :settings="debugSettings"
          :loading="loadingSettings"
          :saving-key="savingSetting"
          :disabled="disabled"
          :refresh-settings="refreshSettings"
          :save-setting="saveSetting"
        />

        <article class="panel">
          <div class="section-head">
            <div>
              <p class="section-kicker">Debug</p>
              <h2>Explorer access</h2>
            </div>
            <span class="status-pill tone-info">
              {{ hiddenModulesEnabled ? "Enabled for admin" : "Disabled by setting" }}
            </span>
          </div>

          <p class="helper-text">
            Hidden and debug modules remain outside the daily admin workflow and live only in this dedicated section.
          </p>
        </article>

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
      </section>

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
    </section>
  </div>
</template>
