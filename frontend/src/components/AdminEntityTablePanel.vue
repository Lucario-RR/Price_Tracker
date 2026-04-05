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
  <article class="panel admin-module">
    <div class="section-head">
      <div>
        <p class="section-kicker">Management table</p>
        <h2>{{ table?.label || "Admin table" }}</h2>
      </div>
      <div class="action-row wrap">
        <button class="ghost-button" :disabled="disabled || loading || saving" @click="refreshTable">
          Refresh
        </button>
        <button
          v-if="table?.supportsCreate && canEditTable"
          :disabled="disabled || saving || !table"
          @click="openCreate"
        >
          Add new row
        </button>
      </div>
    </div>

    <p class="helper-text">
      {{ table?.description || "Choose a management module from the left navigation." }}
    </p>

    <template v-if="table">
      <section v-if="error" class="feedback-banner feedback-danger">
        <div>
          <strong>Unable to load this module</strong>
          <p>{{ error }}</p>
        </div>
      </section>

      <div class="admin-toolbar">
        <div v-if="canApproveTable" class="approval-switch" role="tablist" aria-label="Approval view">
          <button
            :class="['approval-switch-button', ui.approvalView === 'all' ? 'is-active' : '']"
            :disabled="disabled || saving"
            type="button"
            @click="ui.approvalView = 'all'"
          >
            Everything
          </button>
          <button
            :class="['approval-switch-button', ui.approvalView === 'pending' ? 'is-active' : '']"
            :disabled="disabled || saving"
            type="button"
            @click="ui.approvalView = 'pending'"
          >
            Awaiting approval
            <span class="status-pill tone-warn">{{ pendingApprovalCount }}</span>
          </button>
        </div>
        <label class="field">
          <span>Search rows</span>
          <input v-model="ui.search" :disabled="disabled" placeholder="Search inside the table" />
        </label>
        <label v-if="availableStatuses.length > 1" class="field compact-field">
          <span>View</span>
          <select v-model="ui.statusFilter" :disabled="disabled">
            <option v-for="status in availableStatuses" :key="status" :value="status">
              {{ status === "all" ? "All rows" : status }}
            </option>
          </select>
        </label>
        <label class="field compact-field">
          <span>Rows per page</span>
          <select v-model.number="ui.pageSize" :disabled="disabled">
            <option :value="10">10</option>
            <option :value="20">20</option>
            <option :value="25">25</option>
            <option :value="50">50</option>
            <option :value="100">100</option>
          </select>
        </label>
      </div>

      <article class="admin-subpanel">
        <div class="inline-head">
          <h3>Selected rows</h3>
          <div class="action-row wrap">
            <span class="status-pill tone-info">{{ bulkSelection.selectedIds.length }} selected</span>
            <span v-if="canApproveTable" class="status-pill tone-warn">
              {{ selectedPendingApprovalIds.length }} awaiting approval
            </span>
          </div>
        </div>

        <div class="action-row wrap">
          <button
            class="ghost-button"
            :disabled="disabled || saving || !paginatedRows.length"
            @click="toggleVisibleRows"
          >
            {{ allVisibleSelected ? "Clear visible selection" : "Select visible rows" }}
          </button>
          <button
            class="ghost-button"
            :disabled="disabled || saving || !bulkSelection.selectedIds.length"
            @click="bulkSelection.selectedIds = []"
          >
            Clear selection
          </button>
          <button
            v-if="canApproveTable"
            class="ghost-button"
            :disabled="disabled || saving || !selectedPendingApprovalIds.length"
            @click="approveSelectedRows"
          >
            Approve selected
          </button>
          <button
            v-if="canDeleteTable"
            class="danger-button"
            :disabled="disabled || saving || !bulkSelection.selectedIds.length"
            @click="removeSelectedRows"
          >
            Delete selected
          </button>
        </div>
      </article>

      <div class="table-wrap">
        <table class="admin-table">
          <thead>
            <tr>
              <th class="select-col">Select</th>
              <th v-for="column in visibleColumns" :key="column.key">{{ column.label }}</th>
              <th class="actions-col">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in paginatedRows" :key="row.id" @click="openDetail(row)">
              <td class="select-col" @click.stop>
                <input
                  :checked="bulkSelection.selectedIds.includes(row.id)"
                  :disabled="disabled || saving"
                  type="checkbox"
                  @change="toggleRow(row.id)"
                />
              </td>
              <td v-for="column in visibleColumns" :key="column.key">
                {{ formatValue(column, row[column.key]) }}
              </td>
              <td class="actions-col" @click.stop>
                <div v-if="canEditTable || canApproveRow(row)" class="action-row wrap">
                  <button
                    v-if="canApproveRow(row)"
                    class="ghost-button compact-button"
                    :disabled="disabled || saving"
                    @click="approveRow(row)"
                  >
                    Approve
                  </button>
                  <button
                    v-if="canEditTable"
                    class="ghost-button compact-button"
                    :disabled="disabled || saving"
                    @click="openEdit(row)"
                  >
                    Edit
                  </button>
                  <button
                    v-if="canDeleteTable"
                    class="danger-button compact-button"
                    :disabled="disabled || saving"
                    @click="removeRow(row)"
                  >
                    Delete
                  </button>
                </div>
                <span v-else class="table-subtext">Read only</span>
              </td>
            </tr>
            <tr v-if="!paginatedRows.length">
              <td :colspan="visibleColumns.length + 2" class="empty-table">
                {{
                  loading
                    ? "Loading rows..."
                    : error
                      ? "The table could not be loaded. Review the error message above and retry."
                      : ui.approvalView === "pending"
                        ? "No rows are awaiting approval in this module."
                        : "No rows matched the current view."
                }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="admin-pagination">
        <span class="helper-text">Page {{ ui.page }} of {{ totalPages }}</span>
        <div class="action-row wrap">
          <button class="ghost-button compact-button" :disabled="ui.page <= 1" @click="ui.page -= 1">
            Previous
          </button>
          <button class="ghost-button compact-button" :disabled="ui.page >= totalPages" @click="ui.page += 1">
            Next
          </button>
        </div>
      </div>
    </template>

    <div v-if="ui.detailOpen && ui.detailRow" class="admin-modal-backdrop" @click.self="ui.detailOpen = false">
      <article class="admin-modal">
        <div class="section-head">
          <div>
            <p class="section-kicker">Record detail</p>
            <h2>{{ rowLabel(ui.detailRow) }}</h2>
          </div>
          <div class="action-row wrap">
            <button
              v-if="canApproveRow(ui.detailRow)"
              class="ghost-button compact-button"
              :disabled="disabled || saving"
              @click="approveRow(ui.detailRow)"
            >
              Approve row
            </button>
            <button
              v-if="canEditTable"
              class="ghost-button compact-button"
              :disabled="disabled || saving"
              @click="openEdit(ui.detailRow)"
            >
              Edit row
            </button>
            <button class="ghost-button compact-button" @click="ui.detailOpen = false">Close</button>
          </div>
        </div>

        <div class="simple-list">
          <div v-for="column in table.columns" :key="column.key" class="list-row">
            <strong>{{ column.label }}</strong>
            <span>{{ formatValue(column, ui.detailRow[column.key]) }}</span>
          </div>
        </div>

        <div v-if="canEditTable || canApproveRow(ui.detailRow)" class="action-row wrap">
          <button
            v-if="canApproveRow(ui.detailRow)"
            :disabled="disabled || saving"
            @click="approveRow(ui.detailRow)"
          >
            Approve row
          </button>
          <button v-if="canEditTable" :disabled="disabled || saving" @click="openEdit(ui.detailRow)">
            Edit row
          </button>
          <button
            v-if="canDeleteTable"
            class="danger-button"
            :disabled="disabled || saving"
            @click="removeRow(ui.detailRow)"
          >
            Delete row
          </button>
        </div>
      </article>
    </div>

    <div v-if="ui.editorOpen && table" class="admin-modal-backdrop" @click.self="ui.editorOpen = false">
      <article class="admin-modal">
        <div class="section-head">
          <div>
            <p class="section-kicker">{{ ui.editorMode === "create" ? "Add row" : "Edit row" }}</p>
            <h2>{{ table.label }}</h2>
          </div>
          <button class="ghost-button compact-button" @click="ui.editorOpen = false">Close</button>
        </div>

        <div v-if="editableColumns.length" class="field-grid">
          <template v-for="column in editableColumns" :key="column.key">
            <label v-if="column.input === 'textarea'" class="field full-span">
              <span>{{ column.label }}</span>
              <textarea v-model="editorValues[column.key]" :disabled="isColumnDisabled(column)" rows="4" />
              <small class="field-note">{{ fieldDescription(column) }}</small>
            </label>

            <label v-else-if="column.input === 'boolean'" class="field checkbox-stack">
              <span>{{ column.label }}</span>
              <input v-model="editorValues[column.key]" :disabled="isColumnDisabled(column)" type="checkbox" />
              <small class="field-note">{{ fieldDescription(column) }}</small>
            </label>

            <label v-else-if="isLookupColumn(column)" class="field">
              <span>{{ column.label }}</span>
              <input
                v-model="lookupSearch[column.key]"
                :disabled="isColumnDisabled(column)"
                placeholder="Type to search"
              />
              <select
                :value="editorValues[column.key]"
                :disabled="isColumnDisabled(column)"
                @change="selectLookupOption(column, $event.target.value)"
              >
                <option value="">{{ column.required ? "Select an option" : "Optional" }}</option>
                <option
                  v-for="option in filteredLookupOptions(column)"
                  :key="option.id"
                  :value="option.id"
                >
                  {{ option.label }}{{ option.detail ? ` (${option.detail})` : "" }}
                </option>
              </select>
              <small class="field-note">
                <template v-if="table.id === 'units' && column.key === 'unitFamilyId' && editorValues.isBaseUnit">
                  Base units can create or reuse their own family automatically.
                </template>
                <template v-else>
                  {{ fieldDescription(column) }}
                </template>
              </small>
            </label>

            <label v-else class="field">
              <span>{{ column.label }}</span>
              <input
                v-model="editorValues[column.key]"
                :disabled="isColumnDisabled(column)"
                :placeholder="column.required ? 'Required' : 'Optional'"
              />
              <small class="field-note">{{ fieldDescription(column) }}</small>
            </label>
          </template>
        </div>
        <p v-else class="helper-text">
          This module is read only right now, so records can be inspected but not edited from this screen.
        </p>

        <div v-if="editableColumns.length" class="action-row wrap">
          <button :disabled="disabled || saving" @click="submitEditor">
            {{ saving ? "Saving..." : ui.editorMode === "create" ? "Create row" : "Save changes" }}
          </button>
        </div>
      </article>
    </div>
  </article>
</template>
