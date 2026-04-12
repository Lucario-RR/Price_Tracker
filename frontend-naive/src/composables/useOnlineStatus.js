import { onBeforeUnmount, onMounted, ref } from "vue";

export function useOnlineStatus() {
  const isOnline = ref(navigator.onLine);

  function syncStatus() {
    isOnline.value = navigator.onLine;
  }

  onMounted(() => {
    window.addEventListener("online", syncStatus);
    window.addEventListener("offline", syncStatus);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("online", syncStatus);
    window.removeEventListener("offline", syncStatus);
  });

  return isOnline;
}
