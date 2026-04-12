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
  return `${price.finalAmount || price.originalAmount || "—"} ${price.originalCurrency || "GBP"}`;
}

function openDetail(price) {
  ui.detailPrice = price;
  ui.detailOpen = true;
}

function closeDetail() {
  ui.detailOpen = false;
}
</script>

<template>
  <n-card title="待审批价格" :segmented="{ content: true }">
    <template #header-extra>
      <n-button size="small" quaternary :disabled="disabled || loading" @click="refreshPrices">刷新</n-button>
    </template>

    <n-space align="center" style="margin-bottom: 12px" wrap>
      <n-input
        v-model:value="ui.search"
        :disabled="disabled"
        clearable
        placeholder="搜索 ID、变体、采购单或备注"
        style="min-width: 240px"
      />
      <n-tag type="info" size="small">{{ filteredPrices.length }} 条</n-tag>
    </n-space>

    <n-spin :show="loading">
      <n-empty v-if="!filteredPrices.length" :description="loading ? '加载中…' : '没有待审批价格'" />

      <n-scrollbar v-else style="max-height: 520px">
        <n-space vertical size="medium" style="padding-right: 8px">
          <n-card v-for="price in filteredPrices" :key="price.id" size="small" embedded hoverable>
            <n-space justify="space-between" align="center">
              <n-space vertical size="small" style="cursor: pointer" @click="openDetail(price)">
                <n-text strong>{{ price.id }}</n-text>
                <n-text depth="3">变体 {{ price.itemVariantId }} · 采购 {{ price.purchaseId }}</n-text>
                <n-text>{{ moneyLabel(price) }}</n-text>
                <n-tag type="warning" size="small">{{ price.submissionStatus }}</n-tag>
              </n-space>
              <n-space vertical>
                <n-button size="small" :disabled="disabled || loading" @click="approvePrice(price.id)">批准</n-button>
                <n-button size="small" type="error" tertiary :disabled="disabled || loading" @click="rejectPrice(price.id)">
                  拒绝
                </n-button>
              </n-space>
            </n-space>
          </n-card>
        </n-space>
      </n-scrollbar>
    </n-spin>

    <n-modal
      v-model:show="ui.detailOpen"
      preset="card"
      style="width: min(640px, 92vw)"
      :title="ui.detailPrice ? `价格 ${ui.detailPrice.id}` : ''"
    >
      <template v-if="ui.detailPrice">
        <n-descriptions bordered size="small" :column="1">
          <n-descriptions-item label="变体">{{ ui.detailPrice.itemVariantId }}</n-descriptions-item>
          <n-descriptions-item label="采购">{{ ui.detailPrice.purchaseId }}</n-descriptions-item>
          <n-descriptions-item label="原价">{{ ui.detailPrice.originalAmount }}</n-descriptions-item>
          <n-descriptions-item label="折算价">{{ ui.detailPrice.finalAmount }}</n-descriptions-item>
          <n-descriptions-item label="货币">{{ ui.detailPrice.originalCurrency }}</n-descriptions-item>
          <n-descriptions-item label="状态">{{ ui.detailPrice.submissionStatus }}</n-descriptions-item>
          <n-descriptions-item label="记录时间">
            {{ ui.detailPrice.recordedAt ? new Date(ui.detailPrice.recordedAt).toLocaleString() : "—" }}
          </n-descriptions-item>
          <n-descriptions-item label="备注">{{ ui.detailPrice.notes || "无" }}</n-descriptions-item>
        </n-descriptions>
        <n-space style="margin-top: 16px">
          <n-button type="primary" :disabled="disabled || loading" @click="approvePrice(ui.detailPrice.id)">
            批准
          </n-button>
          <n-button type="error" tertiary :disabled="disabled || loading" @click="rejectPrice(ui.detailPrice.id)">
            拒绝
          </n-button>
          <n-button quaternary @click="closeDetail">关闭</n-button>
        </n-space>
      </template>
    </n-modal>
  </n-card>
</template>
