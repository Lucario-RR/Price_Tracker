<script setup>
const props = defineProps({
  queue: {
    type: Array,
    required: true
  },
  online: {
    type: Boolean,
    default: false
  },
  flushing: {
    type: Boolean,
    default: false
  },
  confirmAction: {
    type: Function,
    required: true
  }
});

const emit = defineEmits(["flush", "retry", "remove", "clear"]);

async function confirmQueueRemoval(entry) {
  if (
    await props.confirmAction({
      title: "移除队列项？",
      message: `从重试队列移除「${entry.label}」？`,
      confirmLabel: "移除",
      cancelLabel: "保留",
      tone: "danger"
    })
  ) {
    emit("remove", entry.id);
  }
}

async function confirmQueueClear() {
  if (
    await props.confirmAction({
      title: "清空队列？",
      message: `清空全部 ${props.queue.length} 条队列记录？`,
      confirmLabel: "清空",
      cancelLabel: "保留",
      tone: "danger"
    })
  ) {
    emit("clear");
  }
}
</script>

<template>
  <n-card title="离线重试队列" :segmented="{ content: true }">
    <template #header-extra>
      <n-tag :type="online ? 'success' : 'warning'" size="small">
        {{ online ? "可刷新" : "等待网络或后端" }}
      </n-tag>
    </template>

    <n-text depth="3" style="display: block; margin-top: 0">
      浏览器或后端不可用时写入会排队，恢复后自动重试；不确定的失败会保留以便人工处理。
    </n-text>

    <n-space style="margin-bottom: 12px">
      <n-button type="primary" :disabled="!online || flushing || !queue.length" @click="$emit('flush')">
        {{ flushing ? "正在刷新…" : "立即刷新队列" }}
      </n-button>
      <n-button quaternary :disabled="!queue.length" @click="confirmQueueClear">清空队列</n-button>
    </n-space>

    <n-empty v-if="!queue.length" description="队列为空" />

    <n-space v-else vertical size="medium" style="width: 100%">
      <n-card
        v-for="entry in queue"
        :key="entry.id"
        size="small"
        :title="entry.label"
        embedded
        bordered
      >
        <n-space vertical size="small">
          <n-space align="center">
            <n-tag size="tiny" type="info">{{ entry.kind === "priceCapture" ? "工作流回放" : "请求回放" }}</n-tag>
            <n-text depth="3">尝试次数：{{ entry.attemptCount }}</n-text>
          </n-space>
          <n-alert v-if="entry.lastError" type="error" :title="entry.lastError" />
          <n-space>
            <n-button size="small" :disabled="!online" @click="$emit('retry', entry.id)">重试</n-button>
            <n-button size="small" type="error" tertiary @click="confirmQueueRemoval(entry)">移除</n-button>
          </n-space>
        </n-space>
      </n-card>
    </n-space>
  </n-card>
</template>
