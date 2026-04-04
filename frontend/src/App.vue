<script setup>
import { computed, ref } from "vue";

const baseUrl = ref("http://127.0.0.1:3000/api/v1");
const accountId = ref("");
const email = ref("alex@example.com");
const password = ref("StrongPassword!234");
const itemVariantId = ref("");
const shopId = ref("");
const purchaseId = ref("");
const responseText = ref("");
const loading = ref(false);

const headers = computed(() => {
  const value = { "Content-Type": "application/json" };
  if (accountId.value) {
    value["x-account-id"] = accountId.value;
  }
  return value;
});

async function request(path, options = {}) {
  loading.value = true;
  try {
    const res = await fetch(`${baseUrl.value}${path}`, {
      headers: headers.value,
      ...options
    });
    const text = await res.text();
    responseText.value = `HTTP ${res.status}\n\n${text}`;
    return text ? JSON.parse(text) : null;
  } catch (error) {
    responseText.value = String(error);
    return null;
  } finally {
    loading.value = false;
  }
}

async function loginDemo() {
  const data = await request("/auth/login", {
    method: "POST",
    body: JSON.stringify({
      email: email.value,
      password: password.value
    })
  });

  if (data?.data?.user?.id) {
    accountId.value = data.data.user.id;
  }
}

function loadCatalog() {
  request("/items");
}

function loadShops() {
  request("/shops");
}

function loadProfile() {
  request("/me");
}

function createPurchase() {
  if (!shopId.value) {
    responseText.value = "Set a shopId first.";
    return;
  }

  request("/purchases", {
    method: "POST",
    body: JSON.stringify({
      shopId: shopId.value,
      purchaseTime: new Date().toISOString(),
      notes: "Created from the Vue API tester"
    })
  }).then((data) => {
    if (data?.data?.id) {
      purchaseId.value = data.data.id;
    }
  });
}

function submitPrice() {
  if (!itemVariantId.value || !purchaseId.value) {
    responseText.value = "Set both itemVariantId and purchaseId first.";
    return;
  }

  request("/prices", {
    method: "POST",
    body: JSON.stringify({
      itemVariantId: itemVariantId.value,
      purchaseId: purchaseId.value,
      originalAmount: "1.80",
      originalCurrency: "GBP",
      discountAmount: "0.20",
      discountCurrency: "GBP",
      recordedAt: new Date().toISOString(),
      notes: "Created from the Vue API tester"
    })
  });
}

function compareVariant() {
  if (!itemVariantId.value) {
    responseText.value = "Set an itemVariantId first.";
    return;
  }
  request("/comparisons", {
    method: "POST",
    body: JSON.stringify({
      variantIds: [itemVariantId.value]
    })
  });
}
</script>

<template>
  <main class="app-shell">
    <section class="hero">
      <p class="eyebrow">PriceTracker</p>
      <h1>Vue API Test Bench</h1>
      <p class="lede">
        This lightweight client is meant for manual endpoint testing while the Axum backend is running locally.
      </p>
    </section>

    <section class="panel controls">
      <label>
        <span>API base URL</span>
        <input v-model="baseUrl" />
      </label>
      <label>
        <span>Account ID</span>
        <input v-model="accountId" placeholder="Filled after login or use x-account-id manually" />
      </label>
      <label>
        <span>Email</span>
        <input v-model="email" />
      </label>
      <label>
        <span>Password</span>
        <input v-model="password" type="password" />
      </label>
      <div class="button-row">
        <button @click="loginDemo" :disabled="loading">Login demo user</button>
        <button @click="loadProfile" :disabled="loading">Get /me</button>
        <button @click="loadCatalog" :disabled="loading">List items</button>
        <button @click="loadShops" :disabled="loading">List shops</button>
      </div>
    </section>

    <section class="panel workflow">
      <h2>Create linked records</h2>
      <label>
        <span>Shop ID</span>
        <input v-model="shopId" placeholder="Paste from /shops response" />
      </label>
      <label>
        <span>Item Variant ID</span>
        <input v-model="itemVariantId" placeholder="Paste from /items/{id}/variants response" />
      </label>
      <label>
        <span>Purchase ID</span>
        <input v-model="purchaseId" placeholder="Auto-filled after create purchase" />
      </label>
      <div class="button-row">
        <button @click="createPurchase" :disabled="loading">Create purchase</button>
        <button @click="submitPrice" :disabled="loading">Submit price</button>
        <button @click="compareVariant" :disabled="loading">Compare variant</button>
      </div>
    </section>

    <section class="panel output">
      <div class="output-head">
        <h2>Last response</h2>
        <span v-if="loading">Loading...</span>
      </div>
      <pre>{{ responseText || "Responses will appear here." }}</pre>
    </section>
  </main>
</template>

<style>
:root {
  color-scheme: light;
  font-family: "Segoe UI", "Helvetica Neue", sans-serif;
  background:
    radial-gradient(circle at top left, rgba(255, 209, 102, 0.35), transparent 30%),
    radial-gradient(circle at top right, rgba(0, 168, 150, 0.22), transparent 28%),
    linear-gradient(180deg, #f7f2e8 0%, #f3f6fb 100%);
  color: #1f2933;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-height: 100vh;
}

.app-shell {
  max-width: 1080px;
  margin: 0 auto;
  padding: 32px 20px 56px;
}

.hero {
  margin-bottom: 24px;
}

.eyebrow {
  margin: 0 0 8px;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  font-size: 0.78rem;
  color: #0d7a67;
}

h1, h2 {
  margin: 0 0 12px;
}

.lede {
  max-width: 60ch;
  line-height: 1.5;
}

.panel {
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(31, 41, 51, 0.08);
  border-radius: 20px;
  padding: 20px;
  box-shadow: 0 18px 48px rgba(31, 41, 51, 0.08);
  backdrop-filter: blur(10px);
  margin-bottom: 18px;
}

.controls,
.workflow {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

label {
  display: grid;
  gap: 8px;
  font-size: 0.95rem;
}

input {
  width: 100%;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid #ccd6e0;
  background: #fff;
  font-size: 0.96rem;
}

.button-row {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

button {
  border: none;
  border-radius: 999px;
  padding: 12px 18px;
  background: #0d7a67;
  color: white;
  cursor: pointer;
  font-weight: 600;
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.output-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

pre {
  margin: 0;
  padding: 16px;
  background: #15202b;
  color: #e7edf5;
  border-radius: 14px;
  overflow: auto;
  min-height: 280px;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 640px) {
  .app-shell {
    padding: 20px 14px 40px;
  }

  .panel {
    border-radius: 16px;
    padding: 16px;
  }
}
</style>
