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
  <n-card :title="title" :segmented="{ content: true }">
    <template #header-extra>
      <n-button size="small" quaternary :disabled="disabled || loading" @click="refreshSettings">刷新</n-button>
    </template>

    <n-text depth="3" style="display: block; margin-bottom: 12px">
      {{ description || "以下值保存在后端系统设置中。" }}
    </n-text>

    <n-tabs v-if="categories.length > 1" v-model:value="ui.category" type="segment" size="small" style="margin-bottom: 16px">
      <n-tab-pane v-for="category in categories" :key="category" :name="category" :tab="category === 'all' ? '全部分类' : category" />
    </n-tabs>

    <n-spin :show="loading">
      <n-space v-if="filteredSettings.length" vertical size="large" style="width: 100%">
        <n-card v-for="setting in filteredSettings" :key="setting.key" size="small" embedded :title="setting.key">
          <template #header-extra>
            <n-tag size="small" type="info">{{ setting.valueType }}</n-tag>
          </template>

          <n-text depth="3" style="display: block; margin-bottom: 12px">
            {{ setting.description || "无描述" }}
          </n-text>

          <n-checkbox
            v-if="setting.valueType === 'boolean'"
            v-model:checked="drafts[setting.key]"
            :disabled="disabled"
          >
            启用
          </n-checkbox>

          <n-input
            v-else-if="setting.valueType === 'integer' || setting.valueType === 'number'"
            v-model:value="drafts[setting.key]"
            :disabled="disabled"
            placeholder="数字"
          />

          <n-input v-else-if="setting.valueType === 'string'" v-model:value="drafts[setting.key]" :disabled="disabled" />

          <n-input
            v-else
            v-model:value="drafts[setting.key]"
            type="textarea"
            :rows="5"
            :disabled="disabled"
            placeholder="JSON"
          />

          <n-space style="margin-top: 12px">
            <n-tag :type="setting.isSensitive ? 'warning' : 'success'" size="small">
              {{ setting.isSensitive ? "敏感" : "管理员可见" }}
            </n-tag>
            <n-tag type="default" size="small">{{ updatedText(setting) }}</n-tag>
          </n-space>

          <n-space style="margin-top: 12px">
            <n-button type="primary" :disabled="disabled || savingKey === setting.key" @click="submitSetting(setting)">
              {{ savingKey === setting.key ? "保存中…" : "保存" }}
            </n-button>
            <n-button quaternary :disabled="disabled" @click="resetSetting(setting)">重置草稿</n-button>
          </n-space>
        </n-card>
      </n-space>

      <n-empty v-else :description="loading ? '加载系统设置…' : '该分类下没有返回设置项'" />
    </n-spin>
  </n-card>
</template>
