<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string; base_currency: string }
type Uom = { id: string; code: string; name: string; dimension: string; factor_to_base: number; is_base: boolean }
type StockBalance = { company_id: string; product_id: string; warehouse_id: string; qty_base: number; avg_cost: number; total_value: number }
type StockLedgerEntry = { id: string; product_id: string; warehouse_id: string; move_type: string; qty: number; qty_base: number; unit_cost: number; total_value: number; balance_qty: number; balance_avg: number; entry_date: string }
type EntityOption = { id: string; label: string }

const toast = useToast()
const { data: companies } = await useFetch<Company[]>('/api/admin/companies')
const companyId = ref('')
watch(companies, (list) => {
  if (!companyId.value && list?.length) companyId.value = list[0]!.id
}, { immediate: true })

const uomsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/uoms` : '')
const { data: uoms, refresh: refreshUoms } = await useFetch<Uom[]>(uomsUrl, { watch: [uomsUrl], immediate: false })
const balancesUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/stock-balances` : '')
const { data: balances, refresh: refreshBalances } = await useFetch<StockBalance[]>(balancesUrl, { watch: [balancesUrl], immediate: false })
const ledgerUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/stock-ledger` : '')
const { data: ledger, refresh: refreshLedger } = await useFetch<StockLedgerEntry[]>(ledgerUrl, { watch: [ledgerUrl], immediate: false })
const { data: products } = await useFetch<EntityOption[]>('/api/entities/product/options', { query: { limit: 100 } })
const { data: warehouses } = await useFetch<EntityOption[]>('/api/entities/warehouse/options', { query: { limit: 100 } })

const moveForm = reactive({ product_id: '', warehouse_id: '', move_type: 'in', qty: 1, uom_id: '', unit_cost: 0, entry_date: new Date().toISOString().slice(0, 10) })
const moveError = ref('')
const applying = ref(false)
const uomForm = reactive({ code: '', name: '', dimension: 'qty', factor_to_base: 1, is_base: false })
const uomError = ref('')

function formatMoney(minor: number) {
  return (minor / 100).toFixed(2)
}

function formatQty(qty: number) {
  return Number(qty).toLocaleString('en-US', { maximumFractionDigits: 3 })
}

const totalValue = computed(() => (balances.value || []).reduce((sum, row) => sum + row.total_value, 0))
const lowStock = computed(() => (balances.value || []).filter(row => row.qty_base <= 0))

async function refreshAll() {
  await Promise.all([refreshUoms(), refreshBalances(), refreshLedger()])
}

watch(companyId, refreshAll)

async function applyMove() {
  moveError.value = ''
  if (!companyId.value) return
  if (!moveForm.product_id || !moveForm.warehouse_id || moveForm.qty <= 0) {
    moveError.value = 'product, warehouse, and qty > 0 are required'
    return
  }
  applying.value = true
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/stock-ledger`, {
      method: 'POST',
      body: {
        product_id: moveForm.product_id,
        warehouse_id: moveForm.warehouse_id,
        move_type: moveForm.move_type,
        qty: moveForm.qty,
        uom_id: moveForm.uom_id || undefined,
        unit_cost: Math.round(moveForm.unit_cost * 100),
        entry_date: moveForm.entry_date
      }
    })
    await refreshAll()
    toast.add({ title: 'Stock move applied', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    moveError.value = cause?.data?.message || cause?.statusMessage || 'Failed to apply move'
  } finally {
    applying.value = false
  }
}

async function createUom() {
  uomError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/uoms`, { method: 'POST', body: { ...uomForm } })
    uomForm.code = ''
    uomForm.name = ''
    await refreshUoms()
    toast.add({ title: 'UOM created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    uomError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create UOM'
  }
}
</script>

<template>
  <UDashboardPanel id="stock">
    <template #header>
      <UDashboardNavbar title="Stock">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" @click="refreshAll()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mb-4 flex flex-wrap items-center gap-2">
        <USelectMenu
          v-model="companyId"
          :items="(companies || []).map(c => ({ label: c.name, value: c.id }))"
          value-key="value"
          placeholder="Select company…"
          class="w-56"
          aria-label="Select company"
        />
        <p v-if="balances" class="font-mono text-xs text-muted">Valuation {{ formatMoney(totalValue) }} · {{ (balances || []).length }} balances</p>
      </div>

      <div v-if="!companyId" class="py-16 text-center text-sm text-muted">
        Create a company in Settings first.
      </div>

      <template v-else>
        <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Apply stock move</h2>
            </template>
            <div class="space-y-2">
              <div class="grid grid-cols-2 gap-2">
                <USelectMenu v-model="moveForm.product_id" :items="(products || []).map(p => ({ label: p.label, value: p.id }))" value-key="value" placeholder="Product…" />
                <USelectMenu v-model="moveForm.warehouse_id" :items="(warehouses || []).map(w => ({ label: w.label, value: w.id }))" value-key="value" placeholder="Warehouse…" />
              </div>
              <div class="grid grid-cols-3 gap-2">
                <USelectMenu v-model="moveForm.move_type" :items="['in', 'out'].map(k => ({ label: k, value: k }))" value-key="value" />
                <UInput v-model.number="moveForm.qty" type="number" :min="0" :step="0.001" aria-label="Quantity" />
                <UInput v-model="moveForm.entry_date" type="date" aria-label="Entry date" />
              </div>
              <div class="grid grid-cols-2 gap-2">
                <USelectMenu v-model="moveForm.uom_id" :items="[{ label: 'Base units', value: '' }, ...((uoms || []).map(u => ({ label: `${u.code} ×${u.factor_to_base}`, value: u.id })))]" value-key="value" placeholder="UOM…" />
                <UInput v-model.number="moveForm.unit_cost" type="number" :min="0" :step="0.01" aria-label="Unit cost (in moves)" />
              </div>
              <UAlert v-if="moveError" color="error" :title="moveError" />
              <UButton size="sm" :loading="applying" @click="applyMove">Apply move</UButton>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Units of measure</h2>
            </template>
            <div class="mb-3 grid grid-cols-2 gap-2">
              <UInput v-model="uomForm.code" placeholder="BOX" />
              <UInput v-model="uomForm.name" placeholder="Box" />
              <USelectMenu v-model="uomForm.dimension" :items="['qty', 'weight', 'length', 'volume'].map(d => ({ label: d, value: d }))" value-key="value" />
              <UInput v-model.number="uomForm.factor_to_base" type="number" :min="0" :step="0.001" aria-label="Factor to base" />
            </div>
            <UAlert v-if="uomError" color="error" :title="uomError" class="mb-2" />
            <UButton size="sm" @click="createUom">Add UOM</UButton>
            <ol v-if="(uoms || []).length" class="mt-3 space-y-1">
              <li v-for="uom in uoms || []" :key="uom.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 font-mono text-sm">
                <span>{{ uom.code }} · {{ uom.name }} · ×{{ uom.factor_to_base }}</span>
                <UBadge color="neutral" variant="subtle">{{ uom.dimension }}{{ uom.is_base ? ' · base' : '' }}</UBadge>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted">No UOMs yet.</p>
          </UCard>
        </div>

        <UCard class="mt-4">
          <template #header>
            <h2 class="text-sm font-semibold">On-hand balances</h2>
          </template>
          <div class="overflow-x-auto">
            <UTable
              :data="balances || []"
              :columns="[
                { accessorKey: 'product_id', header: 'Product' },
                { accessorKey: 'warehouse_id', header: 'Warehouse' },
                { accessorKey: 'qty_base', header: 'On-hand', cell: ({ row }) => formatQty(row.original.qty_base) },
                { accessorKey: 'avg_cost', header: 'Avg cost', cell: ({ row }) => formatMoney(row.original.avg_cost) },
                { accessorKey: 'total_value', header: 'Value', cell: ({ row }) => formatMoney(row.original.total_value) }
              ]"
              :get-row-id="(row: StockBalance) => `${row.product_id}-${row.warehouse_id}`"
              class="min-w-[640px]"
            />
          </div>
          <p v-if="lowStock.length" class="mt-2 text-xs text-warning">Out of stock: {{ lowStock.length }} line(s).</p>
        </UCard>

        <UCard class="mt-4">
          <template #header>
            <h2 class="text-sm font-semibold">Ledger entries</h2>
          </template>
          <div class="overflow-x-auto">
            <UTable
              :data="ledger || []"
              :columns="[
                { accessorKey: 'entry_date', header: 'Date' },
                { accessorKey: 'product_id', header: 'Product' },
                { accessorKey: 'move_type', header: 'Move' },
                { accessorKey: 'qty_base', header: 'Qty base', cell: ({ row }) => formatQty(row.original.qty_base) },
                { accessorKey: 'total_value', header: 'Value', cell: ({ row }) => formatMoney(row.original.total_value) },
                { accessorKey: 'balance_qty', header: 'Balance', cell: ({ row }) => formatQty(row.original.balance_qty) }
              ]"
              :get-row-id="(row: StockLedgerEntry) => row.id"
              class="min-w-[640px]"
            />
          </div>
        </UCard>
      </template>
    </template>
  </UDashboardPanel>
</template>
