<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string; base_currency: string }
type GlAccount = { id: string; code: string; name: string; type: string }
type JournalLine = { account_id: string; debit: number; credit: number; memo: string }
type TrialRow = { account_id: string; code: string; name: string; type: string; debit: number; credit: number; balance: number }
type TrialBalance = { company_id: string; total_debit: number; total_credit: number; rows: TrialRow[] }
type PeriodLock = { company_id: string; period: string }

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
  await Promise.all([refreshAccounts(), refreshTrial(), refreshLocks()])
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
              <USelectMenu v-model="entryForm.debitAccount" :items="(accounts || []).map(a => ({ label: `${a.code} ${a.name}`, value: a.id }))" value-key="value" placeholder="Debit account…" />
              <USelectMenu v-model="entryForm.creditAccount" :items="(accounts || []).map(a => ({ label: `${a.code} ${a.name}`, value: a.id }))" value-key="value" placeholder="Credit account…" />
              <UAlert v-if="entryError" color="error" :title="entryError" />
              <UButton :loading="posting" @click="postEntry">Post balanced entry</UButton>
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
      </template>
    </template>
  </UDashboardPanel>
</template>
