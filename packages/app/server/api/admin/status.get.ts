import { coreClient } from '../../core/client'

export default defineEventHandler(() => coreClient().getAdminStatus())
