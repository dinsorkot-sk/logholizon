import { solutionCatalog } from '#shared/utils/solution-catalog'

export default defineEventHandler(() => {
  return solutionCatalog()
})
