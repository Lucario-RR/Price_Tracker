<script setup>
import { computed, reactive } from "vue";

const props = defineProps({
  prices: {
    type: Array,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  refreshPrices: {
    type: Function,
    required: true
  },
  approvePrice: {
    type: Function,
    required: true
  },
  rejectPrice: {
    type: Function,
    required: true
  }
});

const ui = reactive({
  search: "",
  detailPrice: null,
  detailOpen: false
});

const filteredPrices = computed(() => {
  const search = ui.search.trim().toLowerCase();

  return props.prices.filter((price) => {
    if (!search) {
      return true;
    }

    return [
      price.id,
      price.itemVariantId,
      price.purchaseId,
      price.submissionStatus,
      price.originalCurrency,
      price.notes
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(search));
  });
});

function moneyLabel(price) {
  return `${price.finalAmount || price.originalAmount || "Unknown"} ${price.originalCurrency || "GBP"}`;
}

function openDetail(price) {
  ui.detailPrice = price;
  ui.detailOpen = true;
}
</script>

<template>
  <article class="panel admin-module">
    <div class="section-head">
      <div>
        <p class="section-kicker">Approvals</p>
        <h2>Items awaiting approval</h2>
      </div>
      <button class="ghost-button" :disabled="disabled || loading" @click="refreshPrices">
        Refresh
      </button>
    </div>

    <div class="admin-toolbar">
      <label class="field">
        <span>Search pending items</span>
        <input v-model="ui.search" :disabled="disabled" placeholder="Search price ID, variant, purchase, or notes" />
      </label>
      <span class="status-pill tone-info">{{ filteredPrices.length }} waiting</span>
    </div>

    <div class="table-wrap">
      <table class="admin-table">
        <thead>
          <tr>
            <th>Price ID</th>
            <th>Variant</th>
            <th>Purchase</th>
            <th>Amount</th>
            <th>Status</th>
            <th class="actions-col">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="price in filteredPrices" :key="price.id" @click="openDetail(price)">
            <td>{{ price.id }}</td>
            <td>{{ price.itemVariantId }}</td>
            <td>{{ price.purchaseId }}</td>
            <td>{{ moneyLabel(price) }}</td>
            <td><span class="status-pill tone-warn">{{ price.submissionStatus }}</span></td>
            <td class="actions-col" @click.stop>
              <div class="action-row wrap">
                <button class="ghost-button compact-button" :disabled="disabled || loading" @click="approvePrice(price.id)">
                  Approve
                </button>
                <button class="danger-button compact-button" :disabled="disabled || loading" @click="rejectPrice(price.id)">
                  Reject
                </button>
              </div>
            </td>
          </tr>
          <tr v-if="!filteredPrices.length">
            <td colspan="6" class="empty-table">
              {{ loading ? "Loading moderation queue..." : "No prices are waiting for review." }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="ui.detailOpen && ui.detailPrice" class="admin-modal-backdrop" @click.self="ui.detailOpen = false">
      <article class="admin-modal">
        <div class="section-head">
          <div>
            <p class="section-kicker">Approval detail</p>
            <h2>{{ ui.detailPrice.id }}</h2>
          </div>
          <button class="ghost-button compact-button" @click="ui.detailOpen = false">Close</button>
        </div>

        <div class="simple-list">
          <div class="list-row"><strong>Variant</strong><span>{{ ui.detailPrice.itemVariantId }}</span></div>
          <div class="list-row"><strong>Purchase</strong><span>{{ ui.detailPrice.purchaseId }}</span></div>
          <div class="list-row"><strong>Original amount</strong><span>{{ ui.detailPrice.originalAmount }}</span></div>
          <div class="list-row"><strong>Final amount</strong><span>{{ ui.detailPrice.finalAmount }}</span></div>
          <div class="list-row"><strong>Currency</strong><span>{{ ui.detailPrice.originalCurrency }}</span></div>
          <div class="list-row"><strong>Status</strong><span>{{ ui.detailPrice.submissionStatus }}</span></div>
          <div class="list-row"><strong>Recorded</strong><span>{{ ui.detailPrice.recordedAt ? new Date(ui.detailPrice.recordedAt).toLocaleString() : "Not available" }}</span></div>
          <div class="list-row"><strong>Notes</strong><span>{{ ui.detailPrice.notes || "No notes" }}</span></div>
        </div>

        <div class="action-row wrap">
          <button :disabled="disabled || loading" @click="approvePrice(ui.detailPrice.id)">Approve</button>
          <button class="danger-button" :disabled="disabled || loading" @click="rejectPrice(ui.detailPrice.id)">
            Reject
          </button>
        </div>
      </article>
    </div>
  </article>
</template>
