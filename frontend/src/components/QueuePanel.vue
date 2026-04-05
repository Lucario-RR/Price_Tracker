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
      title: "Remove queued request?",
      message: `Remove ${entry.label} from the retry queue?`,
      confirmLabel: "Remove",
      cancelLabel: "Keep queued",
      tone: "danger"
    })
  ) {
    emit("remove", entry.id);
  }
}

async function confirmQueueClear() {
  if (
    await props.confirmAction({
      title: "Clear retry queue?",
      message: `Clear all ${props.queue.length} entries from the retry queue?`,
      confirmLabel: "Clear queue",
      cancelLabel: "Keep queue",
      tone: "danger"
    })
  ) {
    emit("clear");
  }
}
</script>

<template>
  <article class="panel">
    <div class="section-head">
      <div>
        <p class="section-kicker">Queue</p>
        <h2>Offline-safe retry lane</h2>
      </div>
      <span :class="['status-pill', online ? 'tone-good' : 'tone-warn']">
        {{ online ? "Ready to flush" : "Waiting for browser or API" }}
      </span>
    </div>

    <p class="helper-text">
      Requests queued while the browser or backend is unavailable retry automatically when service returns.
      Ambiguous mid-flight failures are held for manual review so we do not duplicate writes.
    </p>

    <div class="action-row wrap">
      <button :disabled="!online || flushing || !queue.length" @click="$emit('flush')">
        {{ flushing ? "Flushing..." : "Flush queue now" }}
      </button>
      <button class="ghost-button" :disabled="!queue.length" @click="confirmQueueClear">
        Clear queue
      </button>
    </div>

    <div v-if="queue.length" class="queue-list">
      <article v-for="entry in queue" :key="entry.id" class="queue-card">
        <div class="queue-head">
          <strong>{{ entry.label }}</strong>
          <span :class="['status-pill', entry.status === 'queued' ? 'tone-info' : 'tone-warn']">
            {{ entry.status }}
          </span>
        </div>
        <p class="helper-text">
          {{ entry.kind === "priceCapture" ? "Workflow replay" : "Request replay" }}
          · attempts: {{ entry.attemptCount }}
        </p>
        <p v-if="entry.lastError" class="error-text">{{ entry.lastError }}</p>
        <div class="action-row wrap">
          <button class="ghost-button" :disabled="!online" @click="$emit('retry', entry.id)">
            Retry
          </button>
          <button class="danger-button" @click="confirmQueueRemoval(entry)">
            Remove
          </button>
        </div>
      </article>
    </div>

    <p v-else class="helper-text">The retry queue is clear.</p>
  </article>
</template>
