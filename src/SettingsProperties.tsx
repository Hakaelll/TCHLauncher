import {settingsMatch} from './settingsSearch'
import {translatedMessage,localizedStatic,t} from './locale.ts'
type SettingField = {key:string;label:string;description:string;kind:'text'|'number'|'toggle'|'choice';min?:number;max?:number;options?:{value:string;label:string}[]}
const settingsGroups: {title:string;description:string;fields:SettingField[]}[] = localizedStatic([
  {title:'Identidad y conexión',description:'Lo que verán los jugadores y cómo se conectarán.',fields:[
    {key:'motd',label:'Mensaje del servidor',description:'Aparece en la lista de servidores de Minecraft.',kind:'text'},
    {key:'server-port',label:'Puerto',description:'Puerto TCP usado para las conexiones.',kind:'number',min:1,max:65535},
    {key:'online-mode',label:'Verificación de cuentas',description:'Comprueba las cuentas con los servicios oficiales de Minecraft.',kind:'toggle'},
  ]},
  {title:'Juego',description:'Reglas y distancia que se aplican al mundo.',fields:[
    {key:'gamemode',label:'Modo de juego',description:'Modo inicial de los jugadores nuevos.',kind:'choice',options:[{value:'survival',label:'Supervivencia'},{value:'creative',label:'Creativo'},{value:'adventure',label:'Aventura'},{value:'spectator',label:'Espectador'}]},
    {key:'difficulty',label:'Dificultad',description:'Afecta al combate y a las criaturas.',kind:'choice',options:[{value:'peaceful',label:'Pacífica'},{value:'easy',label:'Fácil'},{value:'normal',label:'Normal'},{value:'hard',label:'Difícil'}]},
    {key:'pvp',label:'Combate entre jugadores',description:'Permite que los jugadores se dañen entre sí.',kind:'toggle'},
    {key:'force-gamemode',label:'Aplicar modo al entrar',description:'Restablece el modo de juego configurado cuando un jugador vuelve a entrar.',kind:'toggle'},
    {key:'hardcore',label:'Hardcore',description:'Tras morir, el jugador pasa a espectador. Los mundos existentes pueden conservar sus propias reglas.',kind:'toggle'},
    {key:'allow-flight',label:'Permitir vuelo',description:'Evita la expulsión automática por volar; no concede la capacidad de volar.',kind:'toggle'},
    {key:'enable-command-block',label:'Bloques de comandos',description:'Permite ejecutar los comandos de estos bloques.',kind:'toggle'},
    {key:'spawn-protection',label:'Protección del spawn',description:'Radio en bloques protegido frente a jugadores sin permisos de operador. 0 lo desactiva.',kind:'number',min:0,max:2147483647},
  ]},
  {title:'Distancias y límites',description:'Ajusta el alcance de la simulación y los límites de la sesión.',fields:[
    {key:'view-distance',label:'Distancia de visión',description:'Chunks que el servidor envía a cada jugador.',kind:'number',min:2,max:32},
    {key:'simulation-distance',label:'Distancia de simulación',description:'Chunks donde el mundo sigue activo.',kind:'number',min:2,max:32},
    {key:'max-world-size',label:'Radio máximo del mundo',description:'Límite en bloques usado por la generación. No sustituye el borde del mundo.',kind:'number',min:1,max:29999984},
    {key:'player-idle-timeout',label:'Tiempo de inactividad',description:'Minutos antes de expulsar a un jugador inactivo. 0 lo desactiva.',kind:'number',min:0,max:2147483647},
  ]},
  {title:'Generación y dimensiones',description:'La semilla y el generador se usan al crear un mundo; cambiarlos no regenera ni borra el mundo existente.',fields:[
    {key:'level-seed',label:'Semilla',description:'Número o texto para un mundo nuevo. Déjalo vacío para una semilla aleatoria.',kind:'text'},
    {key:'level-type',label:'Generador del mundo',description:'Identificador del generador de esta versión, por ejemplo minecraft:normal. Conserva el identificador de tus mods si usas uno.',kind:'text'},
    {key:'generate-structures',label:'Generar estructuras',description:'Permite la generación de estructuras como aldeas.',kind:'toggle'},
    {key:'allow-nether',label:'Acceso al Nether',description:'Permite viajar a esta dimensión si el servidor admite esta propiedad.',kind:'toggle'},
    {key:'spawn-animals',label:'Aparición de animales',description:'Permite la aparición de animales en versiones que admiten esta propiedad.',kind:'toggle'},
    {key:'spawn-monsters',label:'Aparición de monstruos',description:'Permite la aparición de monstruos en versiones que admiten esta propiedad.',kind:'toggle'},
    {key:'spawn-npcs',label:'Aparición de aldeanos',description:'Permite la aparición de aldeanos en versiones que admiten esta propiedad.',kind:'toggle'},
  ]},
  {title:'Acceso',description:'Quién puede entrar y cómo se aplica la lista blanca.',fields:[
    {key:'white-list',label:'Lista blanca',description:'Solo permite jugadores añadidos a la lista.',kind:'toggle'},
    {key:'enforce-whitelist',label:'Expulsar a no autorizados',description:'Saca a quien deje de estar autorizado.',kind:'toggle'},
  ]},
])
export const knownSettings = new Set(settingsGroups.flatMap(group => group.fields.map(field => field.key)))

export function invalidNumericSetting(values:Record<string,string>,savedValues:Record<string,string>){
  return settingsGroups.flatMap(group=>group.fields).find(field=>{
    const value=values[field.key]
    if(field.kind!=='number'||value===undefined||value===savedValues[field.key])return false
    const number=Number(value)
    return !/^\+?\d+$/.test(value)||!Number.isSafeInteger(number)||number<(field.min??0)||number>(field.max??Number.MAX_SAFE_INTEGER)
  })
}

export function SettingsPropertyGroups({values,onChange,version,query=''}: {values:Record<string,string>;onChange:(values:Record<string,string>)=>void;version:string;query?:string}) {
  const update = (key: string, value: string) => onChange({...values,[key]:value})
  const groups=settingsGroups.map(group=>({...group,fields:group.fields.filter(field=>Object.hasOwn(values,field.key)&&settingsMatch(query,group.title,field.key,field.label,field.description))})).filter(group=>group.fields.length)
  return <div className="settings-groups">{query.trim()&&!groups.length&&<p className="empty-inline" role="status">{t("No hay ajustes de juego que coincidan con la búsqueda.")}</p>}{groups.map(group=><section className="settings-group" key={group.title} aria-label={group.title}>
    <div className="settings-group-heading"><h4>{group.title}</h4><p>{translatedMessage(group.description)}</p><span className="settings-field-count">{group.fields.length}  {t("opciones disponibles")}</span></div>
    <div className="settings-fields">{group.fields.map(field=>{
      const value=values[field.key]
      const legacyMinor=version.startsWith('1.')?Number(version.split('.')[1]):null
      const options=field.options?.filter(option=>field.key!=='gamemode'||option.value!=='spectator'||legacyMinor===null||legacyMinor>=8)
      const boolean=field.kind==='toggle'&&['true','false'].includes(value)
      const unmatched=field.kind==='choice'&&!options?.some((option,index)=>option.value===value||['gamemode','difficulty'].includes(field.key)&&String(index)===value)
      return <label className="setting-line" key={field.key}><span className="setting-copy"><strong>{translatedMessage(field.label)}</strong><small>{translatedMessage(field.description)}</small></span>
        {boolean?<input className="setting-toggle" type="checkbox" checked={value==='true'} onChange={event=>update(field.key,String(event.target.checked))}/>:field.kind==='choice'?<select value={value} onChange={event=>update(field.key,event.target.value)}>{unmatched&&<option value={value}>{value || t("(vacío)")}  {t("· Valor actual")}</option>}{options?.map((option,index)=><option key={option.value} value={['gamemode','difficulty'].includes(field.key)&&/^[0-3]$/.test(value)?String(index):option.value}>{translatedMessage(option.label)}</option>)}</select>:<input data-setting-key={field.key} type={field.kind==='number'?'number':'text'} min={field.min} max={field.max} step={field.kind==='number'?1:undefined} value={value} onChange={event=>{event.currentTarget.setCustomValidity('');update(field.key,event.target.value)}}/>}
      </label>
    })}</div>
  </section>)}</div>
}

