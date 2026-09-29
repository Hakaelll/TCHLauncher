import { useEffect, useRef, useState, type CSSProperties } from 'react'

type Props = {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  maxInput: number;
  step: number;
  unit?: string;
  onChange: (value: number) => void;
}

export function SliderField({id,label,value,min,max,maxInput,step,unit,onChange}: Props) {
  const [draft,setDraft] = useState(String(value))
  const editing = useRef(false)
  useEffect(() => { if (!editing.current) setDraft(String(value)) },[value])
  const commit = () => {
    editing.current = false
    const parsed = Number(draft)
    const next = draft !== '' && Number.isFinite(parsed) ? Math.min(maxInput,Math.max(min,Math.round(parsed))) : min
    setDraft(String(next))
    if (next !== value) onChange(next)
  }
  const visibleValue = Number.isFinite(value) ? Math.min(max,Math.max(min,value)) : min
  const fill = max > min ? Math.round((visibleValue - min) / (max - min) * 100) : 0
  return <div className="slider-field"><div className="slider-field-top"><label htmlFor={id}>{label}</label><div className={`slider-value ${unit ? 'has-unit' : ''}`}><input id={id} type="number" min={min} max={maxInput} step="1" required value={draft} onFocus={() => { editing.current = true }} onChange={event => { const next = event.target.value; setDraft(next); const parsed = Number(next); onChange(next === '' || !Number.isFinite(parsed) ? 0 : parsed) }} onBlur={commit}/>{unit && <span aria-hidden="true">{unit}</span>}</div></div><input type="range" aria-label={`Ajustar ${label.toLowerCase()}`} min={min} max={max} step={step} value={visibleValue} style={{'--slider-progress': `${fill}%`} as CSSProperties} onChange={event => { const next = Number(event.target.value); setDraft(String(next)); onChange(next) }}/><div className="slider-ends"><span>{min} {unit}</span><span>{max} {unit}</span></div></div>
}
