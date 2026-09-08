<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string }
type Bom = { id: string; product_id: string; name: string; version: number; status: string; lines: { id: string; component_id: string; qty: number }[] }
type MfgOrder = { id: string; bom_id: string; product_id: string; qty: number; warehouse_id: string; status: string; entry_date: string }
type EntityOption = { id: string; label: string }

const toast = useToast()
const { data: companies } = await useFetch<Company[]>('/api/admin/companies')
const companyId = ref('')
watch(companies, (list) => {
  if (!companyId.value && list?.length) companyId.value = list[0]!.id
}, { immediate: true })

const bomsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/boms` : '')
const { data: boms, refresh: refreshBoms } = await useFetch<Bom[]>(bomsUrl, { watch: [bomsUrl], immediate: false })
const ordersUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/mfg-orders` : '')
const { data: orders, refresh: refreshOrders } = await useFetch<MfgOrder[]>(ordersUrl, { watch: [ordersUrl], immediate: false })
const { data: products } = await useFetch<EntityOption[]>('/api/entities/product/options', { query: { limit: 100 } })
const { data: warehouses } = await useFetch<EntityOption[]>('/api/entities/warehouse/options', { query: { limit: 100 } })

const bomForm = reactive({ product_id: '', name: '' })
const bomLines = ref([{ component_id: '', qty: 1 }])
const bomError = ref('')
const orderForm = reactive({ bom_id: '', qty: 1, warehouse_id: '', entry_date: new Date().toISOString().slice(0, 10) })
const orderError = ref('')
const busyId = ref('')

function formatQty(qty: number) {
  return Number(qty).toLocaleString('en-US', { maximumFractionDigits: 3 })
}

async function refreshAll() {
  await Promise.all([refreshBoms(), refreshOrders()])
}

watch(companyId, refreshAll)

function addBomLine() {
  bomLines.value.push({ component_id: '', qty: 1 })
}

function removeBomLine(index: number) {
  if (bomLines.value.length > 1) bomLines.value.splice(index, 1)
}

async function createBom() {
  bomError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/boms`, {
      method: 'POST',
      body: { product_id: bomForm.product_id, name: bomForm.name, lines: bomLines.value.filter(l => l.component_id && l.qty > 0) }
    })
    bomLines.value = [{ component_id: '', qty: 1 }]
    await refreshBoms()
    toast.add({ title: 'BOM created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    bomError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create BOM'
  }
}

async function activateBom(id: string) {
  busyId.value = `${id}:activate`
  try {
    await $fetch(`/api/admin/boms/${encodeURIComponent(id)}/activate`, { method: 'POST' })
    await refreshBoms()
    toast.add({ title: 'BOM activated', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: cause?.data?.message || 'Failed to activate BOM', color: 'error' })
  } finally {
    busyId.value = ''
  }
}

async function createOrder() {
  orderError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/mfg-orders`, { method: 'POST', body: { ...orderForm } })
    await refreshOrders()
    toast.add({ title: 'Order created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    orderError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create order'
  }
}

async function confirmOrder(id: string) {
  busyId.value = `${id}:confirm`
  try {
    await $fetch(`/api/admin/mfg-orders/${encodeURIComponent(id)}/confirm`, { method: 'POST' })
    await refreshOrders()
    toast.add({ title: 'Order confirmed', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: cause?.data?.message || 'Failed to confirm order', color: 'error' })
  } finally {
    busyId.value = ''
  }
}

async function completeOrder(id: string) {
  busyId.value = `${id}:complete`
  try {
    await $fetch(`/api/admin/mfg-orders/${encodeURIComponent(id)}/complete`, { method: 'POST' })
    await refreshOrders()
    toast.add({ title: 'Order completed', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: cause?.data?.message || 'Failed to complete order', color: 'error' })
  } finally {
    busyId.value = ''
  }
}
</script>

<template>
  <UDashboardPanel id="manufacturing">
    <template #header>
      <UDashboardNavbar title="Manufacturing">
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
      </div>

      <div v-if="!companyId" class="py-16 text-center text-sm text-muted">
        Create a company in Settings first.
      </div>

      <template v-else>
        <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header><h2 class="text-sm font-semibold">Bills of materials</h2></template>
            <div class="mb-3 grid grid-cols-2 gap-2">
              <USelectMenu v-model="bomForm.product_id" :items="(products || []).map(p => ({ label: p.label, value: p.id }))" value-key="value" placeholder="Finished product…" />
              <UInput v-model="bomForm.name" placeholder="BOM name" />
            </div>
            <div v-for="(line, index) in bomLines" :key="index" class="mb-2 grid grid-cols-[1fr_auto_auto] gap-2">
              <USelectMenu v-model="line.component_id" :items="(products || []).map(p => ({ label: p.label, value: p.id }))" value-key="value" placeholder="Component…" />
              <UInput v-model.number="line.qty" type="number" :min="0" :step="0.001" class="w-24" aria-label="Quantity" />
              <UButton size="xs" variant="ghost" icon="i-lucide-x" :disabled="bomLines.length <= 1" @click="removeBomLine(index)" />
            </div>
            <div class="flex gap-2">
              <UButton size="sm" variant="outline" icon="i-lucide-plus" @click="addBomLine()">Add line</UButton>
              <UButton size="sm" @click="createBom">Create BOM</UButton>
            </div>
            <UAlert v-if="bomError" color="error" :title="bomError" class="mt-2" />
            <ol v-if="(boms || []).length" class="mt-3 space-y-1">
              <li v-for="bom in boms || []" :key="bom.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ bom.name }} · {{ bom.lines.length }} lines</span>
                <span class="flex items-center gap-1">
                  <UBadge color="neutral" variant="subtle">{{ bom.status }}</UBadge>
                  <UButton v-if="bom.status === 'draft'" size="xs" variant="outline" :loading="busyId === `${bom.id}:activate`" @click="activateBom(bom.id)">Activate</UButton>
                </span>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted">No BOMs yet.</p>
          </UCard>

          <UCard>
            <template #header><h2 class="text-sm font-semibold">Manufacturing orders</h2></template>
            <div class="mb-3 grid grid-cols-2 gap-2">
              <USelectMenu v-model="orderForm.bom_id" :items="(boms || []).map(b => ({ label: b.name, value: b.id }))" value-key="value" placeholder="BOM…" />
              <USelectMenu v-model="orderForm.warehouse_id" :items="(warehouses || []).map(w => ({ label: w.label, value: w.id }))" value-key="value" placeholder="Warehouse…" />
              <UInput v-model.number="orderForm.qty" type="number" :min="0" :step="0.001" aria-label="Quantity" />
              <UInput v-model="orderForm.entry_date" type="date" aria-label="Entry date" />
            </div>
            <UAlert v-if="orderError" color="error" :title="orderError" class="mb-2" />
            <UButton size="sm" @click="createOrder">Create order</UButton>
            <ol v-if="(orders || []).length" class="mt-3 space-y-1">
              <li v-for="order in orders || []" :key="order.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ formatQty(order.qty) }} · {{ order.status }}</span>
                <span class="flex items-center gap-1">
                  <UButton v-if="order.status === 'draft'" size="xs" variant="outline" :loading="busyId === `${order.id}:confirm`" @click="confirmOrder(order.id)">Confirm</UButton>
                  <UButton v-if="order.status === 'confirmed'" size="xs" variant="outline" :loading="busyId === `${order.id}:complete`" @click="completeOrder(order.id)">Complete</UButton>
                </span>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted">No orders yet.</p>
          </UCard>
        </div>
      </template>
    </template>
  </UDashboardPanel>
</template>
