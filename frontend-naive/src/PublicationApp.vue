<script setup>
import { reactive } from "vue";

import AdminDatabasePanel from "./components/AdminDatabasePanel.vue";
import AdminExplorerPanel from "./components/AdminExplorerPanel.vue";
import AdminModerationPanel from "./components/AdminModerationPanel.vue";
import QueuePanel from "./components/QueuePanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import ScannerPanel from "./components/ScannerPanel.vue";
import { useWorkbench } from "./composables/useWorkbench";

const model = reactive(useWorkbench());
</script>

<template>
  <main class="page-shell">
    <section class="panel hero-panel publication-hero">
      <div class="hero-copy">
        <p class="eyebrow">PriceTracker</p>
        <h1>Scan codes, save price evidence, and keep working when the backend drops.</h1>
        <p>
          The public page focuses on capture and submission. Admin-only tooling now lives in a
          separate dashboard so publication stays clean while testing stays available.
        </p>
      </div>

      <div class="hero-status hero-actions">
        <span class="status-pill tone-info">
          {{ model.workbench.ui.activeView === "admin" ? "Admin view" : "Public view" }}
        </span>
        <button :disabled="model.isBusy" @click="model.runHealthCheck">Check system</button>
        <button class="ghost-button" :disabled="model.isBusy" @click="model.openPublicView">
          Public view
        </button>
        <button class="ghost-button" :disabled="model.isBusy" @click="model.openAdminView">
          Admin dashboard
        </button>
      </div>
    </section>

    <section class="status-light-grid">
      <article
        v-for="light in model.statusLights"
        :key="light.key"
        :class="['panel', 'status-light-card', `summary-${light.tone}`]"
      >
        <div class="status-light-head">
          <span :class="['status-dot', `tone-${light.tone}`]" />
          <strong>{{ light.label }}</strong>
        </div>
        <strong class="summary-value">{{ light.state }}</strong>
        <p class="helper-text">{{ light.detail }}</p>
      </article>
    </section>

    <section class="summary-grid">
      <article
        v-for="card in model.summaryCards"
        :key="card.label"
        :class="['panel', 'summary-card', `summary-${card.tone}`]"
      >
        <span class="summary-label">{{ card.label }}</span>
        <strong class="summary-value">{{ card.value }}</strong>
      </article>
    </section>

    <template v-if="model.workbench.ui.activeView === 'public'">
    <section class="workspace-grid">
      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Account</p>
            <h2>Session and access</h2>
          </div>
        </div>

        <div class="field-grid two-up">
          <label class="field">
            <span>Email</span>
            <input v-model="model.workbench.session.email" />
          </label>
          <label class="field">
            <span>Password</span>
            <input v-model="model.workbench.session.password" type="password" />
          </label>
          <label class="field">
            <span>Display name</span>
            <input v-model="model.workbench.session.displayName" />
          </label>
          <label class="field">
            <span>Primary phone</span>
            <input v-model="model.workbench.session.primaryPhone" />
          </label>
          <label class="field full-span">
            <span>Account ID</span>
            <input v-model="model.workbench.session.accountId" />
          </label>
        </div>

        <div class="action-row wrap">
          <button :disabled="model.isBusy" @click="model.loginSession">Login current form</button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.loginDemo">
            Login demo user
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.loginAdmin">
            Login admin
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.loadProfile">
            Load /me
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.logoutSession">
            Clear session
          </button>
        </div>

        <p class="helper-text">
          Admin seed account: <code>admin@pricetracker.local</code> /
          <code>StrongPassword!234</code>
        </p>

        <div class="pill-row">
          <span
            v-for="role in model.workbench.session.roles"
            :key="role"
            class="status-pill tone-info"
          >
            {{ role }}
          </span>
          <span
            v-if="!model.workbench.session.roles.length"
            class="status-pill tone-warn"
          >
            No roles loaded yet
          </span>
        </div>
      </article>

      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Capture context</p>
            <h2>Lookup state and routing</h2>
          </div>
        </div>

        <div class="field-grid two-up">
          <label class="field">
            <span>API base URL</span>
            <input v-model="model.workbench.workspace.apiBaseUrl" />
          </label>
          <label class="field">
            <span>Shop ID</span>
            <input v-model="model.workbench.lookup.shopId" />
          </label>
          <label class="field">
            <span>Item ID</span>
            <input v-model="model.workbench.lookup.itemId" />
          </label>
          <label class="field">
            <span>Variant ID</span>
            <input v-model="model.workbench.lookup.variantId" />
          </label>
          <label class="field">
            <span>Purchase ID</span>
            <input v-model="model.workbench.lookup.purchaseId" />
          </label>
          <label class="field">
            <span>Price ID</span>
            <input v-model="model.workbench.lookup.priceId" />
          </label>
        </div>

        <div class="action-row wrap">
          <button :disabled="model.isBusy" @click="model.listShops">List shops</button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.listItems">
            List items
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.lookupCapturedCode">
            Lookup scanned code
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.loadVariantDetail">
            Load variant
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.compareVariant">
            Compare variant
          </button>
        </div>
      </article>
    </section>

    <section class="workspace-grid">
      <ScannerPanel
        :model="model.workbench.capture"
        :shop-id="model.workbench.lookup.shopId"
        :disabled="model.isBusy"
        @capture="model.handleCapturedCode"
        @lookup="model.lookupCapturedCode"
      />

      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Submission</p>
            <h2>Price and receipt capture</h2>
          </div>
        </div>

        <div class="field-grid two-up">
          <label class="field">
            <span>Purchase time</span>
            <input v-model="model.workbench.submission.purchaseTime" type="datetime-local" />
          </label>
          <label class="field">
            <span>Recorded at</span>
            <input v-model="model.workbench.submission.recordedAt" type="datetime-local" />
          </label>
          <label class="field">
            <span>Original amount</span>
            <input v-model="model.workbench.submission.originalAmount" />
          </label>
          <label class="field">
            <span>Original currency</span>
            <input v-model="model.workbench.submission.originalCurrency" />
          </label>
          <label class="field">
            <span>Discount amount</span>
            <input v-model="model.workbench.submission.discountAmount" />
          </label>
          <label class="field">
            <span>Discount type ID</span>
            <input v-model="model.workbench.submission.discountTypeId" />
          </label>
          <label class="field full-span">
            <span>Attachment file IDs</span>
            <input v-model="model.workbench.submission.attachmentFileIds" />
          </label>
          <label class="field full-span">
            <span>Price notes</span>
            <textarea v-model="model.workbench.submission.priceNotes" rows="3" />
          </label>
        </div>

        <div class="action-row wrap">
          <button :disabled="model.isBusy" @click="model.createPurchase">Create purchase</button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.submitPrice">
            Submit price
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.saveFullCaptureForRetry">
            Save full capture for retry
          </button>
        </div>
      </article>
    </section>

    <section class="workspace-grid">
      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Files</p>
            <h2>Upload intent helper</h2>
          </div>
        </div>

        <div class="field-grid two-up">
          <label class="field">
            <span>Filename</span>
            <input v-model="model.workbench.submission.fileUpload.filename" />
          </label>
          <label class="field">
            <span>Content type</span>
            <input v-model="model.workbench.submission.fileUpload.contentType" />
          </label>
          <label class="field">
            <span>Size (bytes)</span>
            <input v-model="model.workbench.submission.fileUpload.size" />
          </label>
          <label class="field">
            <span>Purpose</span>
            <input v-model="model.workbench.submission.fileUpload.purpose" />
          </label>
        </div>

        <div class="action-row wrap">
          <button :disabled="model.isBusy" @click="model.createUploadIntent">
            Create upload intent
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.completeUploadIntent">
            Complete upload
          </button>
        </div>
      </article>

      <QueuePanel
        :queue="model.queue"
        :online="model.online"
        :flushing="model.isFlushingQueue"
        @flush="model.flushQueue"
        @retry="model.retryQueueEntry"
        @remove="model.removeQueueEntry"
        @clear="model.clearQueue"
      />
    </section>

    <section class="workspace-grid">
      <ResponsePanel :history="model.history" />

      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Tips</p>
            <h2>What to expect</h2>
          </div>
        </div>

        <p class="helper-text">
          Public buttons now show their latest request and response here so you can see whether
          the backend replied, the request queued, or validation stopped it locally.
        </p>

        <div class="pill-row">
          <span class="status-pill tone-info">Use "List shops" to load a shop ID first</span>
          <span class="status-pill tone-info">Use scanner, image upload, or manual code entry</span>
          <span class="status-pill tone-info">The queue will retry when browser and API are back</span>
        </div>
      </article>
    </section>

    <section class="panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Activity</p>
          <h2>Recent client and API events</h2>
        </div>
      </div>

      <div class="history-list">
        <div v-for="entry in model.history.slice(0, 6)" :key="entry.id" class="history-item">
          <strong>{{ entry.label }}</strong>
          <span>{{ new Date(entry.startedAt).toLocaleString() }}</span>
        </div>
      </div>
    </section>
    </template>

    <template v-else>
    <section class="admin-zone">
      <div class="section-head">
        <div>
          <p class="section-kicker">Admin</p>
          <h2>Dashboard modules</h2>
        </div>
        <span :class="['status-pill', model.isAdmin ? 'tone-good' : 'tone-warn']">
          {{ model.isAdmin ? "Admin access loaded" : "Admin sign-in required" }}
        </span>
      </div>

      <template v-if="model.isAdmin">
        <section class="summary-grid">
          <article
            v-for="card in model.adminSummaryCards"
            :key="card.label"
            :class="['panel', 'summary-card', `summary-${card.tone}`]"
          >
            <span class="summary-label">{{ card.label }}</span>
            <strong class="summary-value">{{ card.value }}</strong>
          </article>
        </section>

        <section class="workspace-grid">
          <AdminDatabasePanel
            :tables="model.adminTables"
            :table="model.adminTable"
            :admin="model.admin"
            :disabled="model.isBusy"
            @select-table="model.selectAdminTable"
            @select-row="model.selectAdminRow"
            @create="model.startCreateAdminRecord"
            @refresh="model.refreshAdminTable"
            @save="model.saveAdminRecord"
          />

          <AdminModerationPanel
            :prices="model.admin.moderationPrices"
            :loading="model.admin.loadingModeration"
            :disabled="model.isBusy"
            @refresh="model.loadModerationPrices"
            @approve="model.approveModerationPrice"
            @reject="model.rejectModerationPrice"
          />
        </section>

        <section class="workspace-grid">
          <ResponsePanel :history="model.history" />

          <AdminExplorerPanel
            :model="model.workbench"
            :endpoint-groups="model.endpointGroups"
            :filtered-endpoints="model.filteredEndpoints"
            :selected-endpoint="model.selectedEndpoint"
            :explorer-error="model.explorerError"
            :busy="model.isBusy"
            :enabled="model.debugToolsEnabled"
            @send="model.sendExplorerRequest"
            @queue="model.queueExplorerRequest"
            @reset="model.resetExplorerDraft"
            @add-header="model.addExplorerHeader"
            @remove-header="model.removeExplorerHeader"
          />
        </section>
      </template>

      <article v-else class="panel admin-gate">
        <h3>Admin dashboard is locked</h3>
        <p class="helper-text">
          Sign in with the seeded admin account to manage database content, review submissions,
          and use the smaller debug modules.
        </p>
        <div class="action-row wrap">
          <button @click="model.fillAdminSession">Use admin credentials</button>
          <button class="ghost-button" @click="model.loginAdmin">Login as admin</button>
        </div>
      </article>
    </section>
    </template>
  </main>
</template>
