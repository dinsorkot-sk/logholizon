<script setup lang="ts">
import { h, resolveComponent } from 'vue'

definePageMeta({ middleware: 'auth' })

const UBadge = resolveComponent('UBadge')

type AdminStatus = { version: string; database_path: string; integrity: boolean; entities: number; documents: number; backup_interval_hours: number; backup_keep: number }
type BackupInfo = { name: string; size: number; modified: number }
type Delivery = { id: string; rule_id: string; document_id: string; action: string; status: string; attempts: number; last_error: string | null; created_at: string }
type Company = { id: string; name: string; base_currency: string }
type Currency = { code: string; name: string; decimals: number }
type FxRate = { id: string; from_currency: string; to_currency: string; rate: number; rate_date: string }
type TaxRule = { id: string; name: string; rate: number; is_inclusive: boolean; is_withholding: boolean }

const toast = useToast()
const { data: status, status: statusState, error, refresh } = await useFetch<AdminStatus>('/api/admin/status')
const { data: backups, refresh: refreshBackups } = await useFetch<{ items: BackupInfo[] }>('/api/admin/backups')
const { data: deliveries, status: deliveriesStatus, refresh: refreshDeliveries } = await useFetch<{ items: Delivery[]; total: number }>('/api/admin/notification-deliveries')
const { data: companies, refresh: refreshCompanies } = await useFetch<Company[]>('/api/admin/companies')
const { data: currencies, refresh: refreshCurrencies } = await useFetch<Currency[]>('/api/admin/currencies')

const creating = ref(false)
const restoring = ref(false)
const restarting = ref(false)
const restoreOpen = ref(false)
const restoreTarget = ref<BackupInfo | null>(null)
const restorePath = ref('')
const restoreConfirm = ref(false)

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

function formatTime(seconds: number) {
  if (!seconds) return '—'
  return new Date(seconds * 1000).toLocaleString()
}

function deliveryColor(deliveryStatus: string) {
  switch (deliveryStatus) {
    case 'delivered': return 'success'
    case 'failed': return 'error'
    default: return 'warning'
  }
}

async function createBackup() {
  creating.value = true
  try {
    const result = await $fetch<{ path: string }>('/api/admin/backup', { method: 'POST' })
    await refreshBackups()
    toast.add({ title: 'Backup created', description: result.path, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to create backup', description: cause?.data?.message || cause?.statusMessage || 'Backup failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    creating.value = false
  }
}

async function downloadBackup(backup: BackupInfo) {
  try {
    const blob = await $fetch<Blob>(`/api/admin/backups/${encodeURIComponent(backup.name)}`)
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = backup.name
    link.click()
    URL.revokeObjectURL(url)
  } catch (cause: any) {
    toast.add({ title: 'Unable to download backup', description: cause?.data?.message || cause?.statusMessage || 'Download failed', color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

function openRestore(backup: BackupInfo) {
  restoreTarget.value = backup
  restorePath.value = backup.name
  restoreConfirm.value = false
  restoreOpen.value = true
}

async function confirmRestore() {
  if (!restorePath.value.trim() || !restoreConfirm.value) return
  restoring.value = true
  try {
    const result = await $fetch<{ message: string }>('/api/admin/restore', {
      method: 'POST',
      body: { path: restorePath.value, force: true }
    })
    restoreOpen.value = false
    toast.add({ title: 'Restore staged', description: result.message, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to restore', description: cause?.data?.message || cause?.statusMessage || 'Restore failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    restoring.value = false
  }
}

async function restartCore() {
  restarting.value = true
  try {
    await $fetch('/api/admin/restart', { method: 'POST' })
    toast.add({ title: 'Core restarting', description: 'The Rust core is restarting. Refresh in a moment.', color: 'success', icon: 'i-lucide-refresh-cw' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to restart core', description: cause?.data?.message || cause?.statusMessage || 'Restart failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    restarting.value = false
  }
}

// --- M2 foundation: companies, currencies, FX, tax ---
const companyForm = reactive({ name: '', base_currency: 'THB' })
const companyError = ref('')
const creatingCompany = ref(false)
const currencyForm = reactive({ code: '', name: '', decimals: 2 })
const currencyError = ref('')
const creatingCurrency = ref(false)
const fxCompany = ref('')
const fxForm = reactive({ from_currency: '', to_currency: '', rate: 0, rate_date: '' })
const fxError = ref('')
const settingFx = ref(false)
const fxRates = ref<FxRate[]>([])
const taxCompany = ref('')
const taxForm = reactive({ name: '', rate: 0.07, is_inclusive: false, is_withholding: false })
const taxError = ref('')
const creatingTax = ref(false)
const taxRules = ref<TaxRule[]>([])
const convertForm = reactive({ company_id: '', amount: 1099, from_currency: '', to_currency: '' })
const convertResult = ref<{ amount: number; currency: string; rate: number } | null>(null)
const convertError = ref('')

async function createCompany() {
  companyError.value = ''
  if (!companyForm.name.trim() || !companyForm.base_currency.trim()) {
    companyError.value = 'name and base_currency are required'
    return
  }
  creatingCompany.value = true
  try {
    await $fetch('/api/admin/companies', { method: 'POST', body: { ...companyForm } })
    companyForm.name = ''
    await refreshCompanies()
    toast.add({ title: 'Company created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    companyError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create company'
  } finally {
    creatingCompany.value = false
  }
}

async function createCurrency() {
  currencyError.value = ''
  if (!currencyForm.code.trim() || !currencyForm.name.trim()) {
    currencyError.value = 'code and name are required'
    return
  }
  creatingCurrency.value = true
  try {
    await $fetch('/api/admin/currencies', { method: 'POST', body: { ...currencyForm } })
    currencyForm.code = ''
    currencyForm.name = ''
    await refreshCurrencies()
    toast.add({ title: 'Currency created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    currencyError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create currency'
  } finally {
    creatingCurrency.value = false
  }
}

async function loadFxRates() {
  if (!fxCompany.value) return
  fxRates.value = await $fetch<FxRate[]>(`/api/admin/companies/${encodeURIComponent(fxCompany.value)}/fx-rates`)
}

async function setFxRate() {
  fxError.value = ''
  if (!fxCompany.value) {
    fxError.value = 'company is required'
    return
  }
  settingFx.value = true
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(fxCompany.value)}/fx-rates`, { method: 'POST', body: { ...fxForm } })
    await loadFxRates()
    toast.add({ title: 'FX rate saved', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    fxError.value = cause?.data?.message || cause?.statusMessage || 'Failed to save FX rate'
  } finally {
    settingFx.value = false
  }
}

async function loadTaxRules() {
  if (!taxCompany.value) return
  taxRules.value = await $fetch<TaxRule[]>(`/api/admin/companies/${encodeURIComponent(taxCompany.value)}/tax-rules`)
}

async function createTaxRule() {
  taxError.value = ''
  if (!taxCompany.value || !taxForm.name.trim()) {
    taxError.value = 'company and name are required'
    return
  }
  creatingTax.value = true
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(taxCompany.value)}/tax-rules`, { method: 'POST', body: { ...taxForm } })
    await loadTaxRules()
    toast.add({ title: 'Tax rule created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    taxError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create tax rule'
  } finally {
    creatingTax.value = false
  }
}

async function previewConvert() {
  convertError.value = ''
  convertResult.value = null
  try {
    convertResult.value = await $fetch('/api/admin/convert', { query: { ...convertForm } })
  } catch (cause: any) {
    convertError.value = cause?.data?.message || cause?.statusMessage || 'Convert failed'
  }
}
</script>

<template>
  <UDashboardPanel id="settings">
    <template #header>
      <UDashboardNavbar title="Settings">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="statusState === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-3xl space-y-6">
        <UAlert v-if="error" color="error" title="Cannot load settings" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>

        <UCard v-else-if="statusState === 'pending'">
          <USkeleton v-for="index in 4" :key="index" class="mb-3 h-8 w-full" />
        </UCard>

        <template v-else-if="status">
          <UCard>
            <template #header>
              <div class="flex items-center justify-between">
                <h2 class="text-sm font-semibold">System status</h2>
                <UBadge :color="status.integrity ? 'success' : 'error'" variant="subtle">
                  {{ status.integrity ? 'Integrity OK' : 'Integrity FAILED' }}
                </UBadge>
              </div>
            </template>
            <dl class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div>
                <dt class="text-xs text-muted">Core version</dt>
                <dd class="font-mono text-sm">{{ status.version }}</dd>
              </div>
              <div>
                <dt class="text-xs text-muted">Database</dt>
                <dd class="truncate font-mono text-sm" :title="status.database_path">{{ status.database_path }}</dd>
              </div>
              <div>
                <dt class="text-xs text-muted">Entities</dt>
                <dd class="text-sm">{{ status.entities }}</dd>
              </div>
              <div>
                <dt class="text-xs text-muted">Documents</dt>
                <dd class="text-sm">{{ status.documents }}</dd>
              </div>
            </dl>
          </UCard>

          <UCard>
            <template #header>
              <div class="flex items-center justify-between">
                <div>
                  <h2 class="text-sm font-semibold">Backups</h2>
                  <p class="text-xs text-muted">Automatic backup every {{ status.backup_interval_hours }}h, keep {{ status.backup_keep }} newest</p>
                </div>
                <UButton size="sm" icon="i-lucide-database-backup" :loading="creating" @click="createBackup">Backup now</UButton>
              </div>
            </template>
            <div v-if="!backups?.items?.length" class="py-8 text-center text-sm text-muted">
              No backups yet. Create your first backup to protect your data.
            </div>
            <UTable v-else :data="backups.items" :columns="[
              { accessorKey: 'name', header: 'Name', cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.name) },
              { accessorKey: 'size', header: 'Size', cell: ({ row }) => formatSize(row.original.size) },
              { accessorKey: 'modified', header: 'Created', cell: ({ row }) => formatTime(row.original.modified) },
              { id: 'actions', header: () => h('span', { class: 'sr-only' }, 'Actions'), cell: ({ row }) => h('div', { class: 'flex justify-end gap-1' }, [
                h(resolveComponent('UButton'), { size: 'xs', variant: 'ghost', icon: 'i-lucide-download', onClick: () => downloadBackup(row.original) }, () => 'Download'),
                h(resolveComponent('UButton'), { size: 'xs', variant: 'ghost', color: 'error', onClick: () => openRestore(row.original) }, () => 'Restore')
              ]) }
            ]" :get-row-id="(row: BackupInfo) => row.name" class="w-full" />
          </UCard>

          <UCard>
            <template #header>
              <div class="flex items-center justify-between">
                <div>
                  <h2 class="text-sm font-semibold">Webhook deliveries</h2>
                  <p class="text-xs text-muted">Latest notification attempts (transition triggers)</p>
                </div>
                <UButton size="sm" variant="ghost" icon="i-lucide-refresh-cw" :loading="deliveriesStatus === 'pending'" @click="refreshDeliveries()">Refresh</UButton>
              </div>
            </template>
            <div v-if="deliveriesStatus === 'pending'" class="space-y-2">
              <USkeleton v-for="index in 3" :key="index" class="h-8 w-full" />
            </div>
            <div v-else-if="!deliveries?.items?.length" class="py-8 text-center text-sm text-muted">
              No deliveries yet. Add a webhook rule, then transition a record.
            </div>
            <UTable v-else :data="deliveries.items" :columns="[
              { accessorKey: 'document_id', header: 'Document', cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.document_id) },
              { accessorKey: 'action', header: 'Action', cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.action) },
              { accessorKey: 'status', header: 'Status', cell: ({ row }) => h(resolveComponent('UBadge'), { color: deliveryColor(row.original.status), variant: 'subtle' }, () => row.original.status) },
              { accessorKey: 'attempts', header: 'Attempts' },
              { accessorKey: 'last_error', header: 'Last error', cell: ({ row }) => h('span', { class: 'truncate text-xs text-muted', title: row.original.last_error || '' }, row.original.last_error || '—') },
              { accessorKey: 'created_at', header: 'When', cell: ({ row }) => h('span', { class: 'text-xs' }, row.original.created_at) }
            ]" :get-row-id="(row: Delivery) => row.id" class="w-full" />
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Core service</h2>
            </template>
            <div class="flex items-center justify-between gap-4">
              <p class="text-sm text-muted">Restart the Rust core process. Use after staging a restore.</p>
              <UButton icon="i-lucide-refresh-cw" variant="outline" :loading="restarting" @click="restartCore">Restart core</UButton>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Companies</h2>
            </template>
            <div class="space-y-3">
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto_auto]">
                <UInput v-model="companyForm.name" placeholder="Acme Co." />
                <UInput v-model="companyForm.base_currency" placeholder="THB" class="sm:w-24" />
                <UButton size="sm" :loading="creatingCompany" @click="createCompany">Add</UButton>
              </div>
              <UAlert v-if="companyError" color="error" :title="companyError" />
              <ol v-if="(companies || []).length" class="space-y-1">
                <li v-for="company in companies || []" :key="company.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
                  <span>{{ company.name }}</span>
                  <UBadge color="neutral" variant="subtle">{{ company.base_currency }}</UBadge>
                </li>
              </ol>
              <p v-else class="text-sm text-muted">No companies yet.</p>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Currencies</h2>
            </template>
            <div class="space-y-3">
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-[auto_1fr_auto_auto]">
                <UInput v-model="currencyForm.code" placeholder="THB" class="sm:w-24" />
                <UInput v-model="currencyForm.name" placeholder="Thai Baht" />
                <UInput v-model.number="currencyForm.decimals" type="number" :min="0" :max="4" class="sm:w-20" aria-label="Decimals" />
                <UButton size="sm" :loading="creatingCurrency" @click="createCurrency">Add</UButton>
              </div>
              <UAlert v-if="currencyError" color="error" :title="currencyError" />
              <ol v-if="(currencies || []).length" class="space-y-1">
                <li v-for="currency in currencies || []" :key="currency.code" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
                  <span class="font-mono">{{ currency.code }} · {{ currency.name }}</span>
                  <UBadge color="neutral" variant="subtle">{{ currency.decimals }}dp</UBadge>
                </li>
              </ol>
              <p v-else class="text-sm text-muted">No currencies yet.</p>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">FX rates</h2>
            </template>
            <div class="space-y-3">
              <USelectMenu
                v-model="fxCompany"
                :items="(companies || []).map(c => ({ label: c.name, value: c.id }))"
                value-key="value"
                placeholder="Select company…"
                class="w-full"
                @update:model-value="loadFxRates"
              />
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_auto_auto_auto]">
                <UInput v-model="fxForm.from_currency" placeholder="USD" />
                <UInput v-model="fxForm.to_currency" placeholder="THB" />
                <UInput v-model.number="fxForm.rate" type="number" :min="0" :step="0.0001" aria-label="Rate" />
                <UInput v-model="fxForm.rate_date" type="date" aria-label="Rate date" />
                <UButton size="sm" :loading="settingFx" :disabled="!fxCompany" @click="setFxRate">Save</UButton>
              </div>
              <UAlert v-if="fxError" color="error" :title="fxError" />
              <ol v-if="fxRates.length" class="space-y-1">
                <li v-for="fx in fxRates" :key="fx.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 font-mono text-sm">
                  <span>{{ fx.from_currency }} → {{ fx.to_currency }} @ {{ fx.rate_date }}</span>
                  <span>{{ fx.rate }}</span>
                </li>
              </ol>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Tax rules</h2>
            </template>
            <div class="space-y-3">
              <USelectMenu
                v-model="taxCompany"
                :items="(companies || []).map(c => ({ label: c.name, value: c.id }))"
                value-key="value"
                placeholder="Select company…"
                class="w-full"
                @update:model-value="loadTaxRules"
              />
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto_auto_auto_auto]">
                <UInput v-model="taxForm.name" placeholder="VAT 7%" />
                <UInput v-model.number="taxForm.rate" type="number" :min="0" :max="1" :step="0.01" aria-label="Rate" />
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="taxForm.is_inclusive" /> Incl</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="taxForm.is_withholding" /> WHT</label>
                <UButton size="sm" :loading="creatingTax" :disabled="!taxCompany" @click="createTaxRule">Add</UButton>
              </div>
              <UAlert v-if="taxError" color="error" :title="taxError" />
              <ol v-if="taxRules.length" class="space-y-1">
                <li v-for="rule in taxRules" :key="rule.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
                  <span>{{ rule.name }} · {{ (rule.rate * 100).toFixed(2) }}%</span>
                  <span class="text-xs text-muted">{{ rule.is_inclusive ? 'incl' : 'excl' }}{{ rule.is_withholding ? ' · wht' : '' }}</span>
                </li>
              </ol>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Convert preview</h2>
            </template>
            <div class="space-y-3">
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto_auto_auto_auto]">
                <USelectMenu
                  v-model="convertForm.company_id"
                  :items="(companies || []).map(c => ({ label: c.name, value: c.id }))"
                  value-key="value"
                  placeholder="Company…"
                />
                <UInput v-model.number="convertForm.amount" type="number" :min="0" aria-label="Amount (minor units)" class="sm:w-28" />
                <UInput v-model="convertForm.from_currency" placeholder="USD" class="sm:w-20" />
                <UInput v-model="convertForm.to_currency" placeholder="THB" class="sm:w-20" />
                <UButton size="sm" @click="previewConvert">Convert</UButton>
              </div>
              <UAlert v-if="convertError" color="error" :title="convertError" />
              <p v-else-if="convertResult" class="font-mono text-sm">{{ convertResult.amount }} {{ convertResult.currency }} @ {{ convertResult.rate }}</p>
              <p v-else class="text-xs text-muted">Amount in minor units, e.g. 1099 = 10.99</p>
            </div>
          </UCard>
        </template>
      </div>

      <UModal v-model:open="restoreOpen" title="Restore backup">
        <template #body>
          <UForm class="space-y-4" @submit="confirmRestore">
            <UFormField label="Backup file">
              <UInput v-model="restorePath" placeholder="core-1234567890.db" />
            </UFormField>
            <UAlert color="error" title="This will replace the current database" description="The current data will be overwritten when the core restarts. This action cannot be undone." />
            <UFormField label="I understand this is destructive">
              <USwitch v-model="restoreConfirm" />
            </UFormField>
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="restoreOpen = false">Cancel</UButton>
            <UButton color="error" :disabled="!restoreConfirm" :loading="restoring" @click="confirmRestore">Stage restore</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>