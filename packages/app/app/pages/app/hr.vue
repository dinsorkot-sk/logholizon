<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type Company = { id: string; name: string }
type Employee = { id: string; code: string; name: string; base_salary: number; currency: string; hire_date: string; status: string }
type LeaveRequest = { id: string; employee_id: string; kind: string; from_date: string; to_date: string; status: string }
type PayrollRun = { id: string; period: string; entry_date: string; status: string; payslips: { id: string; employee_id: string; gross: number; deductions: number; net: number }[]; total_gross: number; total_net: number }

const toast = useToast()
const { data: companies } = await useFetch<Company[]>('/api/admin/companies')
const companyId = ref('')
watch(companies, (list) => {
  if (!companyId.value && list?.length) companyId.value = list[0]!.id
}, { immediate: true })

const employeesUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/employees` : '')
const { data: employees, refresh: refreshEmployees } = await useFetch<Employee[]>(employeesUrl, { watch: [employeesUrl], immediate: false })
const leavesUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/leaves` : '')
const { data: leaves, refresh: refreshLeaves } = await useFetch<LeaveRequest[]>(leavesUrl, { watch: [leavesUrl], immediate: false })
const runsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/payroll-runs` : '')
const { data: runs, refresh: refreshRuns } = await useFetch<PayrollRun[]>(runsUrl, { watch: [runsUrl], immediate: false })

const employeeForm = reactive({ code: '', name: '', base_salary: 0, currency: 'THB', hire_date: new Date().toISOString().slice(0, 10) })
const employeeError = ref('')
const leaveForm = reactive({ employee_id: '', kind: 'annual', from_date: new Date().toISOString().slice(0, 10), to_date: new Date().toISOString().slice(0, 10) })
const leaveError = ref('')
const runForm = reactive({ period: new Date().toISOString().slice(0, 7), entry_date: new Date().toISOString().slice(0, 10) })
const runError = ref('')
const slipForm = reactive({ run_id: '', employee_id: '', gross: 0, deductions: 0 })
const slipError = ref('')
const busyId = ref('')

function formatMoney(minor: number) {
  return (minor / 100).toFixed(2)
}

async function refreshAll() {
  await Promise.all([refreshEmployees(), refreshLeaves(), refreshRuns()])
}

watch(companyId, refreshAll)

async function createEmployee() {
  employeeError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/employees`, {
      method: 'POST',
      body: { ...employeeForm, base_salary: Math.round(employeeForm.base_salary * 100) }
    })
    employeeForm.code = ''
    employeeForm.name = ''
    await refreshEmployees()
    toast.add({ title: 'Employee created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    employeeError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create employee'
  }
}

async function requestLeave() {
  leaveError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/leaves`, { method: 'POST', body: { ...leaveForm } })
    await refreshLeaves()
    toast.add({ title: 'Leave requested', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    leaveError.value = cause?.data?.message || cause?.statusMessage || 'Failed to request leave'
  }
}

async function decideLeave(id: string, approve: boolean) {
  busyId.value = `${id}:${approve ? 'approve' : 'reject'}`
  try {
    await $fetch(`/api/admin/leaves/${encodeURIComponent(id)}/${approve ? 'approve' : 'reject'}`, { method: 'POST' })
    await refreshLeaves()
    toast.add({ title: approve ? 'Leave approved' : 'Leave rejected', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: cause?.data?.message || 'Failed to decide leave', color: 'error' })
  } finally {
    busyId.value = ''
  }
}

async function createRun() {
  runError.value = ''
  if (!companyId.value) return
  try {
    await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/payroll-runs`, { method: 'POST', body: { ...runForm } })
    await refreshRuns()
    toast.add({ title: 'Payroll run created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    runError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create run'
  }
}

async function addPayslip() {
  slipError.value = ''
  if (!slipForm.run_id || !slipForm.employee_id) {
    slipError.value = 'run and employee are required'
    return
  }
  try {
    await $fetch(`/api/admin/payroll-runs/${encodeURIComponent(slipForm.run_id)}/payslips`, {
      method: 'POST',
      body: { employee_id: slipForm.employee_id, gross: Math.round(slipForm.gross * 100), deductions: Math.round(slipForm.deductions * 100) }
    })
    await refreshRuns()
    toast.add({ title: 'Payslip added', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    slipError.value = cause?.data?.message || cause?.statusMessage || 'Failed to add payslip'
  }
}

async function postRun(id: string) {
  busyId.value = `${id}:post`
  try {
    await $fetch(`/api/admin/payroll-runs/${encodeURIComponent(id)}/post`, { method: 'POST' })
    await refreshRuns()
    toast.add({ title: 'Payroll posted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({ title: cause?.data?.message || 'Failed to post payroll', color: 'error' })
  } finally {
    busyId.value = ''
  }
}
</script>

<template>
  <UDashboardPanel id="hr">
    <template #header>
      <UDashboardNavbar title="HR">
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
            <template #header><h2 class="text-sm font-semibold">Employees</h2></template>
            <div class="mb-3 grid grid-cols-2 gap-2">
              <UInput v-model="employeeForm.code" placeholder="E001" />
              <UInput v-model="employeeForm.name" placeholder="Name" />
              <UInput v-model.number="employeeForm.base_salary" type="number" :min="0" :step="0.01" aria-label="Base salary" />
              <UInput v-model="employeeForm.hire_date" type="date" aria-label="Hire date" />
            </div>
            <UAlert v-if="employeeError" color="error" :title="employeeError" class="mb-2" />
            <UButton size="sm" @click="createEmployee">Add employee</UButton>
            <ol v-if="(employees || []).length" class="mt-3 space-y-1">
              <li v-for="employee in employees || []" :key="employee.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 font-mono text-sm">
                <span>{{ employee.code }} · {{ employee.name }}</span>
                <UBadge color="neutral" variant="subtle">{{ employee.status }} · {{ formatMoney(employee.base_salary) }}</UBadge>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted">No employees yet.</p>
          </UCard>

          <UCard>
            <template #header><h2 class="text-sm font-semibold">Leave requests</h2></template>
            <div class="mb-3 grid grid-cols-2 gap-2">
              <USelectMenu v-model="leaveForm.employee_id" :items="(employees || []).map(e => ({ label: `${e.code} ${e.name}`, value: e.id }))" value-key="value" placeholder="Employee…" />
              <USelectMenu v-model="leaveForm.kind" :items="['annual', 'sick', 'unpaid'].map(k => ({ label: k, value: k }))" value-key="value" />
              <UInput v-model="leaveForm.from_date" type="date" aria-label="From date" />
              <UInput v-model="leaveForm.to_date" type="date" aria-label="To date" />
            </div>
            <UAlert v-if="leaveError" color="error" :title="leaveError" class="mb-2" />
            <UButton size="sm" @click="requestLeave">Request leave</UButton>
            <ol v-if="(leaves || []).length" class="mt-3 space-y-1">
              <li v-for="leave in leaves || []" :key="leave.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ leave.kind }} {{ leave.from_date }}→{{ leave.to_date }}</span>
                <span class="flex items-center gap-1">
                  <UBadge color="neutral" variant="subtle">{{ leave.status }}</UBadge>
                  <UButton v-if="leave.status === 'draft'" size="xs" variant="outline" :loading="busyId === `${leave.id}:approve`" @click="decideLeave(leave.id, true)">Approve</UButton>
                  <UButton v-if="leave.status === 'draft'" size="xs" variant="outline" :loading="busyId === `${leave.id}:reject`" @click="decideLeave(leave.id, false)">Reject</UButton>
                </span>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted">No leave requests yet.</p>
          </UCard>
        </div>

        <UCard class="mt-4">
          <template #header><h2 class="text-sm font-semibold">Payroll runs</h2></template>
          <div class="mb-3 flex flex-wrap gap-2">
            <UInput v-model="runForm.period" placeholder="2026-09" aria-label="Period YYYY-MM" class="w-32" />
            <UInput v-model="runForm.entry_date" type="date" aria-label="Entry date" />
            <UButton size="sm" @click="createRun">Create run</UButton>
          </div>
          <UAlert v-if="runError" color="error" :title="runError" class="mb-2" />
          <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-4">
            <USelectMenu v-model="slipForm.run_id" :items="(runs || []).filter(r => r.status === 'draft').map(r => ({ label: r.period, value: r.id }))" value-key="value" placeholder="Draft run…" />
            <USelectMenu v-model="slipForm.employee_id" :items="(employees || []).map(e => ({ label: `${e.code} ${e.name}`, value: e.id }))" value-key="value" placeholder="Employee…" />
            <UInput v-model.number="slipForm.gross" type="number" :min="0" :step="0.01" aria-label="Gross" />
            <UInput v-model.number="slipForm.deductions" type="number" :min="0" :step="0.01" aria-label="Deductions" />
          </div>
          <UAlert v-if="slipError" color="error" :title="slipError" class="mb-2" />
          <UButton size="sm" variant="outline" @click="addPayslip">Add payslip</UButton>
          <ol v-if="(runs || []).length" class="mt-3 space-y-1">
            <li v-for="run in runs || []" :key="run.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
              <span class="font-mono">{{ run.period }} · gross {{ formatMoney(run.total_gross) }} · net {{ formatMoney(run.total_net) }} · {{ run.payslips.length }} slips</span>
              <span class="flex items-center gap-1">
                <UBadge color="neutral" variant="subtle">{{ run.status }}</UBadge>
                <UButton v-if="run.status === 'draft'" size="xs" variant="outline" :loading="busyId === `${run.id}:post`" @click="postRun(run.id)">Post</UButton>
              </span>
            </li>
          </ol>
          <p v-else class="mt-3 text-sm text-muted">No payroll runs yet.</p>
        </UCard>
      </template>
    </template>
  </UDashboardPanel>
</template>
