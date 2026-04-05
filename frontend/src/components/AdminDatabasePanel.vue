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
  }
});

const ui = reactive({
  search: "",
  page: 1,
  pageSize: 10,
  statusFilter: "all",
  detailOpen: false,
  editorOpen: false,
  editorMode: "create",
  detailRow: null,
  editingRecordId: ""
});

const editorValues = reactive({});
const lookupSearch = reactive({});

const availableStatuses = computed(() => {
  const values = new Set(
    props.rows.map((row) => row.status).filter((value) => typeof value === "string" && value)
  );
  return ["all", ...values];
});

const filteredRows = computed(() => {
  const search = ui.search.trim().toLowerCase();

  return props.rows.filter((row) => {
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
const visibleColumns = computed(() => {
  const columns = props.table?.columns || [];
  return columns.filter((column) => column.key !== "id").slice(0, 6);
});
const editableColumns = computed(() =>
  (props.table?.columns || []).filter((column) => column.mutable)
);
const canEditTable = computed(() => editableColumns.value.length > 0);

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
    ui.detailOpen = false;
    ui.editorOpen = false;
  }
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

function formatValue(value) {
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
  if (disabled || !column.mutable || saving) {
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
  if (!props.table || !window.confirm(`Delete or archive ${rowLabel(row)}?`)) {
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
        <button :disabled="disabled || saving || !table" @click="openCreate">Add new row</button>
      </div>
    </div>

    <p class="helper-text">
      {{ table?.description || "Choose a management module from the left navigation." }}
    </p>

    <template v-if="table">
      <div class="admin-toolbar">
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

      <div class="table-wrap">
        <table class="admin-table">
          <thead>
            <tr>
              <th v-for="column in visibleColumns" :key="column.key">{{ column.label }}</th>
              <th class="actions-col">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in paginatedRows" :key="row.id" @click="openDetail(row)">
              <td v-for="column in visibleColumns" :key="column.key">
                <template v-if="typeof row[column.key] === 'boolean'">
                  {{ row[column.key] ? "Yes" : "No" }}
                </template>
                <template v-else>
                  {{ row[column.key] || "—" }}
                </template>
              </td>
              <td class="actions-col" @click.stop>
                <div class="action-row wrap">
                  <button class="ghost-button compact-button" :disabled="disabled || saving" @click="openEdit(row)">
                    Edit
                  </button>
                  <button class="danger-button compact-button" :disabled="disabled || saving" @click="removeRow(row)">
                    Delete
                  </button>
                </div>
              </td>
            </tr>
            <tr v-if="!paginatedRows.length">
              <td :colspan="visibleColumns.length + 1" class="empty-table">
                {{ loading ? "Loading rows..." : "No rows matched the current view." }}
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
            <button class="ghost-button compact-button" :disabled="disabled || saving" @click="openEdit(ui.detailRow)">
              Edit row
            </button>
            <button class="ghost-button compact-button" @click="ui.detailOpen = false">Close</button>
          </div>
        </div>

        <div class="simple-list">
          <div v-for="column in table.columns" :key="column.key" class="list-row">
            <strong>{{ column.label }}</strong>
            <span>{{ ui.detailRow[column.key] ?? "—" }}</span>
          </div>
        </div>

        <div class="action-row wrap">
          <button :disabled="disabled || saving" @click="openEdit(ui.detailRow)">Edit row</button>
          <button class="danger-button" :disabled="disabled || saving" @click="removeRow(ui.detailRow)">
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

        <div class="field-grid">
          <template v-for="column in table.columns" :key="column.key">
            <label v-if="column.input === 'textarea'" class="field full-span">
              <span>{{ column.label }}</span>
              <textarea v-model="editorValues[column.key]" :disabled="disabled || !column.mutable || saving" rows="4" />
            </label>

            <label v-else-if="column.input === 'boolean'" class="field checkbox-stack">
              <span>{{ column.label }}</span>
              <input v-model="editorValues[column.key]" :disabled="disabled || !column.mutable || saving" type="checkbox" />
            </label>

            <label v-else class="field" :class="{ 'full-span': column.input === 'readonly' }">
              <span>{{ column.label }}</span>
              <input
                v-model="editorValues[column.key]"
                :disabled="disabled || !column.mutable || saving"
                :placeholder="column.required ? 'Required' : 'Optional'"
              />
            </label>
          </template>
        </div>

        <div class="action-row wrap">
          <button :disabled="disabled || saving" @click="submitEditor">
            {{ saving ? "Saving..." : ui.editorMode === "create" ? "Create row" : "Save changes" }}
          </button>
        </div>
      </article>
    </div>
  </article>
</template>
