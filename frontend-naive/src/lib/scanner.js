let activeControls = null;
let browserModulePromise = null;
let browserCodeReaderRef = null;

async function loadBrowserModule() {
  if (!browserModulePromise) {
    browserModulePromise = import("@zxing/browser");
  }

  const module = await browserModulePromise;
  browserCodeReaderRef = module.BrowserCodeReader;
  return module;
}

function createResultPayload(result, source, barcodeFormatRef) {
  return {
    text: result.getText(),
    format: barcodeFormatRef[result.getBarcodeFormat()] || "UNKNOWN",
    source,
    capturedAt: new Date().toISOString()
  };
}

export async function listCameraDevices() {
  const { BrowserCodeReader } = await loadBrowserModule();
  return BrowserCodeReader.listVideoInputDevices();
}

export async function startLiveScanner({ deviceId, videoElement, onResult, onError }) {
  stopLiveScanner();

  const { BarcodeFormat, BrowserMultiFormatReader } = await loadBrowserModule();
  const reader = new BrowserMultiFormatReader();
  activeControls = await reader.decodeFromVideoDevice(
    deviceId || undefined,
    videoElement,
    (result, error) => {
      if (result) {
        onResult(createResultPayload(result, "camera", BarcodeFormat));
        return;
      }

      if (error && error.name !== "NotFoundException") {
        onError(error);
      }
    }
  );

  return activeControls;
}

export function stopLiveScanner() {
  try {
    activeControls?.stop();
  } catch (_error) {
    // Scanner teardown should stay best-effort.
  }

  activeControls = null;
  browserCodeReaderRef?.releaseAllStreams?.();
}

export async function scanImageFile(file) {
  const { BarcodeFormat, BrowserMultiFormatReader } = await loadBrowserModule();
  const reader = new BrowserMultiFormatReader();
  const objectUrl = URL.createObjectURL(file);

  try {
    const result = await reader.decodeFromImageUrl(objectUrl);
    return createResultPayload(result, "image-upload", BarcodeFormat);
  } finally {
    URL.revokeObjectURL(objectUrl);
  }
}
