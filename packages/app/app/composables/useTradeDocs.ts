export type TradeKind = 'sale' | 'purchase'

export type TradeLineDraft = { product_id: string; description: string; qty: number; uom_id: string; unit_price: number }

export type TradeDocRow = {
  id: string
  company_id: string
  kind: string
  doc_type: string
  status: string
  partner: string
  currency: string
  entry_date: string
  source_id?: string | null
  invoice_id?: string | null
  created_at: string
  lines: { id: string; product_id?: string | null; description: string; qty: number; uom_id?: string | null; unit_price: number }[]
  subtotal: number
  tax_total: number
}

export type EntityOption = { id: string; label: string }
export type UomOption = { id: string; code: string; factor_to_base: number }

export const LEAD_STATUSES = ['new', 'qualified', 'lost'] as const

export function useTradeDocs(kind: TradeKind) {
  const toast = useToast()
  const { data: companies } = useFetch<{ id: string; name: string }[]>('/api/admin/companies')
  const companyId = ref('')
  watch(companies, (list) => {
    if (!companyId.value && list?.length) companyId.value = list[0]!.id
  }, { immediate: true })

  const leadsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/trade-docs?doc_type=lead` : '')
  const { data: leads, refresh: refreshLeads } = useFetch<TradeDocRow[]>(leadsUrl, { watch: [leadsUrl], immediate: false })
  const quotesUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/trade-docs?doc_type=quotation` : '')
  const { data: quotations, refresh: refreshQuotes } = useFetch<TradeDocRow[]>(quotesUrl, { watch: [quotesUrl], immediate: false })
  const ordersUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/trade-docs?doc_type=order` : '')
  const { data: orders, refresh: refreshOrders } = useFetch<TradeDocRow[]>(ordersUrl, { watch: [ordersUrl], immediate: false })
  const uomsUrl = computed(() => companyId.value ? `/api/admin/companies/${encodeURIComponent(companyId.value)}/uoms` : '')
  const { data: uoms, refresh: refreshUoms } = useFetch<UomOption[]>(uomsUrl, { watch: [uomsUrl], immediate: false })
  const { data: products } = useFetch<EntityOption[]>('/api/entities/product/options', { query: { limit: 100 } })

  const kindLeads = computed(() => (leads.value || []).filter(d => d.kind === kind))
  const kindQuotes = computed(() => (quotations.value || []).filter(d => d.kind === kind))
  const kindOrders = computed(() => (orders.value || []).filter(d => d.kind === kind))
  const leadColumns = computed(() => [...LEAD_STATUSES.map(status => ({
    status,
    docs: kindLeads.value.filter(d => d.status === status)
  })), {
    status: 'Unassigned',
    docs: kindLeads.value.filter(d => !(LEAD_STATUSES as readonly string[]).includes(d.status))
  }])

  const docForm = reactive({ partner: '', currency: 'THB', entry_date: new Date().toISOString().slice(0, 10) })
  const lines = ref<TradeLineDraft[]>([{ product_id: '', description: '', qty: 1, uom_id: '', unit_price: 0 }])
  const docError = ref('')
  const creating = ref(false)
  const busyDoc = ref('')

  function formatMoney(minor: number) {
    return (minor / 100).toFixed(2)
  }

  async function refreshAll() {
    await Promise.all([refreshLeads(), refreshQuotes(), refreshOrders(), refreshUoms()])
  }

  watch(companyId, refreshAll)

  function addLine() {
    lines.value.push({ product_id: '', description: '', qty: 1, uom_id: '', unit_price: 0 })
  }

  function removeLine(index: number) {
    if (lines.value.length > 1) lines.value.splice(index, 1)
  }

  async function createDoc(docType: 'lead' | 'quotation' | 'order') {
    docError.value = ''
    if (!companyId.value) return
    if (!docForm.partner.trim()) {
      docError.value = 'partner is required'
      return
    }
    creating.value = true
    try {
      await $fetch(`/api/admin/companies/${encodeURIComponent(companyId.value)}/trade-docs`, {
        method: 'POST',
        body: {
          kind,
          doc_type: docType,
          partner: docForm.partner.trim(),
          currency: docForm.currency.trim() || 'THB',
          entry_date: docForm.entry_date,
          lines: docType === 'lead' && !lines.value.some(l => l.description.trim()) ? [] : lines.value
            .filter(l => l.description.trim())
            .map(l => ({
              product_id: l.product_id || undefined,
              description: l.description.trim(),
              qty: l.qty,
              uom_id: l.uom_id || undefined,
              unit_price: Math.round(l.unit_price * 100)
            }))
        }
      })
      lines.value = [{ product_id: '', description: '', qty: 1, uom_id: '', unit_price: 0 }]
      await refreshAll()
      toast.add({ title: `${docType} created`, color: 'success', icon: 'i-lucide-check' })
    } catch (cause: any) {
      docError.value = cause?.data?.message || cause?.statusMessage || `Failed to create ${docType}`
    } finally {
      creating.value = false
    }
  }

  async function convertLead(id: string) {
    busyDoc.value = `${id}:convert`
    try {
      await $fetch(`/api/admin/trade/${encodeURIComponent(id)}/convert`, { method: 'POST', body: {} })
      await refreshAll()
      toast.add({ title: 'Lead converted to quotation', color: 'success', icon: 'i-lucide-check' })
    } catch (cause: any) {
      toast.add({ title: cause?.data?.message || 'Failed to convert lead', color: 'error' })
    } finally {
      busyDoc.value = ''
    }
  }

  async function confirmQuote(id: string) {
    busyDoc.value = `${id}:confirm`
    try {
      await $fetch(`/api/admin/trade/${encodeURIComponent(id)}/confirm`, { method: 'POST', body: {} })
      await refreshAll()
      toast.add({ title: 'Quotation confirmed to order', color: 'success', icon: 'i-lucide-check' })
    } catch (cause: any) {
      toast.add({ title: cause?.data?.message || 'Failed to confirm quotation', color: 'error' })
    } finally {
      busyDoc.value = ''
    }
  }

  async function toInvoice(id: string) {
    busyDoc.value = `${id}:invoice`
    try {
      await $fetch(`/api/admin/trade/${encodeURIComponent(id)}/to-invoice`, { method: 'POST' })
      await refreshAll()
      toast.add({ title: 'Order invoiced and posted', color: 'success', icon: 'i-lucide-check' })
    } catch (cause: any) {
      toast.add({ title: cause?.data?.message || 'Failed to invoice order', color: 'error' })
    } finally {
      busyDoc.value = ''
    }
  }

  return {
    companies, companyId, leads: kindLeads, quotations: kindQuotes, orders: kindOrders,
    leadColumns, uoms, products, docForm, lines, docError, creating, busyDoc,
    formatMoney, refreshAll, addLine, removeLine, createDoc, convertLead, confirmQuote, toInvoice
  }
}
