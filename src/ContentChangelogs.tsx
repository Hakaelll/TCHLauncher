import {useState} from 'react'
import type {ContentPlan} from './api'
import type {PreparedContent} from './contentReview'
import {t} from './locale.ts'

function ReleaseNotes({entry}: {entry:ContentPlan['entries'][number]}) {
  const [open,setOpen]=useState(false)
  const notes=entry.version.changelog
  const available=!!notes?.trim()
  return <details className="content-release" onToggle={event=>setOpen(event.currentTarget.open)}>
    <summary><span className="content-release-name">{entry.project.title}</span>
      <span className="content-release-version">{entry.version.version_number}</span>
      {entry.required&&<small>{t('Dependencia obligatoria')}</small>}
      {!available&&<small>{t('Sin notas publicadas')}</small>}
    </summary>
    {open&&(available?<div className="content-release-text" role="region" tabIndex={0} aria-label={t('Notas de {0}',entry.project.title)}>{notes}</div>
      :<p className="field-help">{t('El autor no ha publicado notas para esta versión.')}</p>)}
  </details>
}

export function ContentChangelogs({selection}: {selection:PreparedContent}) {
  const review=selection.value
  const changed=new Set(review.changes.filter(change=>change.action!=='present').map(change=>change.filename))
  const entries=selection.kind==='updates'?review.plan.entries.filter(entry=>changed.has(entry.file.filename)):review.plan.entries
  if(!entries.length)return null
  return <section className="content-release-notes" aria-labelledby="content-release-title">
    <h3 id="content-release-title">{t('Cambios por proyecto')}</h3>
    {entries.map(entry=><ReleaseNotes key={`${entry.project.id}:${entry.version.id}`} entry={entry}/>)}
  </section>
}
