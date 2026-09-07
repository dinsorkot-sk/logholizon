<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'

defineProps<{
  collapsed?: boolean
}>()

const { user, logout } = useAuth()
const router = useRouter()
const colorMode = useColorMode()

const isAdmin = computed(() => user.value?.role === 'admin')
const username = computed(() => user.value?.username || '—')
const role = computed(() => user.value?.role || 'user')
const environment = process.env.NODE_ENV === 'production' ? 'prod' : 'dev'

async function handleLogout() {
  await logout()
  router.push('/login')
}

const items = computed<DropdownMenuItem[][]>(() => {
  const groups: DropdownMenuItem[][] = [
    [
      {
        type: 'label',
        label: `LOGHOLIZON · ${environment}`
      },
      {
        type: 'label',
        label: username.value,
        description: role.value,
        icon: 'i-lucide-user'
      }
    ]
  ]

  if (isAdmin.value) {
    groups.push([
      {
        label: 'Settings',
        icon: 'i-lucide-settings',
        to: '/admin/settings'
      },
      {
        label: 'Users',
        icon: 'i-lucide-users',
        to: '/admin/users'
      },
      {
        label: 'Audit Log',
        icon: 'i-lucide-history',
        to: '/admin/audit'
      }
    ])
  }

  groups.push(
    [
      {
        label: 'Appearance',
        icon: 'i-lucide-sun-moon',
        children: [
          {
            label: 'Light',
            icon: 'i-lucide-sun',
            type: 'checkbox',
            checked: colorMode.preference === 'light',
            onSelect(e: Event) {
              e.preventDefault()
              colorMode.preference = 'light'
            }
          },
          {
            label: 'Dark',
            icon: 'i-lucide-moon',
            type: 'checkbox',
            checked: colorMode.preference === 'dark',
            onSelect(e: Event) {
              e.preventDefault()
              colorMode.preference = 'dark'
            }
          },
          {
            label: 'System',
            icon: 'i-lucide-monitor',
            type: 'checkbox',
            checked: colorMode.preference === 'system',
            onSelect(e: Event) {
              e.preventDefault()
              colorMode.preference = 'system'
            }
          }
        ]
      }
    ],
    [
      {
        label: 'Log out',
        icon: 'i-lucide-log-out',
        onSelect: handleLogout
      }
    ]
  )

  return groups
})
</script>

<template>
  <UDropdownMenu
    :items="items"
    :content="{ align: 'center', collisionPadding: 12 }"
    :ui="{ content: collapsed ? 'w-48' : 'w-(--reka-dropdown-menu-trigger-width)' }"
  >
    <UButton
      icon="i-lucide-user"
      :label="collapsed ? undefined : username"
      :trailing-icon="collapsed ? undefined : 'i-lucide-chevrons-up-down'"
      color="neutral"
      variant="ghost"
      block
      :square="collapsed"
      aria-label="User menu"
      class="data-[state=open]:bg-elevated"
      :ui="{ trailingIcon: 'text-dimmed' }"
    />
  </UDropdownMenu>
</template>
