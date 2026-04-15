<script setup>
defineProps({
  model: {
    type: Object,
    required: true
  },
  endpointGroups: {
    type: Array,
    required: true
  },
  filteredEndpoints: {
    type: Array,
    required: true
  },
  selectedEndpoint: {
    type: Object,
    required: true
  },
  explorerError: {
    type: String,
    default: ""
  },
  busy: {
    type: Boolean,
    default: false
  },
  enabled: {
    type: Boolean,
    default: false
  }
});

defineEmits(["send", "queue", "reset", "add-header", "remove-header"]);

function requestMethods() {
  return ["GET", "POST", "PATCH", "DELETE"];
}
</script>

<template>
  <n-card title="API 调试浏览器" :segmented="{ content: true }">
    <template #header-extra>
      <n-tag :type="enabled ? 'info' : 'warning'" size="small">
        {{ enabled ? "已启用" : "生产环境隐藏" }}
      </n-tag>
    </template>

    <n-alert v-if="!enabled" type="warning" title="已关闭" style="margin-bottom: 12px">
      需在环境变量中设置 VITE_ENABLE_DEBUG_TOOLS=true 方可使用原始调试工具。
    </n-alert>

    <n-layout v-else has-sider sider-placement="left" style="min-height: 420px; background: transparent">
      <n-layout-sider
        bordered
        content-style="padding: 12px"
        :native-scrollbar="false"
        width="min(320px, 38vw)"
      >
        <n-space vertical>
          <n-form-item label="分组">
            <n-select v-model:value="model.explorer.selectedGroup" :options="endpointGroups.map((g) => ({ label: g, value: g }))" />
          </n-form-item>
          <n-form-item label="搜索">
            <n-input v-model:value="model.explorer.search" clearable placeholder="标签、路径或描述" />
          </n-form-item>
        </n-space>
        <n-scrollbar style="max-height: 360px; margin-top: 8px">
          <n-space vertical size="small">
            <n-button
              v-for="endpoint in filteredEndpoints"
              :key="endpoint.id"
              block
              secondary
              :type="endpoint.id === selectedEndpoint.id ? 'primary' : 'default'"
              size="small"
              style="height: auto; text-align: left"
              @click="model.explorer.selectedEndpointId = endpoint.id"
            >
              <n-space vertical size="small" align="start">
                <n-tag size="tiny" type="info">{{ endpoint.method }}</n-tag>
                <n-text strong>{{ endpoint.label }}</n-text>
                <n-code :code="endpoint.path" />
              </n-space>
            </n-button>
          </n-space>
        </n-scrollbar>
      </n-layout-sider>

      <n-layout-content content-style="padding: 12px 16px">
        <n-space vertical size="medium" style="width: 100%">
          <n-space justify="space-between" align="start">
            <n-space vertical>
              <n-text tag="h3" style="margin: 0; font-size: 1.15rem">{{ selectedEndpoint.label }}</n-text>
              <n-text depth="2">{{ selectedEndpoint.description }}</n-text>
            </n-space>
            <n-tag size="small">{{ selectedEndpoint.group }}</n-tag>
          </n-space>

          <n-grid :cols="2" :x-gap="12" responsive="screen">
            <n-gi v-if="selectedEndpoint.id === 'custom'" span="2 m:1">
              <n-form-item label="方法">
                <n-select
                  v-model:value="model.explorer.customMethod"
                  :options="requestMethods().map((m) => ({ label: m, value: m }))"
                />
              </n-form-item>
            </n-gi>
            <n-gi v-if="selectedEndpoint.id === 'custom'" span="2">
              <n-form-item label="路径">
                <n-input v-model:value="model.explorer.customPath" placeholder="/health" />
              </n-form-item>
            </n-gi>
            <n-gi v-for="pathParam in selectedEndpoint.pathParams || []" :key="`path-${pathParam}`" span="2 m:1">
              <n-form-item :label="pathParam">
                <n-input v-model:value="model.explorer.pathParams[pathParam]" />
              </n-form-item>
            </n-gi>
            <n-gi v-for="queryParam in selectedEndpoint.queryParams || []" :key="`query-${queryParam}`" span="2 m:1">
              <n-form-item :label="queryParam">
                <n-input v-model:value="model.explorer.queryParams[queryParam]" />
              </n-form-item>
            </n-gi>
          </n-grid>

          <n-divider title-placement="left">附加请求头</n-divider>
          <n-button size="small" quaternary @click="$emit('add-header')">添加请求头</n-button>
          <n-space v-if="model.explorer.extraHeaders.length" vertical>
            <n-space v-for="(header, index) in model.explorer.extraHeaders" :key="`header-${index}`" align="center">
              <n-input v-model:value="header.key" placeholder="名称" style="flex: 1" />
              <n-input v-model:value="header.value" placeholder="值" style="flex: 1" />
              <n-button type="error" tertiary size="small" @click="$emit('remove-header', index)">删除</n-button>
            </n-space>
          </n-space>

          <n-form-item label="JSON 请求体">
            <n-input v-model:value="model.explorer.bodyText" type="textarea" placeholder="POST / PATCH 时使用" :rows="10" />
          </n-form-item>

          <n-checkbox v-model:checked="model.explorer.queueOnFailure">失败时写入重试队列</n-checkbox>

          <n-alert v-if="explorerError" type="error" :title="explorerError" />

          <n-space>
            <n-button type="primary" :disabled="busy" @click="$emit('send')">发送</n-button>
            <n-button :disabled="busy" @click="$emit('queue')">加入队列</n-button>
            <n-button quaternary :disabled="busy" @click="$emit('reset')">重置模板</n-button>
          </n-space>
        </n-space>
      </n-layout-content>
    </n-layout>
  </n-card>
</template>
