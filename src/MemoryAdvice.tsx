import {memoryStartingPoint} from './capacity'
import {fmtBytes} from './format'
import {t} from './locale'

export function MemoryAdvice(props:Parameters<typeof memoryStartingPoint>[0]) {
  const suggested=memoryStartingPoint(props)
  return suggested===null?null:<p className="field-help">{t('RAM máxima orientativa para empezar: {0}. No es una medición; ajústala según el mundo y el contenido.',fmtBytes(suggested*1024**2))}</p>
}
