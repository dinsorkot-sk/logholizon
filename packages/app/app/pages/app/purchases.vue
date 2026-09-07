<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

const trade = useTradeDocs('purchase')
const { companyId, companies, leadColumns, quotations, orders, uoms, products, docForm, lines, docError, creating, busyDoc } = trade
const { formatMoney, refreshAll, addLine, removeLine, createDoc, convertLead, confirmQuote, toInvoice } = trade
const docType = ref<'lead' | 'quotation' | 'order'>('lead')
</script>

<template>
  <UDashboardPanel id="purchases">
    <template #header>
      <UDashboardNavbar title="Purchases">
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
        <USelectMenu v-model="docType" :items="['lead', 'quotation', 'order'].map(t => ({ label: t, value: t }))" value-key="value" class="w-36" aria-label="Document type" />
      </div>

      <div v-if="!companyId" class="py-16 text-center text-sm text-muted">
        Create a company in Settings first.
      </div>

      <template v-else>
        <UCard>
          <template #header>
            <h2 class="text-sm font-semibold">New {{ docType }} (purchase)</h2>
          </template>
          <div class="space-y-2">
            <div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
              <UInput v-model="docForm.partner" placeholder="Vendor" />
              <UInput v-model="docForm.currency" placeholder="THB" />
              <UInput v-model="docForm.entry_date" type="date" aria-label="Entry date" />
            </div>
            <div class="overflow-x-auto">
              <table class="w-full min-w-[640px] text-sm">
                <thead>
                  <tr class="text-left text-xs text-muted">
                    <th class="py-1 pr-2">Product</th>
                    <th class="py-1 pr-2">Description</th>
                    <th class="py-1 pr-2">Qty</th>
                    <th class="py-1 pr-2">UOM</th>
                    <th class="py-1 pr-2">Unit price</th>
                    <th class="py-1" />
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(line, index) in lines" :key="index">
                    <td class="py-1 pr-2">
                      <USelectMenu v-model="line.product_id" :items="(products || []).map(p => ({ label: p.label, value: p.id }))" value-key="value" placeholder="Product…" />
                    </td>
                    <td class="py-1 pr-2"><UInput v-model="line.description" placeholder="Raw material" /></td>
                    <td class="py-1 pr-2"><UInput v-model.number="line.qty" type="number" :min="0" :step="0.001" class="w-24" aria-label="Quantity" /></td>
                    <td class="py-1 pr-2">
                      <USelectMenu v-model="line.uom_id" :items="[{ label: 'Base', value: '' }, ...((uoms || []).map(u => ({ label: u.code, value: u.id })))]" value-key="value" placeholder="UOM…" class="w-28" />
                    </td>
                    <td class="py-1 pr-2"><UInput v-model.number="line.unit_price" type="number" :min="0" :step="0.01" class="w-28" aria-label="Unit price" /></td>
                    <td class="py-1"><UButton size="xs" variant="ghost" icon="i-lucide-x" :disabled="lines.length <= 1" @click="removeLine(index)" /></td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div class="flex gap-2">
              <UButton size="sm" variant="outline" icon="i-lucide-plus" @click="addLine()">Add line</UButton>
              <UButton size="sm" :loading="creating" @click="createDoc(docType)">Create {{ docType }}</UButton>
            </div>
            <UAlert v-if="docError" color="error" :title="docError" />
          </div>
        </UCard>

        <h2 class="mb-2 mt-4 text-sm font-semibold">Leads</h2>
        <div class="flex gap-3 overflow-x-auto pb-2">
          <div v-for="column in leadColumns" :key="column.status" class="w-72 shrink-0 rounded-lg bg-muted/40 p-2">
            <p class="mb-2 text-xs font-semibold uppercase text-muted">{{ column.status }} ({{ column.docs.length }})</p>
            <UCard v-for="doc in column.docs" :key="doc.id" class="mb-2">
              <p class="text-sm font-medium">{{ doc.partner }}</p>
              <p class="font-mono text-xs text-muted">{{ formatMoney(doc.subtotal) }} {{ doc.currency }}</p>
              <div class="mt-2 flex gap-1">
                <UButton v-if="doc.status === 'new'" size="xs" variant="outline" :loading="busyDoc === `${doc.id}:convert`" @click="convertLead(doc.id)">Convert</UButton>
                <UBadge v-else color="neutral" variant="subtle">{{ doc.status }}</UBadge>
              </div>
            </UCard>
            <p v-if="!column.docs.length" class="text-xs text-muted">No leads.</p>
          </div>
        </div>

        <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header><h2 class="text-sm font-semibold">Quotations</h2></template>
            <ol v-if="(quotations || []).length" class="space-y-1">
              <li v-for="doc in quotations || []" :key="doc.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ doc.partner }} · {{ formatMoney(doc.subtotal) }}</span>
                <span class="flex items-center gap-1">
                  <UBadge color="neutral" variant="subtle">{{ doc.status }}</UBadge>
                  <UButton v-if="doc.status === 'draft' || doc.status === 'sent'" size="xs" variant="outline" :loading="busyDoc === `${doc.id}:confirm`" @click="confirmQuote(doc.id)">Confirm</UButton>
                </span>
              </li>
            </ol>
            <p v-else class="text-sm text-muted">No quotations yet.</p>
          </UCard>
          <UCard>
            <template #header><h2 class="text-sm font-semibold">Orders</h2></template>
            <ol v-if="(orders || []).length" class="space-y-1">
              <li v-for="doc in orders || []" :key="doc.id" class="flex items-center justify-between gap-2 rounded bg-muted/40 px-3 py-2 text-sm">
                <span class="font-mono">{{ doc.partner }} · {{ formatMoney(doc.subtotal) }}</span>
                <span class="flex items-center gap-1">
                  <UBadge color="neutral" variant="subtle">{{ doc.status }}{{ doc.invoice_id ? ' · invoiced' : '' }}</UBadge>
                  <UButton v-if="!doc.invoice_id && (doc.status === 'confirmed' || doc.status === 'done')" size="xs" variant="outline" :loading="busyDoc === `${doc.id}:invoice`" @click="toInvoice(doc.id)">To invoice</UButton>
                </span>
              </li>
            </ol>
            <p v-else class="text-sm text-muted">No orders yet.</p>
          </UCard>
        </div>
      </template>
    </template>
  </UDashboardPanel>
</template>
