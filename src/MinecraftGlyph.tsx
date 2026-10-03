/** Original block geometry for launcher actions; no game textures are bundled. */
export function MinecraftGlyph({kind='world'}:{kind?:'world'|'folder'|'pack'}) {
  return <svg className={`minecraft-glyph ${kind}`} viewBox="0 0 64 64" fill="none" aria-hidden="true">
    {kind==='world' ? <>
      <path className="block-top" d="m32 6 24 13-24 13L8 19 32 6Z"/>
      <path className="block-left" d="M8 19v26l24 13V32L8 19Z"/>
      <path className="block-right" d="M56 19v26L32 58V32l24-13Z"/>
      <path className="block-detail" d="m20 12 24 13M8 19l24 13 24-13M20 26v26M44 26v26M8 32l24 13 24-13"/>
      <path className="block-signal" d="m29 14 7 4-7 4-7-4 7-4Z"/>
    </> : kind==='folder' ? <>
      <path className="block-left" d="M7 13h19l6 7h24v32H7V13Z"/>
      <path className="block-top" d="M7 25h50l-6 27H7V25Z"/>
      <path className="block-detail" d="M7 25h50l-6 27H7V13h19l6 7h24v5"/>
      <path className="block-signal" d="M28 31h7v7h7v7h-7v7h-7v-7h-7v-7h7v-7Z"/>
    </> : <>
      <path className="block-left" d="m7 19 25 13 25-13v29L32 61 7 48V19Z"/>
      <path className="block-top" d="m32 5 25 14-25 13L7 19 32 5Z"/>
      <path className="block-right" d="M32 32v29l25-13V19L32 32Z"/>
      <path className="block-detail" d="m7 19 25 13 25-13M32 32v29M20 12l25 14v14l-7 4V30L13 16"/>
      <path className="block-signal" d="m40 44 9-5v8l-9 5v-8Z"/>
    </>}
  </svg>
}
