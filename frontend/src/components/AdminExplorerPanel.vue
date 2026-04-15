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

defineEmits([
  "send",
  "queue",
  "reset",
  "add-header",
  "remove-header"
]);

function requestMethods() {
  return ["GET", "POST", "PATCH", "DELETE"];
}
</script>

<template>
  <article class="panel admin-module">
    <div class="section-head">
      <div>
        <p class="section-kicker">Debug</p>
        <h2>API explorer</h2>
      </div>
      <span :class="['status-pill', enabled ? 'tone-info' : 'tone-warn']">
        {{ enabled ? "Enabled" : "Hidden for production" }}
      </span>
    </div>

    <p v-if="!enabled" class="helper-text">
      The admin dashboard is still available, but the raw explorer stays hidden unless
      <code>NUXT_PUBLIC_ENABLE_DEBUG_TOOLS=true</code>.
    </p>

    <div v-else class="explorer-layout">
      <aside class="endpoint-browser">
        <label class="field">
          <span>Group</span>
          <select v-model="model.explorer.selectedGroup">
            <option v-for="group in endpointGroups" :key="group" :value="group">
              {{ group }}
            </option>
          </select>
        </label>

        <label class="field">
          <span>Search</span>
          <input
            v-model="model.explorer.search"
            placeholder="Search label, path, or description"
          />
        </label>

        <div class="endpoint-list">
          <button
            v-for="endpoint in filteredEndpoints"
            :key="endpoint.id"
            :class="[
              'endpoint-card',
              endpoint.id === selectedEndpoint.id ? 'is-selected' : ''
            ]"
            @click="model.explorer.selectedEndpointId = endpoint.id"
          >
            <span class="endpoint-method">{{ endpoint.method }}</span>
            <strong>{{ endpoint.label }}</strong>
            <code>{{ endpoint.path }}</code>
          </button>
        </div>
      </aside>

      <article class="endpoint-editor">
        <div class="endpoint-editor-head">
          <div>
            <h3>{{ selectedEndpoint.label }}</h3>
            <p>{{ selectedEndpoint.description }}</p>
          </div>
          <span class="status-pill tone-info">{{ selectedEndpoint.group }}</span>
        </div>

        <div class="field-grid two-up">
          <label v-if="selectedEndpoint.id === 'custom'" class="field">
            <span>Method</span>
            <select v-model="model.explorer.customMethod">
              <option v-for="method in requestMethods()" :key="method" :value="method">
                {{ method }}
              </option>
            </select>
          </label>

          <label v-if="selectedEndpoint.id === 'custom'" class="field full-span">
            <span>Path</span>
            <input v-model="model.explorer.customPath" placeholder="/health" />
          </label>

          <label
            v-for="pathParam in selectedEndpoint.pathParams || []"
            :key="`path-${pathParam}`"
            class="field"
          >
            <span>{{ pathParam }}</span>
            <input v-model="model.explorer.pathParams[pathParam]" />
          </label>

          <label
            v-for="queryParam in selectedEndpoint.queryParams || []"
            :key="`query-${queryParam}`"
            class="field"
          >
            <span>{{ queryParam }}</span>
            <input v-model="model.explorer.queryParams[queryParam]" />
          </label>
        </div>

        <div class="inline-head">
          <h4>Extra headers</h4>
          <button class="ghost-button" @click="$emit('add-header')">Add header</button>
        </div>

        <div v-if="model.explorer.extraHeaders.length" class="stack">
          <div
            v-for="(header, index) in model.explorer.extraHeaders"
            :key="`header-${index}`"
            class="header-row"
          >
            <input v-model="header.key" placeholder="Header name" />
            <input v-model="header.value" placeholder="Header value" />
            <button class="danger-button" @click="$emit('remove-header', index)">
              Remove
            </button>
          </div>
        </div>

        <label class="field">
          <span>JSON body</span>
          <textarea
            v-model="model.explorer.bodyText"
            rows="10"
            placeholder="Body goes here for POST or PATCH routes"
          />
        </label>

        <label class="checkbox-field">
          <input v-model="model.explorer.queueOnFailure" type="checkbox" />
          <span>Queue writes if the browser or backend is unavailable</span>
        </label>

        <p v-if="explorerError" class="error-text">{{ explorerError }}</p>

        <div class="action-row wrap">
          <button :disabled="busy" @click="$emit('send')">Send request</button>
          <button class="ghost-button" :disabled="busy" @click="$emit('queue')">
            Save to retry queue
          </button>
          <button class="ghost-button" :disabled="busy" @click="$emit('reset')">
            Reset template
          </button>
        </div>
      </article>
    </div>
  </article>
</template>
