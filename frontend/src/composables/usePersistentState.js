import { ref, watch } from "vue";

function cloneInitialValue(value) {
  if (typeof structuredClone === "function") {
    return structuredClone(value);
  }

  return JSON.parse(JSON.stringify(value));
}

export function usePersistentState(key, initialValue) {
  const state = ref(cloneInitialValue(initialValue));

  try {
    const stored = localStorage.getItem(key);
    if (stored) {
      state.value = JSON.parse(stored);
    }
  } catch (error) {
    console.warn(`Unable to restore persisted state for ${key}`, error);
  }

  watch(
    state,
    (value) => {
      try {
        localStorage.setItem(key, JSON.stringify(value));
      } catch (error) {
        console.warn(`Unable to persist state for ${key}`, error);
      }
    },
    { deep: true }
  );

  return state;
}
