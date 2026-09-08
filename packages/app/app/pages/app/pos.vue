<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string }
type PosSession = { id: string; name: string; warehouse_id: string; opening_cash: number; closing_cash?: number | null; status: string; entry_date: string; order_count: number; sales_total: number }
type PosOrder = { id: string; partner: string; status: string; tendered: number; change_due: number; total: number; lines: { description: string; qty: number; unit_price: number }[] }
type EntityOption = { id: string; label: string }

const toast = useToast()
const { data: companies } = await useFetch<Company[]>('/api/admin/companies')
const companyId = ref('')
watch(companies, (list) => {
  if (!companyId.value && list?.length) companyId.value = list[0]!.id
}, { immediate: true })

const sessionsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/pos-sessions` : '')
const { data: sessions, refresh: refreshSessions } = await useFetch<PosSession[]>(sessionsUrl, { watch: [sessionsUrl], immediate: false })
const sessionId = ref('')
watch(sessions, (list) => {
  if (!sessionId.value && list?.length) {
    const open = list.find(s => s.status === 'open')
    sessionId.value = (open || list[0])!.id
  }
}, { immediate: true })
const ordersUrl = computed(() => sessionId.value ? `/api/admin/pos-sessions/${encodeURIComponent(sessionId.value)}/orders` : '')
const { data: orders, refresh: refreshOrders } = await useFetch<PosOrder[]>(ordersUrl, { watch: [ordersUrl], immediate: false })
const { data: products } = await useFetch<EntityOption[]>('/api/entities/product/options', { query: { limit: 100 } })
const { data: warehouses } = await useFetch<EntityOption[]>('/api/entities/warehouse/options', { query: { limit: 100 } })

const sessionForm = reactive({ name: '', warehouse_id: '', opening_cash: 0, entry_date: new Date().toISOString().slice(0, 10) })
const sessionError = ref('')
const cart = ref([{ product_id: '', description: '', qty: 1, unit_price: 0 }])
const orderForm = reactive({ partner: 'Walk-in', currency: 'THB', tendered: 0 })
const orderError = ref('')
const closeCash = ref(0)
const closeError = ref('')
const busyId = ref('')

function formatMoney(minor: number) {
  return (minor / 100).toFixed(2)
}

const cartTotal = computed(() => cart.value.reduce((sum, l) => sum + Math.round(l.qty * l.unit_price * 100), 0))

async function refreshAll() {
  await Promise.all([refreshSessions(), refreshOrders()])
}

watch(companyId, refreshAll)
watch(sessionId, () => refreshOrders())

function addCartLine() {
  cart.value.push({ product_id: '', description: '', qty: 1, unit_price: 0 })
}

function removeCartLine(index: number) {
  if (cart.value.length > 1) cart.value.splice(index, 1)
}

async function openSession() {
  sessionError.value = ''
  if (!companyId.value) return
  try {
    const created = await $fetch<PosSession>(`/api/admin/companies/${encodeURIComponent(companyId.value)}/pos-sessions`, {
      method: 'POST',
      body: { ...sessionForm, opening_cash: Math.round(sessionForm.opening_cash * 100) }
    })
    sessionId.value = created.id
    await refreshSessions()
    toast.add({ title: 'Session opened', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    sessionError.value = cause?.data?.message || cause?.statusMessage || 'Failed to open session'
  }
}

async function checkout() {
  orderError.value = ''
  if (!sessionId.value) return
  try {
    await $fetch(`/api/admin/pos-sessions/${encodeURIComponent(sessionId.value)}/orders`, {
      method: 'POST',
      body: {
        partner: orderForm.partner,
        currency: orderForm.currency,
        tendered: Math.round(orderForm.tendered * 100),
        lines: cart.value.filter(l => l.product_id && l.description.trim()).map(l => ({
          product_id: l.product_id,
          description: l.description.trim(),
          qty: l.qty,
          unit_price: Math.round(l.unit_price * 100)
        }))
      }
    })
    cart.value = [{ product_id: '', description: '', qty: 1, unit_price: 0 }]
    await refreshOrders()
    await refreshSessions()
    toast.add({ title: 'Order paid', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    orderError.value = cause?.data?.message || cause?.statusMessage || 'Failed to checkout'
  }
}

async function closeSession() {
  closeError.value = ''
  if (!sessionId.value) return
  busyId.value = `${sessionId.value}:close`
  try {
    await $fetch(`/api/admin/pos-sessions/${encodeURIComponent(sessionId.value)}/close`, {
      method: 'POST',
      body: { closing_cash: Math.round(closeCash.value * 100) }
    })
    await refreshAll()
    toast.add({ title: 'Session closed', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    closeError.value = cause?.data?.message || cause?.statusMessage || 'Failed to close session'
  } finally {
    busyId.value = ''
  }
}
</script>

<template>
  <UDashboardPanel id="pos">
    <template #header>
      <UDashboardNavbar title="POS">
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
        <USelectMenu
          v-model="sessionId"
          :items="(sessions || []).map(s => ({ label: `${s.name} (${s.status})`, value: s.id }))"
          value-key="value"
          placeholder="Select session…"
          class="w-56"
          aria-label="Select session"
        />
      </div>

      <div v-if="!companyId" class="py-16 text-center text-sm text-muted">
        Create a company in Settings first.
      </div>

      <template v-else>
        <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header><h2 class="text-sm font-semibold">Open session</h2></template>
            <div class="grid grid-cols-2 gap-2">
              <UInput v-model="sessionForm.name" placeholder="Shift 1" />
              <USelectMenu v-model="sessionForm.warehouse_id" :items="(warehouses || []).map(w => ({ label: w.label, value: w.id }))" value-key="value" placeholder="Warehouse…" />
              <UInput v-model.number="sessionForm.opening_cash" type="number" :min="0" :step="0.01" aria-label="Opening cash" />
              <UInput v-model="sessionForm.entry_date" type="date" aria-label="Entry date" />
            </div>
            <UAlert v-if="sessionError" color="error" :title="sessionError" class="mt-2" />
            <UButton size="sm" class="mt-2" @click="openSession">Open session</UButton>
            <div class="mt-3 flex flex-wrap items-center gap-2">
              <UInput v-model.number="closeCash" type="number" :min="0" :step="0.01" aria-label="Closing cash" class="w-32" />
              <UButton size="sm" variant="outline" :loading="busyId === `${sessionId}:close`" :disabled="!sessionId" @click="closeSession">Close session</UButton>
            </div>
            <UAlert v-if="closeError" color="error" :title="closeError" class="mt-2" />
          </UCard>

          <UCard>
            <template #header>
              <div class="flex items-center justify-between">
                <h2 class="text-sm font-semibold">Cart</h2>
                <p class="font-mono text-xs text-muted">Total {{ formatMoney(cartTotal) }}</p>
              </div>
            </template>
            <div v-for="(line, index) in cart" :key="index" class="mb-2 grid grid-cols-[1fr_auto_auto_auto] gap-2">
              <USelectMenu v-model="line.product_id" :items="(products || []).map(p => ({ label: p.label, value: p.id }))" value-key="value" placeholder="Product…" @update:model-value="line.description = (products || []).find(p => p.id === line.product_id)?.label || line.description" />
              <UInput v-model.number="line.qty" type="number" :min="0" :step="0.001" class="w-20" aria-label="Quantity" />
              <UInput v-model.number="line.unit_price" type="number" :min="0" :step="0.01" class="w-24" aria-label="Unit price" />
              <UButton size="xs" variant="ghost" icon="i-lucide-x" :disabled="cart.length <= 1" @click="removeCartLine(index)" />
            </div>
            <div class="mb-2"><UButton size="sm" variant="outline" icon="i-lucide-plus" @click="addCartLine()">Add line</UButton></div>
            <div class="grid grid-cols-3 gap-2">
              <UInput v-model="orderForm.partner" placeholder="Walk-in" />
              <UInput v-model="orderForm.currency" placeholder="THB" />
              <UInput v-model.number="orderForm.tendered" type="number" :min="0" :step="0.01" aria-label="Tendered" />
            </div>
            <UAlert v-if="orderError" color="error" :title="orderError" class="mt-2" />
            <UButton size="sm" class="mt-2" :disabled="!sessionId" @click="checkout">Checkout (pay)</UButton>
          </UCard>
        </div>

        <UCard class="mt-4">
          <template #header><h2 class="text-sm font-semibold">Session orders</h2></template>
          <ol v-if="(orders || []).length" class="space-y-1">
            <li v-for="order in orders || []" :key="order.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
              <span class="font-mono">{{ order.partner }} · {{ formatMoney(order.total) }} · change {{ formatMoney(order.change_due) }}</span>
              <UBadge color="neutral" variant="subtle">{{ order.status }}</UBadge>
            </li>
          </ol>
          <p v-else class="text-sm text-muted">No orders yet.</p>
        </UCard>
      </template>
    </template>
  </UDashboardPanel>
</template>
