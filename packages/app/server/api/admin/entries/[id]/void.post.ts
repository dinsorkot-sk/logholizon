import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).voidJournalEntry(String(getRouterParam(event, 'id') || '')))
