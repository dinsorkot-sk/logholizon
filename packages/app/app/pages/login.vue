<script setup lang="ts">
import * as z from 'zod'
import type { AuthFormField, FormSubmitEvent } from '@nuxt/ui'

definePageMeta({ layout: 'auth', middleware: 'auth' })

const { login } = useAuth()
const router = useRouter()
const toast = useToast()

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
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center gap-4 p-4">
    <div class="flex items-center gap-2">
      <span class="text-lg font-bold text-primary">L</span>
      <span class="font-semibold">LOGHOLIZON</span>
    </div>
    <UPageCard class="w-full max-w-md">
      <UAuthForm
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