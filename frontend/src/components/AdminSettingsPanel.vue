<script setup>
import { computed, reactive, watch } from "vue";

const props = defineProps({
  title: {
    type: String,
    default: "Settings"
  },
  description: {
    type: String,
    default: ""
  },
  settings: {
    type: Array,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  savingKey: {
    type: String,
    default: ""
  },
  disabled: {
    type: Boolean,
    default: false
  },
  refreshSettings: {
    type: Function,
    required: true
  },
  saveSetting: {
    type: Function,
    required: true
  }
});

const drafts = reactive({});
const ui = reactive({
  category: "all"
});

function formatDraftValue(setting) {
  const value = setting.value;
  const type = String(setting.valueType || "json").toLowerCase();

  if (type === "boolean") {
    return Boolean(value);
  }

  if (type === "integer" || type === "number") {
    return value === null || value === undefined ? "" : String(value);
  }

  if (type === "string") {
    return value === null || value === undefined ? "" : String(value);
  }

  return JSON.stringify(value ?? null, null, 2);
}

function resetDrafts(settings) {
  Object.keys(drafts).forEach((key) => {
    delete drafts[key];
  });

  settings.forEach((setting) => {
    drafts[setting.key] = formatDraftValue(setting);
  });
}

watch(
  () => props.settings,
  (settings) => {
    resetDrafts(settings);
  },
  { immediate: true, deep: true }
);

const categories = computed(() => {
  const values = new Set(
    props.settings.map((setting) => {
      const [prefix = "general"] = String(setting.key || "general").split(".");
      return prefix;
    })
  );

  return ["all", ...values];
});

const filteredSettings = computed(() => {
  if (ui.category === "all") {
    return props.settings;
  }

  return props.settings.filter((setting) =>
    String(setting.key || "").startsWith(`${ui.category}.`)
  );
});

function parseDraft(setting) {
  const type = String(setting.valueType || "json").toLowerCase();
  const raw = drafts[setting.key];

  if (type === "boolean") {
    return Boolean(raw);
  }

  if (type === "integer") {
    return raw === "" ? null : Number.parseInt(raw, 10);
  }

  if (type === "number") {
    return raw === "" ? null : Number(raw);
  }

  if (type === "string") {
    return raw;
  }

  return raw?.trim() ? JSON.parse(raw) : null;
}

async function submitSetting(setting) {
  await props.saveSetting(setting, parseDraft(setting));
}

function resetSetting(setting) {
  drafts[setting.key] = formatDraftValue(setting);
}

function updatedText(setting) {
  return setting.updatedAt
    ? new Date(setting.updatedAt).toLocaleString()
    : "Using default value";
}
</script>

<template>
  <article class="panel admin-module">
    <div class="section-head">
      <div>
        <p class="section-kicker">Settings</p>
        <h2>{{ title }}</h2>
      </div>
      <button class="ghost-button" :disabled="disabled || loading" @click="refreshSettings">
        Refresh
      </button>
    </div>

    <p class="helper-text">
      {{ description || "These values are stored in the backend system settings tables." }}
    </p>

    <div v-if="categories.length > 1" class="tab-strip">
      <button
        v-for="category in categories"
        :key="category"
        :class="['tab-button', ui.category === category ? 'is-active' : '']"
        @click="ui.category = category"
      >
        {{ category === "all" ? "All categories" : category }}
      </button>
    </div>

    <div v-if="filteredSettings.length" class="settings-grid">
      <article v-for="setting in filteredSettings" :key="setting.key" class="setting-card">
        <div class="inline-head">
          <div>
            <strong>{{ setting.key }}</strong>
            <p class="helper-text">{{ setting.description || "No description provided." }}</p>
          </div>
          <span class="status-pill tone-info">{{ setting.valueType }}</span>
        </div>

        <label v-if="setting.valueType === 'boolean'" class="field checkbox-stack">
          <span>Value</span>
          <input v-model="drafts[setting.key]" :disabled="disabled" type="checkbox" />
        </label>

        <label v-else-if="setting.valueType === 'integer' || setting.valueType === 'number'" class="field">
          <span>Value</span>
          <input v-model="drafts[setting.key]" :disabled="disabled" inputmode="decimal" type="number" />
        </label>

        <label v-else-if="setting.valueType === 'string'" class="field">
          <span>Value</span>
          <input v-model="drafts[setting.key]" :disabled="disabled" />
        </label>

        <label v-else class="field">
          <span>JSON value</span>
          <textarea v-model="drafts[setting.key]" :disabled="disabled" rows="5" />
        </label>

        <div class="pill-row">
          <span class="status-pill tone-good">
            {{ setting.isSensitive ? "Sensitive" : "Visible to admin" }}
          </span>
          <span class="status-pill tone-accent">{{ updatedText(setting) }}</span>
        </div>

        <div class="action-row wrap">
          <button :disabled="disabled || savingKey === setting.key" @click="submitSetting(setting)">
            {{ savingKey === setting.key ? "Saving..." : "Save setting" }}
          </button>
          <button class="ghost-button" :disabled="disabled" @click="resetSetting(setting)">
            Reset draft
          </button>
        </div>
      </article>
    </div>

    <p v-else class="helper-text">
      {{ loading ? "Loading system settings..." : "No system settings were returned for this category." }}
    </p>
  </article>
</template>
