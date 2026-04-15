<script setup>
import { reactive } from "vue";

import QueuePanel from "./components/QueuePanel.vue";
import ResponsePanel from "./components/ResponsePanel.vue";
import ScannerPanel from "./components/ScannerPanel.vue";
import { useWorkbench } from "./composables/useWorkbench";

const model = reactive(useWorkbench());
</script>

<template>
  <main class="page-shell">
    <section class="panel hero-panel">
      <div class="hero-copy">
        <p class="eyebrow">PriceTracker Frontend</p>
        <h1>One Nuxt page for scanning, capturing, and debugging the whole stack.</h1>
        <p>
          This frontend keeps your working state locally, retries safe queued jobs
          when the network returns, and gives you a production-safe way to hide
          the raw debug explorer at publish time.
        </p>
      </div>

      <div class="hero-status">
        <span :class="['status-pill', model.online ? 'tone-good' : 'tone-warn']">
          {{ model.online ? "Online" : "Offline" }}
        </span>
        <span class="status-pill tone-info">{{ model.queue.length }} queued</span>
        <span class="status-pill tone-accent">
          {{ model.debugToolsEnabled ? "Debug explorer on" : "Debug explorer hidden in production" }}
        </span>
      </div>
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

    <section class="workspace-grid">
      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Connection</p>
            <h2>Workspace and routing</h2>
          </div>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.runHealthCheck">
            Check health
          </button>
        </div>

        <div class="field-grid">
          <label class="field">
            <span>API base URL</span>
            <input v-model="model.workbench.workspace.apiBaseUrl" />
          </label>
        </div>

        <p class="helper-text">
          Local development uses a Nuxt proxy by default, so keeping the base URL
          at <code>/api/v1</code> avoids CORS friction while you debug.
        </p>

        <p class="helper-text">
          Use <code>npm run build</code> for a publish-safe build with the explorer
          hidden, or <code>npm run build:debug</code> if you intentionally want to
          ship the raw debug tooling.
        </p>
      </article>

      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Session</p>
            <h2>Demo account and current context</h2>
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
            <input
              v-model="model.workbench.session.primaryPhone"
              placeholder="+447700900123"
            />
          </label>
          <label class="field">
            <span>Account ID</span>
            <input
              v-model="model.workbench.session.accountId"
              placeholder="Auto-filled from login or profile calls"
            />
          </label>
          <label class="field">
            <span>Access token</span>
            <input
              v-model="model.workbench.session.accessToken"
              placeholder="Captured from login or refresh"
            />
          </label>
        </div>

        <div class="action-row">
          <button :disabled="model.isBusy" @click="model.loginDemo">Login demo user</button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.refreshSession">
            Refresh session
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.loadProfile">
            Load /me
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
            <p class="section-kicker">Lookup state</p>
            <h2>Saved IDs and catalog helpers</h2>
          </div>
        </div>

        <div class="field-grid two-up">
          <label class="field">
            <span>Shop ID</span>
            <input v-model="model.workbench.lookup.shopId" placeholder="Pick from /shops" />
          </label>
          <label class="field">
            <span>Item ID</span>
            <input v-model="model.workbench.lookup.itemId" placeholder="Pick from /items" />
          </label>
          <label class="field">
            <span>Variant ID</span>
            <input
              v-model="model.workbench.lookup.variantId"
              placeholder="Filled from scanner lookup or variant responses"
            />
          </label>
          <label class="field">
            <span>Purchase ID</span>
            <input
              v-model="model.workbench.lookup.purchaseId"
              placeholder="Filled after creating a purchase"
            />
          </label>
          <label class="field">
            <span>Price ID</span>
            <input v-model="model.workbench.lookup.priceId" />
          </label>
          <label class="field">
            <span>File ID</span>
            <input v-model="model.workbench.lookup.fileId" />
          </label>
          <label class="field">
            <span>Alert ID</span>
            <input v-model="model.workbench.lookup.alertId" />
          </label>
          <label class="field">
            <span>Watch ID</span>
            <input v-model="model.workbench.lookup.watchId" />
          </label>
          <label class="field">
            <span>Email ID</span>
            <input v-model="model.workbench.lookup.emailId" />
          </label>
          <label class="field">
            <span>Phone ID</span>
            <input v-model="model.workbench.lookup.phoneId" />
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
            Get variant
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.compareVariant">
            Compare variant
          </button>
        </div>
      </article>
    </section>

    <section class="workspace-grid">
      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Submission</p>
            <h2>Purchase and price capture</h2>
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
          <label class="field full-span">
            <span>Attachment file IDs</span>
            <input
              v-model="model.workbench.submission.attachmentFileIds"
              placeholder="Comma-separated file IDs from upload intents"
            />
          </label>
          <label class="field full-span">
            <span>Purchase notes</span>
            <textarea v-model="model.workbench.submission.purchaseNotes" rows="3" />
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
            <input
              v-model="model.workbench.submission.discountAmount"
              placeholder="Optional"
            />
          </label>
          <label class="field">
            <span>Discount currency</span>
            <input v-model="model.workbench.submission.discountCurrency" />
          </label>
          <label class="field full-span">
            <span>Discount type ID</span>
            <input
              v-model="model.workbench.submission.discountTypeId"
              placeholder="Optional /discount-types value"
            />
          </label>
          <label class="field full-span">
            <span>Price notes</span>
            <textarea v-model="model.workbench.submission.priceNotes" rows="3" />
          </label>
        </div>

        <div class="action-row wrap">
          <button :disabled="model.isBusy" @click="model.createPurchase">
            Create purchase now
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.submitPrice">
            Submit price now
          </button>
          <button class="ghost-button" :disabled="model.isBusy" @click="model.saveFullCaptureForRetry">
            Save full capture for retry
          </button>
        </div>
      </article>

      <article class="panel">
        <div class="section-head">
          <div>
            <p class="section-kicker">Files</p>
            <h2>Receipt and evidence stubs</h2>
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
          <label class="field full-span">
            <span>Checksum SHA-256</span>
            <input
              v-model="model.workbench.submission.fileUpload.checksumSha256"
              placeholder="Optional"
            />
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
    </section>

    <section class="workspace-grid">
      <QueuePanel
        :queue="model.queue"
        :online="model.online"
        :flushing="model.isFlushingQueue"
        @flush="model.flushQueue"
        @retry="model.retryQueueEntry"
        @remove="model.removeQueueEntry"
        @clear="model.clearQueue"
      />

      <ResponsePanel :history="model.history" />
    </section>

    <section v-if="model.debugToolsEnabled" class="panel debug-panel">
      <div class="section-head">
        <div>
          <p class="section-kicker">Debug explorer</p>
          <h2>Full API surface</h2>
        </div>
      </div>

      <div class="explorer-layout">
        <aside class="endpoint-browser">
          <label class="field">
            <span>Group</span>
            <select v-model="model.workbench.explorer.selectedGroup">
              <option v-for="group in model.endpointGroups" :key="group" :value="group">
                {{ group }}
              </option>
            </select>
          </label>

          <label class="field">
            <span>Search</span>
            <input
              v-model="model.workbench.explorer.search"
              placeholder="Search label, path, or description"
            />
          </label>

          <div class="endpoint-list">
            <button
              v-for="endpoint in model.filteredEndpoints"
              :key="endpoint.id"
              :class="[
                'endpoint-card',
                endpoint.id === model.selectedEndpoint.id ? 'is-selected' : ''
              ]"
              @click="model.workbench.explorer.selectedEndpointId = endpoint.id"
            >
              <span class="endpoint-method">{{ endpoint.method }}</span>
              <strong>{{ endpoint.label }}</strong>
              <code>{{ endpoint.path }}</code>
            </button>
          </div>
        </aside>

        <article class="endpoint-editor">
          <div class="endpoint-editor-head">
            <div>
              <h3>{{ model.selectedEndpoint.label }}</h3>
              <p>{{ model.selectedEndpoint.description }}</p>
            </div>
            <span class="status-pill tone-info">{{ model.selectedEndpoint.group }}</span>
          </div>

          <div class="field-grid two-up">
            <label v-if="model.selectedEndpoint.id === 'custom'" class="field">
              <span>Method</span>
              <select v-model="model.workbench.explorer.customMethod">
                <option
                  v-for="method in model.requestMethodOptions()"
                  :key="method"
                  :value="method"
                >
                  {{ method }}
                </option>
              </select>
            </label>

            <label v-if="model.selectedEndpoint.id === 'custom'" class="field full-span">
              <span>Path</span>
              <input v-model="model.workbench.explorer.customPath" placeholder="/health" />
            </label>

            <label
              v-for="pathParam in model.selectedEndpoint.pathParams || []"
              :key="`path-${pathParam}`"
              class="field"
            >
              <span>{{ pathParam }}</span>
              <input v-model="model.workbench.explorer.pathParams[pathParam]" />
            </label>

            <label
              v-for="queryParam in model.selectedEndpoint.queryParams || []"
              :key="`query-${queryParam}`"
              class="field"
            >
              <span>{{ queryParam }}</span>
              <input v-model="model.workbench.explorer.queryParams[queryParam]" />
            </label>
          </div>

          <div class="inline-head">
            <h4>Extra headers</h4>
            <button class="ghost-button" @click="model.addExplorerHeader">Add header</button>
          </div>

          <div v-if="model.workbench.explorer.extraHeaders.length" class="stack">
            <div
              v-for="(header, index) in model.workbench.explorer.extraHeaders"
              :key="`header-${index}`"
              class="header-row"
            >
              <input v-model="header.key" placeholder="Header name" />
              <input v-model="header.value" placeholder="Header value" />
              <button class="danger-button" @click="model.removeExplorerHeader(index)">
                Remove
              </button>
            </div>
          </div>

          <label class="field">
            <span>JSON body</span>
            <textarea
              v-model="model.workbench.explorer.bodyText"
              rows="12"
              placeholder="Body goes here for POST or PATCH routes"
            />
          </label>

          <label class="checkbox-field">
            <input v-model="model.workbench.explorer.queueOnFailure" type="checkbox" />
            <span>Queue writes when the browser is offline</span>
          </label>

          <p v-if="model.explorerError" class="error-text">{{ model.explorerError }}</p>

          <div class="action-row wrap">
            <button :disabled="model.isBusy" @click="model.sendExplorerRequest">
              Send request
            </button>
            <button class="ghost-button" :disabled="model.isBusy" @click="model.queueExplorerRequest">
              Save to retry queue
            </button>
            <button class="ghost-button" :disabled="model.isBusy" @click="model.resetExplorerDraft">
              Reset template
            </button>
          </div>
        </article>
      </div>
    </section>
  </main>
</template>
