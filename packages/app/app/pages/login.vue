<script setup lang="ts">
import * as z from 'zod'
import type { AuthFormField, FormSubmitEvent } from '@nuxt/ui'

definePageMeta({ layout: 'auth', middleware: 'auth' })

const { login } = useAuth()
const router = useRouter()
const toast = useToast()

const { data: authStatus } = await useFetch<{ has_users: boolean }>('/api/auth/status')
const isFirstRun = computed(() => authStatus.value && !authStatus.value.has_users)

const fields: AuthFormField[] = [
  {
    name: 'username',
    type: 'text',
    label: 'Username',
    placeholder: 'Enter your username',
    required: true
  },
  {
    name: 'password',
    type: 'password',
    label: 'Password',
    placeholder: 'Enter your password',
    required: true
  }
]

const schema = z.object({
  username: z.string('Username is required').min(1, 'Username is required'),
  password: z.string('Password is required').min(1, 'Password is required')
})

type Schema = z.output<typeof schema>

const setupSchema = z.object({
  username: z.string('Username is required').min(1, 'Username is required'),
  password: z.string('Password must be at least 8 characters').min(8, 'Password must be at least 8 characters')
})

type SetupSchema = z.output<typeof setupSchema>

const setupFields: AuthFormField[] = [
  {
    name: 'username',
    type: 'text',
    label: 'Admin username',
    placeholder: 'Choose an admin username',
    required: true
  },
  {
    name: 'password',
    type: 'password',
    label: 'Admin password',
    placeholder: 'At least 8 characters',
    required: true
  }
]

const error = ref('')
const submitting = ref(false)

async function onSubmit(event: FormSubmitEvent<Schema>) {
  error.value = ''
  submitting.value = true
  try {
    await login(event.data.username, event.data.password)
    toast.add({ title: 'Welcome back', color: 'success', icon: 'i-lucide-check' })
    await router.push('/dashboard')
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Login failed'
  } finally {
    submitting.value = false
  }
}

async function onSetup(event: FormSubmitEvent<SetupSchema>) {
  error.value = ''
  submitting.value = true
  try {
    await $fetch('/api/auth/register', {
      method: 'POST',
      body: { username: event.data.username, password: event.data.password }
    })
    await login(event.data.username, event.data.password)
    toast.add({ title: 'Admin account created', color: 'success', icon: 'i-lucide-check' })
    await router.push('/dashboard')
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Setup failed'
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center gap-4 p-4">
    <div class="flex items-center gap-2">
      <span class="text-lg font-bold text-primary">L</span>
      <span class="font-semibold">LOGHOLIZON</span>
    </div>
    <UPageCard class="w-full max-w-md">
      <UAuthForm
        v-if="isFirstRun"
        :schema="setupSchema"
        :fields="setupFields"
        :loading="submitting"
        title="Set up LOGHOLIZON"
        description="Create the first admin account to get started."
        icon="i-lucide-user-plus"
        :submit="{ label: 'Create admin account', block: true }"
        @submit="onSetup"
      >
        <template #validation>
          <UAlert v-if="error" color="error" icon="i-lucide-info" :title="error" />
        </template>
      </UAuthForm>
      <UAuthForm
        v-else
        :schema="schema"
        :fields="fields"
        :loading="submitting"
        title="Welcome back"
        description="Enter your credentials to access your account."
        icon="i-lucide-lock"
        :submit="{ label: 'Sign in', block: true }"
        @submit="onSubmit"
      >
        <template #validation>
          <UAlert v-if="error" color="error" icon="i-lucide-info" :title="error" />
        </template>
      </UAuthForm>
    </UPageCard>
  </div>
</template>