<script setup>
import { computed } from "vue";

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: "确认操作"
  },
  message: {
    type: String,
    default: ""
  },
  confirmLabel: {
    type: String,
    default: "确认"
  },
  cancelLabel: {
    type: String,
    default: "取消"
  },
  tone: {
    type: String,
    default: "danger"
  }
});

defineEmits(["confirm", "cancel"]);

const confirmType = computed(() => (props.tone === "danger" ? "error" : "primary"));
</script>

<template>
  <n-modal
    :show="visible"
    preset="dialog"
    :title="title"
    :type="confirmType"
    :positive-text="confirmLabel"
    :negative-text="cancelLabel"
    @positive-click="$emit('confirm')"
    @negative-click="$emit('cancel')"
    @mask-click="$emit('cancel')"
  >
    <n-text depth="2">{{ message }}</n-text>
  </n-modal>
</template>
