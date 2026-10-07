import type {ContentKind} from './api'

const paths:Record<ContentKind|'search'|'external'|'download',string>={
  mod:'M3 3h7v3h4V3h7v7h-3v4h3v7h-7v-3h-4v3H3v-7h3v-4H3z',
  plugin:'M7 2h3v5h4V2h3v5h3v8h-3v3h-4v4h-3v-4H7v-3H4V7h3z',
  modpack:'M3 3h18v18H3zM6 6v5h5V6zM13 6v5h5V6zM6 13v5h5v-5zM13 13v5h5v-5z',
  resourcepack:'M3 3h6v6H3zM15 3h6v6h-6zM9 9h6v6H9zM3 15h6v6H3zM15 15h6v6h-6z',
  shader:'M10 2h4v4h-4zM10 18h4v4h-4zM2 10h4v4H2zM18 10h4v4h-4zM4 4h3v3H4zM17 4h3v3h-3zM4 17h3v3H4zM17 17h3v3h-3zM8 8h8v8H8z',
  search:'M4 2h10v2H4zM2 4h2v10H2zM14 4h2v10h-2zM4 14h10v2H4zM14 14h3v3h3v3h2v2h-4v-3h-3v-3h-1z',
  external:'M12 2h10v10h-3V7l-8 8-2-2 8-8h-5zM2 5h7v3H5v11h11v-4h3v7H2z',
  download:'M10 2h4v10h4l-6 6-6-6h4zM3 18h3v3h12v-3h3v6H3z',
}

/** Small flat inventory marks, independent of the project's downloaded artwork. */
export function ContentGlyph({kind}:{kind:keyof typeof paths}) {
  return <svg className={`content-glyph glyph-${kind}`} viewBox="0 0 24 24" fill="currentColor" fillRule="evenodd" shapeRendering="crispEdges" aria-hidden="true"><path d={paths[kind]}/></svg>
}
