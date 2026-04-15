<script setup>
import { computed, reactive, watch } from "vue";

const props = defineProps({
  table: {
    type: Object,
    default: null
  },
  rows: {
    type: Array,
    required: true
  },
  lookups: {
    type: Object,
    default: () => ({})
  },
  error: {
    type: String,
    default: ""
  },
  loading: {
    type: Boolean,
    default: false
  },
  saving: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  refreshTable: {
    type: Function,
    required: true
  },
  saveRecord: {
    type: Function,
    required: true
  },
  deleteRecord: {
    type: Function,
    required: true
  },
  approveRecord: {
    type: Function,
    required: true
  },
  bulkDeleteRecords: {
    type: Function,
    required: true
  },
  bulkApproveRecords: {
    type: Function,
    required: true
  },
  confirmAction: {
    type: Function,
    required: true
  }
});

const ui = reactive({
  search: "",
  page: 1,
  pageSize: 10,
  statusFilter: "all",
  approvalView: "all",
  detailOpen: false,
  editorOpen: false,
  editorMode: "create",
  detailRow: null,
  editingRecordId: ""
});

const editorValues = reactive({});
const lookupSearch = reactive({});
const bulkSelection = reactive({
  selectedIds: []
});

const awaitingApprovalStatuses = new Set([
  "awaiting approval",
  "awaiting-approval",
  "awaiting_approval",
  "flagged",
  "pending",
  "review",
  "submitted"
]);

function normalizeStatus(value) {
  return String(value || "")
    .trim()
    .toLowerCase();
}

function isAwaitingApprovalStatus(value) {
  const status = normalizeStatus(value);

  if (!status || status === "approved") {
    return false;
  }

  return (
    awaitingApprovalStatuses.has(status) ||
    status.includes("awaiting") ||
    status.includes("review")
  );
}

const availableStatuses = computed(() => {
  const values = new Set(
    props.rows.map((row) => row.status).filter((value) => typeof value === "string" && value)
  );
  return ["all", ...values];
});
const canApproveTable = computed(() => Boolean(props.table?.supportsApproval));
const canDeleteTable = computed(() => canEditTable.value);
const pendingApprovalCount = computed(() =>
  props.rows.filter((row) => canApproveTable.value && isAwaitingApprovalStatus(row.status)).length
);
const selectedRows = computed(() =>
  props.rows.filter((row) => bulkSelection.selectedIds.includes(row.id))
);
const selectedPendingApprovalIds = computed(() =>
  selectedRows.value
    .filter((row) => canApproveTable.value && isAwaitingApprovalStatus(row.status))
    .map((row) => row.id)
);

const filteredRows = computed(() => {
  const search = ui.search.trim().toLowerCase();

  return props.rows.filter((row) => {
    if (ui.approvalView === "pending" && !isAwaitingApprovalStatus(row.status)) {
      return false;
    }

    if (ui.statusFilter !== "all" && String(row.status || "") !== ui.statusFilter) {
      return false;
    }

    if (!search) {
      return true;
    }

    return Object.values(row)
      .filter((value) => value !== null && value !== undefined)
      .some((value) => String(value).toLowerCase().includes(search));
  });
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil(filteredRows.value.length / ui.pageSize))
);
const paginatedRows = computed(() => {
  const start = (ui.page - 1) * ui.pageSize;
  return filteredRows.value.slice(start, start + ui.pageSize);
});
const allVisibleSelected = computed(
  () =>
    paginatedRows.value.length > 0 &&
    paginatedRows.value.every((row) => bulkSelection.selectedIds.includes(row.id))
);
const visibleColumns = computed(() => {
  const columns = props.table?.columns || [];
  return columns.filter((column) => column.key !== "id").slice(0, 6);
});
const editableColumns = computed(() =>
  (props.table?.columns || []).filter((column) => column.mutable)
);
const canEditTable = computed(() => editableColumns.value.length > 0);

function canApproveRow(row) {
  return canApproveTable.value && isAwaitingApprovalStatus(row?.status);
}

function relocatePageForPageSize(nextPageSize, previousPageSize) {
  const nextSize = Math.max(1, Number(nextPageSize) || 1);
  const previousSize = Math.max(1, Number(previousPageSize) || nextSize);
  const firstVisibleIndex = Math.max(0, (ui.page - 1) * previousSize);
  const nextPage = Math.floor(firstVisibleIndex / nextSize) + 1;
  ui.page = Math.min(totalPages.value, Math.max(1, nextPage));
}

watch(filteredRows, () => {
  if (ui.page > totalPages.value) {
    ui.page = totalPages.value;
  }
});

watch(
  () => ui.pageSize,
  (nextPageSize, previousPageSize) => {
    if (nextPageSize === previousPageSize) {
      return;
    }

    relocatePageForPageSize(nextPageSize, previousPageSize);
  }
);

watch(
  () => props.table,
  () => {
    ui.search = "";
    ui.page = 1;
    ui.statusFilter = "all";
    ui.approvalView = "all";
    ui.detailOpen = false;
    ui.editorOpen = false;
    bulkSelection.selectedIds = [];
  }
);

watch(
  () => props.rows,
  (rows) => {
    const allowed = new Set(rows.map((row) => row.id));
    bulkSelection.selectedIds = bulkSelection.selectedIds.filter((id) => allowed.has(id));

    if (ui.detailRow?.id) {
      const matching = rows.find((row) => row.id === ui.detailRow.id);
      ui.detailRow = matching || null;

      if (!matching) {
        ui.detailOpen = false;
      }
    }
  },
  { deep: true }
);

watch(
  () => editorValues.isBaseUnit,
  (isBaseUnit) => {
    if (props.table?.id === "units" && isBaseUnit) {
      editorValues.unitFamilyId = "";
      lookupSearch.unitFamilyId = "";
    }
  }
);

function defaultValue(column) {
  if (column.input === "boolean") {
    return column.key === "isActive";
  }

  if (column.key === "timezoneName") {
    return "Europe/London";
  }

  if (column.key === "status") {
    return "approved";
  }

  if (column.key === "retailerType") {
    return "SUPERMARKET";
  }

  return "";
}

function findLookupOption(columnKey, value) {
  return (props.lookups?.[columnKey] || []).find((option) => option.id === value) || null;
}

function syncLookupSearchFromValues() {
  Object.keys(lookupSearch).forEach((key) => {
    delete lookupSearch[key];
  });

  Object.entries(props.lookups || {}).forEach(([columnKey]) => {
    const match = findLookupOption(columnKey, editorValues[columnKey]);
    lookupSearch[columnKey] = match ? match.label : "";
  });
}

function loadEditorValues(row = null) {
  Object.keys(editorValues).forEach((key) => {
    delete editorValues[key];
  });

  (props.table?.columns || []).forEach((column) => {
    editorValues[column.key] =
      row && Object.prototype.hasOwnProperty.call(row, column.key)
        ? row[column.key]
        : defaultValue(column);
  });

  syncLookupSearchFromValues();
}

function rowLabel(row) {
  return row.name || row.code || row.displayName || row.email || row.id;
}

function formatValue(column, value) {
  if (column && isLookupColumn(column)) {
    const match = findLookupOption(column.key, value);

    if (match) {
      return match.detail ? `${match.label} (${match.detail})` : match.label;
    }
  }

  if (typeof value === "boolean") {
    return value ? "Yes" : "No";
  }

  if (value === null || value === undefined || value === "") {
    return "Not set";
  }

  return value;
}

function fieldDescription(column) {
  if (column.description) {
    return column.description;
  }

  return column.required ? "Required field." : "Optional field.";
}

function isLookupColumn(column) {
  return Array.isArray(props.lookups?.[column.key]) && props.lookups[column.key].length > 0;
}

function filteredLookupOptions(column) {
  const options = props.lookups?.[column.key] || [];
  const search = String(lookupSearch[column.key] || "").trim().toLowerCase();

  if (!search) {
    return options.slice(0, 30);
  }

  return options
    .filter((option) =>
      [option.label, option.detail]
        .filter(Boolean)
        .some((value) => String(value).toLowerCase().includes(search))
    )
    .slice(0, 30);
}

function selectLookupOption(column, optionId) {
  editorValues[column.key] = optionId;
  const match = findLookupOption(column.key, optionId);
  lookupSearch[column.key] = match ? match.label : "";
}

function isColumnDisabled(column) {
  if (props.disabled || !column.mutable || props.saving) {
    return true;
  }

  return props.table?.id === "units" && column.key === "unitFamilyId" && editorValues.isBaseUnit;
}

function openDetail(row) {
  ui.detailRow = row;
  ui.detailOpen = true;
}

function openCreate() {
  ui.editorMode = "create";
  ui.editingRecordId = "";
  loadEditorValues();
  ui.editorOpen = true;
}

function openEdit(row) {
  ui.editorMode = "edit";
  ui.editingRecordId = row.id;
  loadEditorValues(row);
  ui.editorOpen = true;
}

function toggleRow(rowId) {
  if (bulkSelection.selectedIds.includes(rowId)) {
    bulkSelection.selectedIds = bulkSelection.selectedIds.filter((id) => id !== rowId);
    return;
  }

  bulkSelection.selectedIds = [...bulkSelection.selectedIds, rowId];
}

function toggleVisibleRows() {
  if (allVisibleSelected.value) {
    const visibleIds = new Set(paginatedRows.value.map((row) => row.id));
    bulkSelection.selectedIds = bulkSelection.selectedIds.filter((id) => !visibleIds.has(id));
    return;
  }

  bulkSelection.selectedIds = Array.from(
    new Set([...bulkSelection.selectedIds, ...paginatedRows.value.map((row) => row.id)])
  );
}

async function submitEditor() {
  if (!props.table) {
    return;
  }

  const result = await props.saveRecord({
    tableId: props.table.id,
    mode: ui.editorMode,
    recordId: ui.editingRecordId,
    values: { ...editorValues }
  });

  if (result?.ok) {
    ui.editorOpen = false;
  }
}

async function removeRow(row) {
  if (!props.table) {
    return;
  }

  if (
    !(await props.confirmAction({
      title: `Delete ${props.table.label || "record"}?`,
      message: `Delete or archive ${rowLabel(row)}?`,
      confirmLabel: "Delete record",
      cancelLabel: "Keep record",
      tone: "danger"
    }))
  ) {
    return;
  }

  const result = await props.deleteRecord({
    tableId: props.table.id,
    recordId: row.id
  });

  if (result?.ok && ui.detailRow?.id === row.id) {
    ui.detailOpen = false;
  }
}

async function approveRow(row) {
  if (!props.table || !canApproveRow(row)) {
    return;
  }

  if (
    !(await props.confirmAction({
      title: `Approve ${props.table.label || "record"}?`,
      message: `Approve ${rowLabel(row)} and move it into the live catalog?`,
      confirmLabel: "Approve record",
      cancelLabel: "Keep pending",
      tone: "info"
    }))
  ) {
    return;
  }

  await props.approveRecord({
    tableId: props.table.id,
    recordId: row.id
  });
}

async function removeSelectedRows() {
  if (!props.table || !bulkSelection.selectedIds.length) {
    return;
  }

  const selectedIds = [...bulkSelection.selectedIds];
  if (
    !(await props.confirmAction({
      title: `Delete selected ${props.table.label || "records"}?`,
      message: `Delete or archive ${selectedIds.length} selected record(s)?`,
      confirmLabel: "Delete selected",
      cancelLabel: "Keep records",
      tone: "danger"
    }))
  ) {
    return;
  }

  const result = await props.bulkDeleteRecords({
    tableId: props.table.id,
    recordIds: selectedIds
  });

  if (result?.ok) {
    if (selectedIds.includes(ui.detailRow?.id)) {
      ui.detailOpen = false;
    }
    bulkSelection.selectedIds = [];
  }
}

async function approveSelectedRows() {
  if (!props.table || !selectedPendingApprovalIds.value.length) {
    return;
  }

  const selectedIds = [...selectedPendingApprovalIds.value];
  if (
    !(await props.confirmAction({
      title: `Approve selected ${props.table.label || "records"}?`,
      message: `Approve ${selectedIds.length} selected record(s) that are awaiting approval?`,
      confirmLabel: "Approve selected",
      cancelLabel: "Keep pending",
      tone: "info"
    }))
  ) {
    return;
  }

  const result = await props.bulkApproveRecords({
    tableId: props.table.id,
    recordIds: selectedIds
  });

  if (result?.ok) {
    bulkSelection.selectedIds = bulkSelection.selectedIds.filter((id) => !selectedIds.includes(id));
  }
}
</script>

<template>
  <n-card :title="table?.label || '数据表'" :segmented="{ content: true }">
    <template #header-extra>
      <n-space>
        <n-button size="small" quaternary :disabled="disabled || loading || saving" @click="refreshTable">刷新</n-button>
        <n-button v-if="table?.supportsCreate && canEditTable" size="small" type="primary" :disabled="disabled || saving || !table" @click="openCreate">
          新建
        </n-button>
      </n-space>
    </template>

    <n-text depth="3" style="display: block; margin-bottom: 12px">
      {{ table?.description || '请从左侧选择管理模块。' }}
    </n-text>

    <template v-if="table">
      <n-alert v-if="error" type="error" :title="error" style="margin-bottom: 12px" />

      <n-space v-if="canApproveTable" style="margin-bottom: 12px" wrap>
        <n-button-group size="small">
          <n-button :type="ui.approvalView === 'all' ? 'primary' : 'default'" :disabled="disabled || saving" @click="ui.approvalView = 'all'">全部</n-button>
          <n-button :type="ui.approvalView === 'pending' ? 'primary' : 'default'" :disabled="disabled || saving" @click="ui.approvalView = 'pending'">
            待审批 ({{ pendingApprovalCount }})
          </n-button>
        </n-button-group>
        <n-input v-model:value="ui.search" :disabled="disabled" clearable placeholder="搜索行" style="min-width: 180px" />
        <n-select
          v-if="availableStatuses.length > 1"
          v-model:value="ui.statusFilter"
          :disabled="disabled"
          style="width: 160px"
          :options="availableStatuses.map((s) => ({ label: s === 'all' ? '全部状态' : s, value: s }))"
        />
        <n-select
          v-model:value="ui.pageSize"
          :disabled="disabled"
          style="width: 120px"
          :options="[
            { label: '10/页', value: 10 },
            { label: '20/页', value: 20 },
            { label: '25/页', value: 25 },
            { label: '50/页', value: 50 },
            { label: '100/页', value: 100 }
          ]"
        />
      </n-space>

      <n-card size="small" embedded title="批量选择" style="margin-bottom: 12px">
        <n-space wrap align="center">
          <n-tag size="small">已选 {{ bulkSelection.selectedIds.length }}</n-tag>
          <n-tag v-if="canApproveTable" size="small" type="warning">待批 {{ selectedPendingApprovalIds.length }}</n-tag>
          <n-button quaternary size="small" :disabled="disabled || saving || !paginatedRows.length" @click="toggleVisibleRows">
            {{ allVisibleSelected ? '取消本页' : '选择本页' }}
          </n-button>
          <n-button quaternary size="small" :disabled="disabled || saving || !bulkSelection.selectedIds.length" @click="bulkSelection.selectedIds = []">清空</n-button>
          <n-button v-if="canApproveTable" size="small" :disabled="disabled || saving || !selectedPendingApprovalIds.length" @click="approveSelectedRows">批量批准</n-button>
          <n-button v-if="canDeleteTable" type="error" tertiary size="small" :disabled="disabled || saving || !bulkSelection.selectedIds.length" @click="removeSelectedRows">批量删除</n-button>
        </n-space>
      </n-card>

      <n-spin :show="loading">
        <n-scrollbar x-scrollable style="max-height: 480px">
          <table style="width: 100%; border-collapse: collapse; font-size: 13px">
            <thead>
              <tr>
                <th style="padding: 8px; text-align: left; border-bottom: 1px solid var(--n-border-color)">选</th>
                <th v-for="column in visibleColumns" :key="column.key" style="padding: 8px; text-align: left; border-bottom: 1px solid var(--n-border-color)">
                  {{ column.label }}
                </th>
                <th style="padding: 8px; text-align: left; border-bottom: 1px solid var(--n-border-color)">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in paginatedRows" :key="row.id">
                <td style="padding: 8px; vertical-align: top; border-bottom: 1px solid var(--n-border-color)">
                  <n-checkbox
                    :checked="bulkSelection.selectedIds.includes(row.id)"
                    :disabled="disabled || saving"
                    @update:checked="
                      (checked) => {
                        const has = bulkSelection.selectedIds.includes(row.id);
                        if (checked && !has) bulkSelection.selectedIds.push(row.id);
                        if (!checked && has) {
                          bulkSelection.selectedIds = bulkSelection.selectedIds.filter((id) => id !== row.id);
                        }
                      }
                    "
                  />
                </td>
                <td v-for="column in visibleColumns" :key="column.key" style="padding: 8px; border-bottom: 1px solid var(--n-border-color)">
                  {{ formatValue(column, row[column.key]) }}
                </td>
                <td style="padding: 8px; border-bottom: 1px solid var(--n-border-color); white-space: nowrap">
                  <n-space>
                    <n-button size="tiny" quaternary :disabled="disabled || saving" @click="openDetail(row)">查看</n-button>
                    <n-button v-if="canApproveRow(row)" size="tiny" :disabled="disabled || saving" @click="approveRow(row)">批准</n-button>
                    <n-button v-if="canEditTable" size="tiny" quaternary :disabled="disabled || saving" @click="openEdit(row)">编辑</n-button>
                    <n-button v-if="canDeleteTable" size="tiny" type="error" tertiary :disabled="disabled || saving" @click="removeRow(row)">删除</n-button>
                    <n-text v-if="!canEditTable && !canApproveRow(row)" depth="3">只读</n-text>
                  </n-space>
                </td>
              </tr>
              <tr v-if="!paginatedRows.length">
                <td :colspan="visibleColumns.length + 2" style="padding: 16px">
                  <n-empty
                    :description="
                      loading
                        ? '加载中…'
                        : error
                          ? '加载失败，请查看上方错误。'
                          : ui.approvalView === 'pending'
                            ? '当前视图没有待审批行。'
                            : '没有匹配行。'
                    "
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </n-scrollbar>
      </n-spin>

      <n-space justify="center" style="margin-top: 12px">
        <n-pagination v-model:page="ui.page" :page-count="totalPages" simple />
      </n-space>
    </template>

    <n-modal v-model:show="ui.detailOpen" preset="card" style="width: min(720px, 94vw)" :title="ui.detailRow ? rowLabel(ui.detailRow) : ''">
      <template v-if="ui.detailRow && table">
        <n-descriptions bordered size="small" :column="1">
          <n-descriptions-item v-for="column in table.columns" :key="column.key" :label="column.label">
            {{ formatValue(column, ui.detailRow[column.key]) }}
          </n-descriptions-item>
        </n-descriptions>
        <n-space style="margin-top: 16px" wrap>
          <n-button v-if="canApproveRow(ui.detailRow)" :disabled="disabled || saving" @click="approveRow(ui.detailRow)">批准</n-button>
          <n-button v-if="canEditTable" :disabled="disabled || saving" @click="openEdit(ui.detailRow)">编辑</n-button>
          <n-button v-if="canDeleteTable" type="error" tertiary :disabled="disabled || saving" @click="removeRow(ui.detailRow)">删除</n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal v-model:show="ui.editorOpen" preset="card" style="width: min(720px, 96vw)" :title="table ? (ui.editorMode === 'create' ? '新建' : '编辑') + ' · ' + table.label : ''">
      <template v-if="table">
        <n-form v-if="editableColumns.length" label-placement="top">
          <template v-for="column in editableColumns" :key="column.key">
            <n-form-item v-if="column.input === 'textarea'" :label="column.label">
              <n-input v-model:value="editorValues[column.key]" type="textarea" :rows="4" :disabled="isColumnDisabled(column)" />
              <n-text depth="3" style="font-size: 12px">{{ fieldDescription(column) }}</n-text>
            </n-form-item>
            <n-form-item v-else-if="column.input === 'boolean'" :label="column.label">
              <n-checkbox v-model:checked="editorValues[column.key]" :disabled="isColumnDisabled(column)" />
              <n-text depth="3" style="font-size: 12px">{{ fieldDescription(column) }}</n-text>
            </n-form-item>
            <n-form-item v-else-if="isLookupColumn(column)" :label="column.label">
              <n-input v-model:value="lookupSearch[column.key]" :disabled="isColumnDisabled(column)" placeholder="搜索" />
              <n-select
                :value="editorValues[column.key]"
                :disabled="isColumnDisabled(column)"
                :options="[
                  { label: column.required ? '请选择' : '可选', value: '' },
                  ...filteredLookupOptions(column).map((o) => ({
                    label: o.label + (o.detail ? ` (${o.detail})` : ''),
                    value: o.id
                  }))
                ]"
                @update:value="(v) => selectLookupOption(column, v)"
              />
              <n-text depth="3" style="font-size: 12px">
                <template v-if="table.id === 'units' && column.key === 'unitFamilyId' && editorValues.isBaseUnit">基础单位可自动关联族。</template>
                <template v-else>{{ fieldDescription(column) }}</template>
              </n-text>
            </n-form-item>
            <n-form-item v-else :label="column.label">
              <n-input
                v-model:value="editorValues[column.key]"
                :disabled="isColumnDisabled(column)"
                :placeholder="column.required ? '必填' : '可选'"
              />
              <n-text depth="3" style="font-size: 12px">{{ fieldDescription(column) }}</n-text>
            </n-form-item>
          </template>
        </n-form>
        <n-empty v-else description="此模块当前只读" />
        <n-button v-if="editableColumns.length" type="primary" :disabled="disabled || saving" style="margin-top: 16px" @click="submitEditor">
          {{ saving ? '保存中…' : ui.editorMode === 'create' ? '创建' : '保存' }}
        </n-button>
      </template>
    </n-modal>
  </n-card>
</template>

