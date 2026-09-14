export function parseDate(iso: string) {
  // SQLite CURRENT_TIMESTAMP is UTC without a timezone suffix; treat it as UTC.
  const normalized = iso.replace(' ', 'T')
  const withTz = /Z$|[+-]\d{2}:?\d{2}$/.test(normalized) ? normalized : `${normalized}Z`
  return new Date(withTz)
}

export function relativeTime(iso: string) {
  const date = parseDate(iso)
  if (Number.isNaN(date.getTime())) return iso
  const diff = Date.now() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes} min ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days} day${days > 1 ? 's' : ''} ago`
  return date.toLocaleDateString()
}

export function absoluteTime(iso: string) {
  const date = parseDate(iso)
  if (Number.isNaN(date.getTime())) return iso
  return date.toLocaleString()
}

export const actionLabels: Record<string, string> = {
  submit: 'Submit',
  approve: 'Approve',
  reject: 'Reject',
  done: 'Mark Done',
  complete: 'Complete',
  schedule: 'Schedule',
  transition: 'Status changed',
  create: 'Created',
  update: 'Updated',
  delete: 'Deleted',
  import: 'Imported'
}

export function actionLabel(action: string) {
  return actionLabels[action] || action
}
