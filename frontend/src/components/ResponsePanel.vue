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

  return entry.ok ? "Local event" : "Failed locally";
}

function summaryLabel(entry) {
  const raw = String(entry.responsePretty || "").trim();
  if (!raw) {
    return entry.ok ? "Completed without a response body." : "No response body returned.";
  }

  return raw.split("\n")[0].slice(0, 120);
}
</script>

<template>
  <article class="panel">
    <div class="section-head">
      <div>
        <p class="section-kicker">Responses</p>
        <h2>Latest result and activity trail</h2>
      </div>
    </div>

    <template v-if="latest">
      <div class="response-meta">
        <span class="status-pill tone-info">{{ latest.method }}</span>
        <span
          :class="[
            'status-pill',
            latest.ok ? 'tone-good' : latest.status ? 'tone-warn' : 'tone-info'
          ]"
        >
          {{ latest.status ? `HTTP ${latest.status}` : "Local event" }}
        </span>
        <span class="helper-text">{{ latest.durationMs }} ms</span>
      </div>

      <p class="helper-text">{{ latest.label }}</p>
      <code class="response-url">{{ latest.url }}</code>

      <div class="response-stack">
        <div>
          <h3>Request body</h3>
          <pre>{{ latest.requestBodyText || "(no body)" }}</pre>
        </div>
        <div>
          <h3>Response</h3>
          <pre>{{ latest.responsePretty || "(empty)" }}</pre>
        </div>
      </div>
    </template>

    <p v-else class="helper-text">Send a request or save a capture to see activity here.</p>

    <div v-if="history.length" class="history-list">
      <div v-for="entry in history.slice(0, 8)" :key="entry.id" class="history-item">
        <div>
          <strong>{{ entry.label }}</strong>
          <div class="table-subtext">{{ summaryLabel(entry) }}</div>
        </div>
        <div class="history-meta">
          <span :class="['status-pill', entry.ok ? 'tone-good' : 'tone-warn']">
            {{ statusLabel(entry) }}
          </span>
          <span>{{ new Date(entry.startedAt).toLocaleString() }}</span>
        </div>
      </div>
    </div>
  </article>
</template>
