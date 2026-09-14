<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'

definePageMeta({ middleware: 'auth' })

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type UserRow = { id: string; username: string; role: string; created_at: string }

const toast = useToast()
const { user: currentUser } = useAuth()
const { data: users, status, error, refresh } = await useFetch<UserRow[]>('/api/admin/users')

// --- Create user ---
const createOpen = ref(false)
const createForm = reactive({ username: '', password: '', role: 'user' })
const createError = ref('')
const creating = ref(false)

function openCreate() {
  createForm.username = ''
  createForm.password = ''
  createForm.role = 'user'
  createError.value = ''
  createOpen.value = true
}

async function createUser() {
  createError.value = ''
  if (!createForm.username.trim() || !createForm.password) {
    createError.value = 'username and password are required'
    return
  }
  if (createForm.password.length < 8) {
    createError.value = 'password must be at least 8 characters'
    return
  }
  creating.value = true
  try {
    await $fetch('/api/admin/users', {
      method: 'POST',
      body: { ...createForm }
    })
    createOpen.value = false
    await refresh()
    toast.add({ title: 'User created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    createError.value = e?.data?.message || e?.statusMessage || 'Failed to create user'
  } finally {
    creating.value = false
  }
}

// --- Change role ---
const roleOpen = ref(false)
const roleTarget = ref<UserRow | null>(null)
const roleValue = ref('user')
const roleError = ref('')
const savingRole = ref(false)

function openRole(user: UserRow) {
  roleTarget.value = user
  roleValue.value = user.role
  roleError.value = ''
  roleOpen.value = true
}

async function saveRole() {
  if (!roleTarget.value) return
  roleError.value = ''
  savingRole.value = true
  try {
    await $fetch(`/api/admin/users/${encodeURIComponent(roleTarget.value.id)}`, {
      method: 'PUT',
      body: { role: roleValue.value }
    })
    roleOpen.value = false
    await refresh()
    toast.add({ title: 'Role updated', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    roleError.value = e?.data?.message || e?.statusMessage || 'Failed to update role'
  } finally {
    savingRole.value = false
  }
}

// --- Delete user ---
const deleteOpen = ref(false)
const deleteTarget = ref<UserRow | null>(null)
const deleting = ref(false)

function openDelete(user: UserRow) {
  deleteTarget.value = user
  deleteOpen.value = true
}

async function removeUser() {
  if (!deleteTarget.value) return
  deleting.value = true
  try {
    await $fetch(`/api/admin/users/${encodeURIComponent(deleteTarget.value.id)}`, { method: 'DELETE' })
    deleteOpen.value = false
    deleteTarget.value = null
    await refresh()
    toast.add({ title: 'User deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete user',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deleting.value = false
  }
}

// --- Reset password ---
const resetOpen = ref(false)
const resetTarget = ref<UserRow | null>(null)
const resetPassword = ref('')
const resetError = ref('')
const resetting = ref(false)

function openReset(user: UserRow) {
  resetTarget.value = user
  resetPassword.value = ''
  resetError.value = ''
  resetOpen.value = true
}

async function confirmReset() {
  if (!resetTarget.value) return
  resetError.value = ''
  if (resetPassword.value.length < 8) {
    resetError.value = 'password must be at least 8 characters'
    return
  }
  resetting.value = true
  try {
    await $fetch(`/api/admin/users/${encodeURIComponent(resetTarget.value.id)}/reset-password`, {
      method: 'POST',
      body: { password: resetPassword.value }
    })
    resetOpen.value = false
    toast.add({ title: 'Password reset', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    resetError.value = e?.data?.message || e?.statusMessage || 'Failed to reset password'
  } finally {
    resetting.value = false
  }
}

const userColumns: TableColumn<UserRow>[] = [
  {
    accessorKey: 'username',
    header: 'Username',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.username)
  },
  {
    accessorKey: 'role',
    header: 'Role',
    cell: ({ row }) => h(UBadge, { color: row.original.role === 'admin' ? 'primary' : 'neutral', variant: 'subtle' }, () => row.original.role)
  },
  {
    accessorKey: 'created_at',
    header: 'Created'
  },
  {
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => {
      const isSelf = row.original.id === currentUser.value?.id
      return h('div', { class: 'flex justify-end gap-1' }, [
        h(UButton, { size: 'xs', variant: 'ghost', onClick: () => openRole(row.original) }, () => 'Role'),
        h(UButton, { size: 'xs', variant: 'ghost', onClick: () => openReset(row.original) }, () => 'Reset password'),
        h(UButton, {
          size: 'xs',
          variant: 'ghost',
          color: 'error',
          disabled: isSelf,
          onClick: () => openDelete(row.original)
        }, () => 'Delete')
      ])
    }
  }
]
</script>

<template>
  <UDashboardPanel id="users">
    <template #header>
      <UDashboardNavbar title="Users">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <div class="flex items-center gap-2">
            <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
            <UButton icon="i-lucide-plus" @click="openCreate">New user</UButton>
          </div>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-3xl">
        <UAlert v-if="error" color="error" title="Cannot load users" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>

        <div v-else-if="status === 'pending'" class="space-y-3">
          <USkeleton v-for="index in 3" :key="index" class="h-10 w-full" />
        </div>

        <UCard v-else>
          <UTable :data="users || []" :columns="userColumns" :get-row-id="(row: UserRow) => row.id" class="w-full">
            <template #empty>
              <div class="py-10 text-center text-muted">No users yet.</div>
            </template>
          </UTable>
        </UCard>
      </div>

      <!-- Create user modal -->
      <UModal v-model:open="createOpen" title="New user">
        <template #body>
          <UForm class="space-y-4" @submit="createUser">
            <UFormField label="Username">
              <UInput v-model="createForm.username" placeholder="alice" autocomplete="off" />
            </UFormField>
            <UFormField label="Password" hint="At least 8 characters">
              <UInput v-model="createForm.password" type="password" placeholder="••••••••" autocomplete="new-password" />
            </UFormField>
            <UFormField label="Role">
              <USelectMenu
                v-model="createForm.role"
                :items="[{ label: 'User', value: 'user' }, { label: 'Admin', value: 'admin' }]"
                value-key="value"
                class="w-full"
              />
            </UFormField>
            <UAlert v-if="createError" color="error" :title="createError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="createOpen = false">Cancel</UButton>
            <UButton :loading="creating" @click="createUser">Create user</UButton>
          </div>
        </template>
      </UModal>

      <!-- Change role modal -->
      <UModal v-model:open="roleOpen" title="Change role">
        <template #body>
          <UForm class="space-y-4" @submit="saveRole">
            <p class="text-sm text-muted">
              Change role for <span class="font-mono">{{ roleTarget?.username }}</span>
            </p>
            <UFormField label="Role">
              <USelectMenu
                v-model="roleValue"
                :items="[{ label: 'User', value: 'user' }, { label: 'Admin', value: 'admin' }]"
                value-key="value"
                class="w-full"
              />
            </UFormField>
            <UAlert v-if="roleError" color="error" :title="roleError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="roleOpen = false">Cancel</UButton>
            <UButton :loading="savingRole" @click="saveRole">Save</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete user modal -->
      <UModal v-model:open="deleteOpen" title="Delete user">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the user
            <span class="font-mono">{{ deleteTarget?.username }}</span>. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deleting" @click="removeUser">Delete</UButton>
          </div>
        </template>
      </UModal>

      <!-- Reset password modal -->
      <UModal v-model:open="resetOpen" title="Reset password">
        <template #body>
          <UForm class="space-y-4" @submit="confirmReset">
            <p class="text-sm text-muted">
              Set a new password for <span class="font-mono">{{ resetTarget?.username }}</span>
            </p>
            <UFormField label="New password" hint="At least 8 characters">
              <UInput v-model="resetPassword" type="password" placeholder="••••••••" autocomplete="new-password" />
            </UFormField>
            <UAlert v-if="resetError" color="error" :title="resetError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="resetOpen = false">Cancel</UButton>
            <UButton :loading="resetting" @click="confirmReset">Reset password</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>