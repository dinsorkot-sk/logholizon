import { coreClient } from '../../core/client'

export default defineEventHandler((event) => {
  const query = getQuery(event)
  const company_id = typeof query.company_id === 'string' ? query.company_id : ''
  const from_currency = typeof query.from_currency === 'string' ? query.from_currency : ''
  const to_currency = typeof query.to_currency === 'string' ? query.to_currency : ''
  const amount = Number(query.amount)
  if (!company_id.trim()) throw createError({ statusCode: 400, statusMessage: 'company_id is required' })
  if (!from_currency.trim() || !to_currency.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'from_currency and to_currency are required' })
  }
  if (!Number.isInteger(amount) || amount < 0) {
    throw createError({ statusCode: 400, statusMessage: 'amount must be an integer >= 0' })
  }
  return coreClient(event).convertMoney({ company_id, amount, from_currency, to_currency })
})
