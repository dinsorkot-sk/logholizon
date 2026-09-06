<script setup lang="ts">
import { h, resolveComponent } from 'vue'

definePageMeta({ middleware: 'auth' })

const UBadge = resolveComponent('UBadge')

type AdminStatus = { version: string; database_path: string; integrity: boolean; entities: number; documents: number; backup_interval_hours: number; backup_keep: number }
type BackupInfo = { name: string; size: number; modified: number }
type Delivery = { id: string; rule_id: string; document_id: string; action: string; status: string; attempts: number; last_error: string | null; created_at: string }

const toast = useToast()
const { data: status, status: statusState, error, refresh } = await useFetch<AdminStatus>('/api/admin/status')
const { data: backups, refresh: refreshBackups } = await useFetch<{ items: BackupInfo[] }>('/api/admin/backups')
const { data: deliveries, status: deliveriesStatus, refresh: refreshDeliveries } = await useFetch<{ items: Delivery[]; total: number }>('/api/admin/notification-deliveries')

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