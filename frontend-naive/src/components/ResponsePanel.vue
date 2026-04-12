<script setup>
import { computed } from "vue";

const props = defineProps({
  history: {
    type: Array,
    required: true
  }
});

const latest = computed(() => props.history[0] || null);

function statusLabel(entry) {
  if (!entry) {
    return "";
  }

  if (entry.status) {
    return `HTTP ${entry.status}`;
  }

  return entry.ok ? "本地事件" : "本地失败";
}

function summaryLabel(entry) {
  const raw = String(entry.responsePretty || "").trim();
  if (!raw) {
    return entry.ok ? "无响应体。" : "无响应体。";
  }

  return raw.split("\n")[0].slice(0, 120);
}
</script>

<template>
  <n-card title="请求与响应" :segmented="{ content: true }">
    <template v-if="latest">
      <n-space align="center" style="margin-bottom: 8px">
        <n-tag size="small" type="info">{{ latest.method }}</n-tag>
        <n-tag
          size="small"
          :type="latest.ok ? 'success' : latest.status ? 'warning' : 'default'"
        >
          {{ latest.status ? `HTTP ${latest.status}` : "本地事件" }}
        </n-tag>
        <n-text depth="3">{{ latest.durationMs }} ms</n-text>
      </n-space>

      <n-text depth="2">{{ latest.label }}</n-text>
      <n-code :code="latest.url" word-wrap style="display: block; margin: 8px 0" />

      <n-grid :cols="1" responsive="screen" style="margin-top: 12px">
        <n-gi>
          <n-text strong>请求体</n-text>
          <n-scrollbar style="max-height: 200px; margin-top: 4px">
            <pre style="margin: 0; white-space: pre-wrap">{{ latest.requestBodyText || "（无）" }}</pre>
          </n-scrollbar>
        </n-gi>
        <n-gi style="margin-top: 12px">
          <n-text strong>响应</n-text>
          <n-scrollbar style="max-height: 280px; margin-top: 4px">
            <pre style="margin: 0; white-space: pre-wrap">{{ latest.responsePretty || "（空）" }}</pre>
          </n-scrollbar>
        </n-gi>
      </n-grid>
    </template>

    <n-empty v-else description="发送请求或保存捕获后可在此查看" />

    <n-divider v-if="history.length" title-placement="left">最近记录</n-divider>

    <n-list v-if="history.length" bordered>
      <n-list-item v-for="entry in history.slice(0, 8)" :key="entry.id">
        <n-thing :title="entry.label" :description="summaryLabel(entry)">
          <template #header-extra>
            <n-space vertical align="end" size="small">
              <n-tag :type="entry.ok ? 'success' : 'warning'" size="small">{{ statusLabel(entry) }}</n-tag>
              <n-text depth="3" style="font-size: 12px">
                {{ new Date(entry.startedAt).toLocaleString() }}
              </n-text>
            </n-space>
          </template>
        </n-thing>
      </n-list-item>
    </n-list>
  </n-card>
</template>
