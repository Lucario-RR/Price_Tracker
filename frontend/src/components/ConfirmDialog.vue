<script setup>
defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: "Confirm action"
  },
  message: {
    type: String,
    default: ""
  },
  confirmLabel: {
    type: String,
    default: "Confirm"
  },
  cancelLabel: {
    type: String,
    default: "Cancel"
  },
  tone: {
    type: String,
    default: "danger"
  }
});

defineEmits(["confirm", "cancel"]);
</script>

<template>
  <transition name="confirm-fade">
    <div
      v-if="visible"
      class="confirm-modal-backdrop"
      @click.self="$emit('cancel')"
    >
      <section class="confirm-modal" role="alertdialog" aria-modal="true">
        <div class="confirm-modal-head">
          <div>
            <p class="section-kicker">Confirmation</p>
            <h2>{{ title }}</h2>
          </div>
          <span :class="['status-pill', tone === 'danger' ? 'tone-warn' : 'tone-info']">
            {{ tone === "danger" ? "Needs approval" : "Please confirm" }}
          </span>
        </div>

        <p class="helper-text">{{ message }}</p>

        <div class="confirm-modal-actions">
          <button class="ghost-button" type="button" @click="$emit('cancel')">
            {{ cancelLabel }}
          </button>
          <button
            :class="[tone === 'danger' ? 'danger-button' : 'primary-button']"
            type="button"
            @click="$emit('confirm')"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </section>
    </div>
  </transition>
</template>
