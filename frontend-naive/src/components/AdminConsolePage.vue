<script setup>
import { onMounted, reactive } from "vue";

import AdminDatabasePanel from "./AdminDatabasePanel.vue";
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
  hiddenModulesEnabled: {
    type: Boolean,
    default: true
  },
  maintenanceMode: {
    type: Boolean,
    default: false
  },
  settingMap: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([
  "refresh-all",
  "refresh-settings",
  "save-setting",
  "create-user",
  "bulk-user-action"
]);

const ui = reactive({
  revealDebug: false
});

function forwardSave(...args) {
  emit("save-setting", ...args);
}

onMounted(() => {
  if (props.model.isAdmin) {
    emit("refresh-all");
  }
});
</script>

<template>
  <section class="stack-lg">
    <section class="dashboard-hero">
      <div>
        <p class="eyebrow">Admin console</p>
        <h2>Operational controls</h2>
        <p class="helper-text">
          Moderation, user management, system settings, and database helpers live here so regular users never load them.
        </p>
      </div>

      <div class="action-row wrap">
        <button :disabled="disabled" @click="$emit('refresh-all')">
          {{ disabled ? "Refreshing..." : "Refresh admin page" }}
        </button>
        <span :class="['status-pill', maintenanceMode ? 'tone-warn' : 'tone-good']">
          {{ maintenanceMode ? "Maintenance mode on" : "Maintenance mode off" }}
        </span>
      </div>
    </section>

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

    <AdminUsersPanel
      :users="model.admin.users"
      :loading="model.admin.loadingUsers"
      :saving="model.admin.savingUsers"
      :disabled="disabled"
      @refresh="model.loadAdminUsers"
      @create-user="$emit('create-user', $event)"
      @bulk-action="$emit('bulk-user-action', $event)"
    />

    <AdminModerationPanel
      :prices="model.admin.moderationPrices"
      :loading="model.admin.loadingModeration"
      :disabled="disabled"
      @refresh="model.loadModerationPrices"
      @approve="model.approveModerationPrice"
      @reject="model.rejectModerationPrice"
    />

    <AdminSettingsPanel
      :settings="settings"
      :loading="loadingSettings"
      :saving-key="savingSetting"
      :disabled="disabled"
      @refresh="$emit('refresh-settings')"
      @save="forwardSave"
    />

    <AdminDatabasePanel
      :tables="model.adminTables"
      :table="model.adminTable"
      :admin="model.admin"
      :disabled="disabled"
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
        <span class="status-pill tone-info">
          {{ hiddenModulesEnabled ? "Available to admin" : "Disabled" }}
        </span>
      </div>

      <div class="simple-list">
        <div class="list-row"><strong>Maintenance mode</strong><span>{{ maintenanceMode ? "On" : "Off" }}</span></div>
        <div class="list-row"><strong>Hidden modules</strong><span>{{ hiddenModulesEnabled ? "Enabled" : "Disabled" }}</span></div>
        <div class="list-row"><strong>Banner text</strong><span>{{ settingMap["ui.publishBannerText"] || "Not set" }}</span></div>
      </div>

      <div class="action-row wrap">
        <button :disabled="!hiddenModulesEnabled" @click="ui.revealDebug = !ui.revealDebug">
          {{ ui.revealDebug ? "Hide debug modules" : "Reveal debug modules" }}
        </button>
      </div>
    </article>

    <section v-if="ui.revealDebug && hiddenModulesEnabled" class="stack-lg">
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
  </section>
</template>
