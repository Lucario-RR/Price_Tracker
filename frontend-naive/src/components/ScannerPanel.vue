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
const imageInputRef = ref(null);

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
    scanError.value = "请先输入条码或商品编码。";
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

const cameraOptions = () =>
  cameraDevices.value.map((d) => ({
    label: d.label || d.deviceId,
    value: d.deviceId
  }));
</script>

<template>
  <n-card title="扫码与识别" :segmented="{ content: true }">
    <template #header-extra>
      <n-button size="small" quaternary :disabled="disabled || loadingCameras" @click="loadCameras">
        刷新摄像头
      </n-button>
    </template>

    <n-text depth="3" style="display: block; margin-top: 0">
      使用 ZXing 进行实时扫码与图片识别，无法识别时可手动输入。
    </n-text>

    <n-form-item label="摄像头设备">
      <n-select
        v-model:value="model.selectedDeviceId"
        :options="[{ label: '默认摄像头', value: '' }, ...cameraOptions()]"
        :disabled="disabled"
        placeholder="选择设备"
        clearable
      />
    </n-form-item>

    <div class="scanner-video-wrap">
      <video ref="videoRef" class="scanner-video" muted playsinline />
    </div>

    <n-space style="margin: 12px 0" wrap>
      <n-button type="primary" :disabled="disabled || isScanning" @click="startScanning">开启摄像头</n-button>
      <n-button :disabled="!isScanning" @click="stopScanning">停止</n-button>
      <input
        ref="imageInputRef"
        accept="image/*"
        type="file"
        style="display: none"
        @change="handleImageSelection"
      />
      <n-button @click="imageInputRef?.click()">识别图片</n-button>
    </n-space>

    <n-grid :cols="2" :x-gap="12" responsive="screen">
      <n-gi span="2 m:1">
        <n-form-item label="当前编码">
          <n-input
            v-model:value="model.lookupCode"
            :disabled="disabled"
            placeholder="来自摄像头、图片或手动输入"
          />
        </n-form-item>
      </n-gi>
      <n-gi span="2 m:1">
        <n-form-item label="手动输入">
          <n-input
            v-model:value="model.manualCode"
            :disabled="disabled"
            placeholder="条码或商品编码"
            @keyup.enter.prevent="useManualCode"
          />
        </n-form-item>
      </n-gi>
    </n-grid>

    <n-space wrap>
      <n-button :disabled="disabled" @click="useManualCode">使用手动编码</n-button>
      <n-button :disabled="disabled || !model.lookupCode || !shopId" @click="$emit('lookup')">
        在已选商店中查询
      </n-button>
    </n-space>

    <n-alert v-if="scanError" type="error" :title="scanError" style="margin-top: 12px" />

    <n-divider v-if="model.recentCaptures.length" title-placement="left">最近捕获</n-divider>

    <n-space v-if="model.recentCaptures.length" vertical>
      <n-card
        v-for="capture in model.recentCaptures"
        :key="capture.capturedAt + capture.text"
        size="small"
        embedded
      >
        <n-space align="center">
          <n-tag size="small">{{ capture.format }}</n-tag>
          <n-text strong>{{ capture.text }}</n-text>
          <n-text depth="3">{{ new Date(capture.capturedAt).toLocaleString() }}</n-text>
        </n-space>
      </n-card>
    </n-space>
  </n-card>
</template>

<style scoped>
.scanner-video-wrap {
  border-radius: 8px;
  overflow: hidden;
  background: #000;
}
.scanner-video {
  width: 100%;
  max-height: 320px;
  display: block;
}
</style>
