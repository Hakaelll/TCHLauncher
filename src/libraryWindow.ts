import type { Server } from './api'

export type LibraryFilter = 'all' | 'running' | 'attention' | 'pinned'
export type LibraryOrder = 'existing' | 'name' | 'recent'
export function matchesLibraryFilter(server: Server, filter: LibraryFilter) {
  return filter === 'all' || (filter === 'running' && server.state === 'running')
    || (filter === 'attention' && ['crashed', 'error'].includes(server.state))
    || (filter === 'pinned' && server.pinned)
}
export function libraryWindow(servers: Server[], query: string, filter: LibraryFilter, order: LibraryOrder) {
  const search = query.trim().toLocaleLowerCase('es')
  const rows = servers.filter(server => matchesLibraryFilter(server, filter)
    && `${server.name} ${server.platform} ${server.version}`.toLocaleLowerCase('es').includes(search))
  if (order === 'name') rows.sort((a, b) => a.name.localeCompare(b.name, 'es', {numeric:true, sensitivity:'base'}))
  if (order === 'recent') rows.sort((a, b) => (Date.parse(b.createdAt) || 0) - (Date.parse(a.createdAt) || 0))
  return rows
}
