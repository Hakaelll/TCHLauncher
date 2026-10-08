import {test} from 'node:test'
import assert from 'node:assert/strict'
import {english} from '../src/locales/en.ts'
import {getLanguage,initializeLanguage,languageKey,languageSaved,localizedStatic,readLanguage,setLanguage,subscribeLanguage,t,translatedMessage,writeLanguage,formatNumber,formatDate} from '../src/locale.ts'

const storage=()=>{const data=new Map();return {getItem:key=>data.get(key)??null,setItem:(key,value)=>data.set(key,value)}}
test('language survives reload, invalid storage falls back and storage failure leaves the choice usable',()=>{
  const store=storage(),root={lang:''};initializeLanguage(store,root);assert.equal(getLanguage(),'es');assert.equal(root.lang,'es')
  let notifications=0;const unsubscribe=subscribeLanguage(()=>notifications++)
  setLanguage('en',store,root);assert.equal(store.getItem(languageKey),'en');assert.equal(root.lang,'en');assert.equal(languageSaved(),true)
  initializeLanguage(store,root);assert.equal(getLanguage(),'en')
  setLanguage('es',null,root);assert.equal(languageSaved(),false);assert.equal(getLanguage(),'es');assert.equal(root.lang,'es');assert.equal(notifications,2);unsubscribe()
  assert.equal(readLanguage({getItem:()=>'{bad json}'}),'es');assert.equal(readLanguage({getItem:()=>{throw Error('blocked')}}),'es')
  assert.equal(writeLanguage(null,'en'),false)
})
test('language switches translate static metadata without changing IDs, inputs or interpolated user data',()=>{
  const metadata=localizedStatic([{id:'worlds',label:'Mundos',options:[{value:'survival',label:'Supervivencia'}]}])
  const input={name:'Mundos',command:'/say Hola',path:'C:\\Users\\Name\\world(1)'}
  setLanguage('en',null);assert.equal(metadata[0].label,'Worlds');assert.equal(metadata[0].options[0].label,'Survival');assert.equal(metadata[0].id,'worlds');assert.equal(metadata[0].options[0].value,'survival')
  assert.equal(t('Se eliminó «{0}» y sus archivos locales.',input.name),'Deleted “Mundos” and its local files.')
  assert.equal(t('Exportación guardada en {0}',input.path),'Export saved at '+input.path)
  assert.equal(translatedMessage('[Server thread/INFO]: Hola desde Mundos'),'[Server thread/INFO]: Hola desde Mundos')
  assert.equal(input.name,'Mundos');assert.equal(input.command,'/say Hola')
  setLanguage('es',null);assert.equal(metadata[0].label,'Mundos');assert.equal(metadata[0].options[0].label,'Supervivencia')
})
test('stored notices and native messages translate at display time and unknown diagnostics remain verbatim',()=>{
  const path='C:\\Mods\\test.jar';setLanguage('en',null)
  assert.equal(t('No se pudo abrir Java (code 123)'), 'No se pudo abrir Java (code 123)')
  assert.equal(t('La suma de comprobación de '+path+' no coincide'),'Checksum mismatch for '+path)
  const notice=t('Se importó «{0}».','My world');assert.equal(notice,'Imported “My world”.')
  setLanguage('es',null);assert.equal(t(notice),'Se importó «My world».');assert.equal(t('Cancelar'),'Cancelar')
  setLanguage('en',null);assert.equal(t(' Guardando… '),' Saving… ');assert.equal(formatNumber(1234.5),'1,234.5')
  setLanguage('es',null);assert.equal(formatNumber(1234.5),'1234,5')
})
test('all translations preserve placeholder counts and core user flows have English copy',()=>{
  for(const [key,value] of Object.entries(english)) {
    const parameters=text=>[...text.matchAll(/\{(\d+)\}/g)].map(m=>m[1]).sort()
    assert.deepEqual(parameters(value),parameters(key),key)
  }
  setLanguage('en',null)
  for(const key of ['Mundos','Nuevo servidor','Jugadores','Consola','Ajustes','Explorar Modrinth','Idioma de la aplicación','Mundo y mods','Guardar ZIP','Descargando','Secciones del servidor'])assert.notEqual(t(key),key,key)
  setLanguage('es',null)
})

test('date formatting follows the selected language for Date, timestamp and stored ISO values',()=>{
  const date=new Date('2026-10-08T12:34:56Z'),options={timeZone:'UTC'}
  for(const [language,locale] of [['en','en-US'],['es','es-ES']]) {
    setLanguage(language,null)
    const expected=date.toLocaleString(locale,options)
    for(const value of [date,date.getTime(),date.toISOString()])assert.equal(formatDate(value,options),expected)
  }
  setLanguage('es',null)
})
