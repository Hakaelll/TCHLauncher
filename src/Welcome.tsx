type WelcomeProps = {onCreate:()=>void;onImport:()=>void}

function ChoiceEmblem({kind}: {kind:'create'|'import'}) {
  return <svg className="choice-emblem" viewBox="0 0 120 120" fill="none" aria-hidden="true">
    <path className="emblem-frame" d="m60 7 43 53-43 53L17 60zM60 18 94 60 60 102 26 60z"/>
    <path className="emblem-ticks" d="M60 0v12m0 96v12M9 60h13m76 0h13"/>
    {kind === 'create' ? <g className="emblem-symbol"><path d="m60 35 24 14v27L60 90 36 76V49z"/><path d="m36 49 24 14 24-14M60 63v27M48 42l24 14M60 29v-9m-5 4h10"/></g>
      : <g className="emblem-symbol"><path d="M33 48h21l7 7h27v27H33zM33 48v-9h23l7 7h25v9M60 34V20m-6 8 6-8 6 8"/><path d="m53 70 7 6 10-13"/></g>}
    <path className="emblem-detail" d="m18 22 4 4-4 4-4-4zM102 90l4 4-4 4-4-4z"/>
  </svg>
}

export function Welcome({onCreate,onImport}: WelcomeProps) {
  return <section className="welcome" aria-labelledby="welcome-title">
    <header className="welcome-heading"><h1 id="welcome-title">Tu primer servidor</h1><p>Empieza de cero o trae el que ya tienes.</p></header>
    <div className="server-choices">
      <button className="server-choice create-choice" onClick={onCreate} aria-labelledby="create-choice-title" aria-describedby="create-choice-description">
        <span className="choice-crown" aria-hidden="true"><svg viewBox="0 0 68 18" fill="none"><path d="M0 9h24l10-8 10 8h24M24 9l10 8 10-8"/></svg></span>
        <ChoiceEmblem kind="create"/>
        <span className="choice-copy"><strong id="create-choice-title">Crear servidor</strong><span id="create-choice-description">Elige versión, capacidad y mundo.<br/>Te guiamos paso a paso.</span></span>
        <span className="choice-action">Empezar<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M4 12h15m-6-6 6 6-6 6"/></svg></span>
      </button>
      <button className="server-choice import-choice" onClick={onImport} aria-labelledby="import-choice-title" aria-describedby="import-choice-description">
        <span className="choice-crown" aria-hidden="true"><svg viewBox="0 0 68 18" fill="none"><path d="M0 9h24l10-8 10 8h24M24 9l10 8 10-8"/></svg></span>
        <ChoiceEmblem kind="import"/>
        <span className="choice-copy"><strong id="import-choice-title">Importar servidor</strong><span id="import-choice-description">Continúa con tus archivos actuales.<br/>Cópialos o usa su carpeta.</span></span>
        <span className="choice-action">Elegir carpeta<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M4 12h15m-6-6 6 6-6 6"/></svg></span>
      </button>
    </div>
  </section>
}
