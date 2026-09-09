import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => coreClient(event).listDashboards())

