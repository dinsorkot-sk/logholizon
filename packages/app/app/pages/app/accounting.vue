<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string; base_currency: string }
type GlAccount = { id: string; code: string; name: string; type: string }
type JournalLine = { account_id: string; debit: number; credit: number; memo: string }
type TrialRow = { account_id: string; code: string; name: string; type: string; debit: number; credit: number; balance: number }
type TrialBalance = { company_id: string; total_debit: number; total_credit: number; rows: TrialRow[] }
type PeriodLock = { company_id: string; period: string }
type Invoice = { id: string; kind: string; partner: string; currency: string; base_total: number; status: string; entry_date: string; paid: number; remaining: number }
type Payment = { id: string; kind: string; partner: string; currency: string; amount: number; entry_date: string; allocated: number; remaining: number }

const toast = useToast()
const { data: companies } = await useFetch<Company[]>('/api/admin/companies')
const companyId = ref('')
watch(companies, (list) => {
  if (!companyId.value && list?.length) companyId.value = list[0]!.id
}, { immediate: true })

const accountsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/accounts` : '')
const { data: accounts, refresh: refreshAccounts } = await useFetch<GlAccount[]>(accountsUrl, { watch: [accountsUrl], immediate: false })
const trialUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/trial-balance` : '')
const { data: trial, status: trialStatus, refresh: refreshTrial } = await useFetch<TrialBalance>(trialUrl, { watch: [trialUrl], immediate: false })
const locksUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/locks` : '')
const { data: locks, refresh: refreshLocks } = await useFetch<PeriodLock[]>(locksUrl, { watch: [locksUrl], immediate: false })
const invoicesUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/invoices` : '')
const { data: invoices, refresh: refreshInvoices } = await useFetch<Invoice[]>(invoicesUrl, { watch: [invoicesUrl], immediate: false })
const paymentsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/payments` : '')
const { data: payments, refresh: refreshPayments } = await useFetch<Payment[]>(paymentsUrl, { watch: [paymentsUrl], immediate: false })

const accountForm = reactive({ code: '', name: '', type: 'asset' })
const accountError = ref('')
const entryForm = reactive({ memo: '', entry_date: new Date().toISOString().slice(0, 10), debitAccount: '', creditAccount: '', amount: 0 })
const entryError = ref('')
const posting = ref(false)
const lockPeriodValue = ref('')
const lockError = ref('')

function formatMoney(minor: number) {
  return (minor / 100).toFixed(2)
}

async function refreshAll() {
  await Promise.all([refreshAccounts(), refreshTrial(), refreshLocks(), refreshInvoices(), refreshPayments()])
}

watch(companyId, refreshAll)

async function createAccount() {
  accountError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/accounts`, { method: 'POST', body: { ...accountForm } })
    accountForm.code = ''
    accountForm.name = ''
    await refreshAll()
    toast.add({ title: 'Account created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    accountError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create account'
  }
}

async function postEntry() {
  entryError.value = ''
  if (!companyId.value) return
  if (!entryForm.debitAccount || !entryForm.creditAccount || entryForm.amount <= 0) {
    entryError.value = 'debit, credit, and amount > 0 are required'
    return
  }
  posting.value = true
  try {
    const lines: JournalLine[] = [
      { account_id: entryForm.debitAccount, debit: Math.round(entryForm.amount * 100), credit: 0, memo: '' },
      { account_id: entryForm.creditAccount, debit: 0, credit: Math.round(entryForm.amount * 100), memo: '' }
    ]
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/entries`, {
      method: 'POST',
      body: { memo: entryForm.memo, entry_date: entryForm.entry_date, lines }
    })
    await refreshAll()
    toast.add({ title: 'Journal posted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    entryError.value = cause?.data?.message || cause?.statusMessage || 'Failed to post journal'
  } finally {
    posting.value = false
  }
}

async function lockPeriod() {
  lockError.value = ''
  if (!companyId.value || !lockPeriodValue.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/locks`, { method: 'POST', body: { period: lockPeriodValue.value } })
    lockPeriodValue.value = ''
    await refreshLocks()
    toast.add({ title: 'Period locked', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    lockError.value = cause?.data?.message || cause?.statusMessage || 'Failed to lock period'
  }
}

const invoiceForm = reactive({ kind: 'sale', partner: '', currency: 'THB', entry_date: new Date().toISOString().slice(0, 10), description: '', quantity: 1, unit_price: 0 })
const invoiceError = ref('')
const creatingInvoice = ref(false)
const paymentForm = reactive({ kind: 'receive', partner: '', currency: 'THB', amount: 0, entry_date: new Date().toISOString().slice(0, 10) })
const paymentError = ref('')
const creatingPayment = ref(false)
const allocateForm = reactive({ payment_id: '', invoice_id: '', amount: 0 })
const allocateError = ref('')

async function createInvoice() {
  invoiceError.value = ''
  if (!companyId.value) return
  creatingInvoice.value = true
  try {
    const created = await $fetch<Invoice>(`/api/admin/companies/${encodeURIComponent(companyId.value)}/invoices`, {
      method: 'POST',
      body: {
        kind: invoiceForm.kind,
        partner: invoiceForm.partner,
        currency: invoiceForm.currency,
        entry_date: invoiceForm.entry_date,
        lines: [{ description: invoiceForm.description, quantity: invoiceForm.quantity, unit_price: Math.round(invoiceForm.unit_price * 100) }]
      }
    })
    await $fetch(`/api/admin/invoices/${encodeURIComponent(created.id)}/post`, { method: 'POST' })
    await refreshAll()
    toast.add({ title: 'Invoice posted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    invoiceError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create invoice'
  } finally {
    creatingInvoice.value = false
  }
}

async function createPayment() {
  paymentError.value = ''
  if (!companyId.value) return
  creatingPayment.value = true
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/payments`, {
      method: 'POST',
      body: { ...paymentForm, amount: Math.round(paymentForm.amount * 100) }
    })
    await refreshAll()
    toast.add({ title: 'Payment created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    paymentError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create payment'
  } finally {
    creatingPayment.value = false
  }
}

async function allocatePayment() {
  allocateError.value = ''
  if (!allocateForm.payment_id || !allocateForm.invoice_id || allocateForm.amount <= 0) {
    allocateError.value = 'payment, invoice, and amount > 0 are required'
    return
  }
  try {
    await $fetch(`/api/admin/payments/${encodeURIComponent(allocateForm.payment_id)}/allocate`, {
      method: 'POST',
      body: { invoice_id: allocateForm.invoice_id, amount: Math.round(allocateForm.amount * 100) }
    })
    await refreshAll()
    toast.add({ title: 'Payment allocated', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    allocateError.value = cause?.data?.message || cause?.statusMessage || 'Failed to allocate payment'
  }
}
</script>

<template>
  <UDashboardPanel id="accounting">
    <template #header>
      <UDashboardNavbar title="Accounting">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="trialStatus === 'pending'" @click="refreshAll()">Refresh</UButton>
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
            <template #header>
              <h2 class="text-sm font-semibold">Chart of accounts</h2>
            </template>
            <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-[auto_1fr_auto_auto]">
              <UInput v-model="accountForm.code" placeholder="1000" class="sm:w-24" />
              <UInput v-model="accountForm.name" placeholder="Cash" />
              <USelectMenu v-model="accountForm.type" :items="['asset', 'liability', 'equity', 'income', 'expense'].map(t => ({ label: t, value: t }))" value-key="value" class="sm:w-32" />
              <UButton size="sm" @click="createAccount">Add</UButton>
            </div>
            <UAlert v-if="accountError" color="error" :title="accountError" class="mb-2" />
            <ol v-if="(accounts || []).length" class="space-y-1">
              <li v-for="account in accounts || []" :key="account.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 font-mono text-sm">
                <span>{{ account.code }} · {{ account.name }}</span>
                <UBadge color="neutral" variant="subtle">{{ account.type }}</UBadge>
              </li>
            </ol>
            <p v-else class="text-sm text-muted">No accounts yet.</p>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Post journal</h2>
            </template>
            <div class="space-y-2">
              <UInput v-model="entryForm.memo" placeholder="Memo" />
              <div class="grid grid-cols-2 gap-2">
                <UInput v-model="entryForm.entry_date" type="date" aria-label="Entry date" />
                <UInput v-model.number="entryForm.amount" type="number" :min="0" :step="0.01" aria-label="Amount" />
              </div>
              <div class="grid grid-cols-3 gap-2">
                <USelectMenu v-model="entryForm.debitAccount" :items="(accounts || []).map(a => ({ label: `${a.code} ${a.name}`, value: a.id }))" value-key="value" placeholder="Debit account…" />
                <USelectMenu v-model="entryForm.creditAccount" :items="(accounts || []).map(a => ({ label: `${a.code} ${a.name}`, value: a.id }))" value-key="value" placeholder="Credit account…" />
                <UButton :loading="posting" @click="postEntry">Post balanced entry</UButton>
              </div>
                <UAlert v-if="entryError" color="error" :title="entryError" /> 
            </div>
          </UCard>
        </div>

        <UCard class="mt-4">
          <template #header>
            <div class="flex items-center justify-between">
              <h2 class="text-sm font-semibold">Trial balance</h2>
              <p v-if="trial" class="font-mono text-xs text-muted">Dr {{ formatMoney(trial.total_debit) }} = Cr {{ formatMoney(trial.total_credit) }}</p>
            </div>
          </template>
          <UTable
            :data="trial?.rows || []"
            :columns="[
              { accessorKey: 'code', header: 'Code' },
              { accessorKey: 'name', header: 'Account' },
              { accessorKey: 'type', header: 'Type' },
              { accessorKey: 'debit', header: 'Debit', cell: ({ row }) => formatMoney(row.original.debit) },
              { accessorKey: 'credit', header: 'Credit', cell: ({ row }) => formatMoney(row.original.credit) },
              { accessorKey: 'balance', header: 'Balance', cell: ({ row }) => formatMoney(row.original.balance) }
            ]"
            :get-row-id="(row: TrialRow) => row.account_id"
            class="w-full"
          />
        </UCard>

        <UCard class="mt-4">
          <template #header>
            <h2 class="text-sm font-semibold">Period locks</h2>
          </template>
          <div class="mb-3 flex gap-2">
            <UInput v-model="lockPeriodValue" type="month" aria-label="Period YYYY-MM" />
            <UButton size="sm" @click="lockPeriod">Lock</UButton>
          </div>
          <UAlert v-if="lockError" color="error" :title="lockError" class="mb-2" />
          <ol v-if="(locks || []).length" class="space-y-1">
            <li v-for="lock in locks || []" :key="lock.period" class="rounded bg-muted/40 px-3 py-2 font-mono text-sm">{{ lock.period }}</li>
          </ol>
          <p v-else class="text-sm text-muted">No locked periods.</p>
        </UCard>

        <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Invoices</h2>
            </template>
            <div class="mb-3 space-y-2">
              <div class="grid grid-cols-2 gap-2">
                <USelectMenu v-model="invoiceForm.kind" :items="['sale', 'purchase'].map(k => ({ label: k, value: k }))" value-key="value" />
                <UInput v-model="invoiceForm.partner" placeholder="Partner" />
              </div>
              <div class="grid grid-cols-2 gap-2">
                <UInput v-model="invoiceForm.currency" placeholder="THB" />
                <UInput v-model="invoiceForm.entry_date" type="date" aria-label="Invoice date" />
              </div>
              <UInput v-model="invoiceForm.description" placeholder="Description" />
              <div class="grid grid-cols-2 gap-2">
                <UInput v-model.number="invoiceForm.quantity" type="number" :min="1" aria-label="Quantity" />
                <UInput v-model.number="invoiceForm.unit_price" type="number" :min="0" :step="0.01" aria-label="Unit price" />
              </div>
              <UAlert v-if="invoiceError" color="error" :title="invoiceError" />
              <UButton size="sm" :loading="creatingInvoice" @click="createInvoice">Create + post</UButton>
            </div>
            <ol v-if="(invoices || []).length" class="space-y-1">
              <li v-for="invoice in invoices || []" :key="invoice.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ invoice.partner }} · {{ formatMoney(invoice.base_total) }}</span>
                <UBadge color="neutral" variant="subtle">{{ invoice.status }} · left {{ formatMoney(invoice.remaining) }}</UBadge>
              </li>
            </ol>
            <p v-else class="text-sm text-muted">No invoices yet.</p>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Payments</h2>
            </template>
            <div class="mb-3 space-y-2">
              <div class="grid grid-cols-2 gap-2">
                <USelectMenu v-model="paymentForm.kind" :items="['receive', 'pay'].map(k => ({ label: k, value: k }))" value-key="value" />
                <UInput v-model="paymentForm.partner" placeholder="Partner" />
              </div>
              <div class="grid grid-cols-3 gap-2">
                <UInput v-model="paymentForm.currency" placeholder="THB" />
                <UInput v-model.number="paymentForm.amount" type="number" :min="0" :step="0.01" aria-label="Amount" />
                <UInput v-model="paymentForm.entry_date" type="date" aria-label="Payment date" />
              </div>
              <UAlert v-if="paymentError" color="error" :title="paymentError" />
              <UButton size="sm" :loading="creatingPayment" @click="createPayment">Create payment</UButton>
            </div>
            <ol v-if="(payments || []).length" class="mb-3 space-y-1">
              <li v-for="payment in payments || []" :key="payment.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ payment.partner }} · {{ formatMoney(payment.amount) }}</span>
                <UBadge color="neutral" variant="subtle">left {{ formatMoney(payment.remaining) }}</UBadge>
              </li>
            </ol>
            <div class="space-y-2 border-t pt-3">
              <h3 class="text-xs font-semibold text-muted">Allocate payment</h3>
              <USelectMenu v-model="allocateForm.payment_id" :items="(payments || []).map(p => ({ label: `${p.partner} ${formatMoney(p.remaining)}`, value: p.id }))" value-key="value" placeholder="Payment…" />
              <USelectMenu v-model="allocateForm.invoice_id" :items="(invoices || []).filter(i => i.status !== 'void').map(i => ({ label: `${i.partner} ${formatMoney(i.remaining)}`, value: i.id }))" value-key="value" placeholder="Invoice…" />
              <div class="flex gap-2">
                <UInput v-model.number="allocateForm.amount" type="number" :min="0" :step="0.01" aria-label="Allocate amount" class="flex-1" />
                <UButton size="sm" @click="allocatePayment">Allocate</UButton>
              </div>
              <UAlert v-if="allocateError" color="error" :title="allocateError" />
            </div>
          </UCard>
        </div>
      </template>
    </template>
  </UDashboardPanel>
</template>
