import { coreClient } from '../../../../core/client'

export default defineEventHandler(event => coreClient(event).listPayrollRuns(String(getRouterParam(event, 'id') || '')))
