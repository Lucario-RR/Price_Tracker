<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";

import {
  listCameraDevices,
  scanImageFile,
  startLiveScanner,
  stopLiveScanner
} from "../lib/scanner";

const props = defineProps({
  model: {
    type: Object,
    required: true
  },
  shopId: {
    type: String,
    default: ""
  },
  disabled: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(["capture", "lookup"]);
const cameraDevices = ref([]);
const videoRef = ref(null);
const isScanning = ref(false);
const scanError = ref("");
const loadingCameras = ref(false);
const lastFingerprint = ref("");

async function loadCameras() {
  loadingCameras.value = true;
  scanError.value = "";

  try {
    cameraDevices.value = await listCameraDevices();

    if (!props.model.selectedDeviceId && cameraDevices.value[0]?.deviceId) {
      props.model.selectedDeviceId = cameraDevices.value[0].deviceId;
    }
  } catch (error) {
    scanError.value = String(error.message || error);
  } finally {
    loadingCameras.value = false;
  }
}

function registerCapture(payload) {
  const fingerprint = `${payload.source}:${payload.format}:${payload.text}`;

  if (lastFingerprint.value === fingerprint) {
    return;
  }

  lastFingerprint.value = fingerprint;
  props.model.lookupCode = payload.text;
  props.model.manualCode = payload.text;
  props.model.recentCaptures = [
    payload,
    ...props.model.recentCaptures.filter(
      (entry) => entry.text !== payload.text || entry.source !== payload.source
    )
  ].slice(0, 6);
  emit("capture", payload);
}

async function startScanning() {
  if (props.disabled) {
    return;
  }

  scanError.value = "";
  await nextTick();

  if (!videoRef.value) {
    return;
  }

  try {
    await startLiveScanner({
      deviceId: props.model.selectedDeviceId,
      videoElement: videoRef.value,
      onResult: registerCapture,
      onError: (error) => {
        scanError.value = String(error.message || error);
      }
    });
    isScanning.value = true;
  } catch (error) {
    isScanning.value = false;
    scanError.value = String(error.message || error);
  }
}

function stopScanning() {
  stopLiveScanner();
  isScanning.value = false;
}

async function handleImageSelection(event) {
  const [file] = event.target.files || [];

  if (!file) {
    return;
  }

  scanError.value = "";

  try {
    registerCapture(await scanImageFile(file));
  } catch (error) {
    scanError.value = String(error.message || error);
  } finally {
    event.target.value = "";
  }
}

function useManualCode() {
  if (!props.model.manualCode?.trim()) {
    scanError.value = "Type a code manually before saving it.";
    return;
  }

  scanError.value = "";
  registerCapture({
    text: props.model.manualCode.trim(),
    format: "MANUAL",
    source: "manual-entry",
    capturedAt: new Date().toISOString()
  });
}

onMounted(loadCameras);
onBeforeUnmount(stopScanning);
</script>

<template>
  <article class="panel scanner-panel">
    <div class="section-head">
      <div>
        <p class="section-kicker">Scanner</p>
        <h2>Live scan, image recognition, and manual fallback</h2>
      </div>
      <button class="ghost-button" :disabled="disabled || loadingCameras" @click="loadCameras">
        Refresh cameras
      </button>
    </div>

    <p class="helper-text">
      The camera and image upload flow uses the ZXing multi-format browser reader,
      then falls back to manual entry when a code is hard to detect.
    </p>

    <label class="field">
      <span>Camera device</span>
      <select v-model="model.selectedDeviceId">
        <option value="">Default camera</option>
        <option v-for="device in cameraDevices" :key="device.deviceId" :value="device.deviceId">
          {{ device.label || device.deviceId }}
        </option>
      </select>
    </label>

    <div class="scanner-preview">
      <video ref="videoRef" muted playsinline />
    </div>

    <div class="action-row wrap">
      <button :disabled="disabled || isScanning" @click="startScanning">Start camera</button>
      <button class="ghost-button" :disabled="!isScanning" @click="stopScanning">
        Stop camera
      </button>
      <label class="ghost-button file-button">
        <span>Scan image</span>
        <input accept="image/*" type="file" @change="handleImageSelection" />
      </label>
    </div>

    <div class="field-grid two-up">
      <label class="field">
        <span>Current code</span>
        <input
          v-model="model.lookupCode"
          placeholder="Filled from camera, image upload, or manual entry"
        />
      </label>
      <label class="field">
        <span>Manual code entry</span>
        <input
          v-model="model.manualCode"
          placeholder="Type a barcode or product code"
          @keyup.enter.prevent="useManualCode"
        />
      </label>
    </div>

    <div class="action-row wrap">
      <button class="ghost-button" :disabled="disabled" @click="useManualCode">
        Use manual code
      </button>
      <button
        class="ghost-button"
        :disabled="disabled || !model.lookupCode || !shopId"
        @click="$emit('lookup')"
      >
        Lookup code in selected shop
      </button>
    </div>

    <p v-if="scanError" class="error-text">{{ scanError }}</p>

    <div v-if="model.recentCaptures.length" class="capture-list">
      <div v-for="capture in model.recentCaptures" :key="capture.capturedAt" class="capture-card">
        <span class="status-pill tone-info">{{ capture.format }}</span>
        <strong>{{ capture.text }}</strong>
        <small>{{ new Date(capture.capturedAt).toLocaleString() }}</small>
      </div>
    </div>
  </article>
</template>
