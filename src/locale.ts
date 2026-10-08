import { english } from './locales/en.ts'

export type Language = 'es' | 'en'
export const languageKey = 'tch-language-v1'
type StorageLike = Pick<Storage, 'getItem' | 'setItem'>
export function readLanguage(storage: Pick<StorageLike,'getItem'> | null): Language {
  try { return storage?.getItem(languageKey)==='en'?'en':'es' } catch { return 'es' }
}
export function writeLanguage(storage: Pick<StorageLike,'setItem'> | null, value: Language): boolean {
  try { if(!storage)return false;storage.setItem(languageKey,value);return true } catch { return false }
}
const listeners=new Set<()=>void>()
let language:Language='es'
let saved=true
export function getLanguage(){return language}
export function languageSaved(){return saved}
export function subscribeLanguage(listener:()=>void){listeners.add(listener);return()=>{listeners.delete(listener)}}
export function initializeLanguage(storage:StorageLike|null,root?:HTMLElement){language=readLanguage(storage);if(root)root.lang=language}
export function setLanguage(value:Language,storage:StorageLike|null,root?:HTMLElement){
  language=value;saved=writeLanguage(storage,value);if(root)root.lang=value
  for(const listener of listeners)listener()
}
const reverse=new Map(Object.entries(english).map(([es,en])=>[en,es]))
const fill=(value:string,args:unknown[])=>value.replace(/\{(\d+)\}/g,(_,index:string)=>String(args[Number(index)]??`{${index}}`))
const escape=(value:string)=>value.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')
const pattern=(source:string,target:string)=>({target,prefix:source.split(/\{\d+\}/)[0],suffix:source.split(/\{\d+\}/).at(-1)!,indices:[...source.matchAll(/\{(\d+)\}/g)].map(match=>Number(match[1])),expression:new RegExp('^'+source.split(/\{\d+\}/).map(escape).join('(.*?)')+'$','s')})
const entries=Object.entries(english).filter(([es])=>/\{\d+\}/.test(es)).sort(([a],[b])=>b.replace(/\{\d+\}/g,'').length-a.replace(/\{\d+\}/g,'').length)
const patterns={en:entries.map(([es,en])=>pattern(es,en)),es:entries.map(([es,en])=>pattern(en,es))}
/** Only call for application copy. User names, paths and Minecraft logs stay verbatim. */
export function t(message:string,...args:unknown[]):string {
  if(!args.length&&!Object.hasOwn(english,message)&&!reverse.has(message)&&message.trim()!==message) {
    const trimmed=message.trim(),result=t(trimmed)
    if(result!==trimmed)return message.slice(0,message.indexOf(trimmed))+result+message.slice(message.indexOf(trimmed)+trimmed.length)
  }
  const key=Object.hasOwn(english,message)?message:reverse.get(message)??message
  if(args.length)return fill(language==='en'?english[key]??key:key,args)
  if(language==='es'&&key!==message)return key
  if(language==='en'&&english[key])return english[key]
  for(const pattern of patterns[language]){
    if(!key.startsWith(pattern.prefix)||!key.endsWith(pattern.suffix))continue
    const match=pattern.expression.exec(key)
    if(match){const values:unknown[]=[];pattern.indices.forEach((index,i)=>{values[index]=match[i+1]});return fill(pattern.target,values)}
  }
  return message
}
/** Lazy reads keep static UI metadata localized when the language changes. Never use on server data. */
export function localizedStatic<T extends object>(value:T):T {
  const cache=new WeakMap<object,object>()
  const wrap=(target:object):object=>{
    const existing=cache.get(target);if(existing)return existing
    const proxy=new Proxy(target,{get(object,key,receiver){const item=Reflect.get(object,key,receiver);return typeof item==='string'?(Object.hasOwn(english,item)||reverse.has(item)?t(item):item):item&&typeof item==='object'?wrap(item):item}})
    cache.set(target,proxy);return proxy
  }
  return wrap(value) as T
}
export function formatNumber(value:number,options?:Intl.NumberFormatOptions){return new Intl.NumberFormat(language==='en'?'en-US':'es-ES',options).format(value)}
export function formatDate(value:Date|string|number,options?:Intl.DateTimeFormatOptions){return new Date(value).toLocaleString(language==='en'?'en-US':'es-ES',options)}
export function translatedMessage<T>(value:T):T {return (typeof value==='string'?t(value):value) as T}

export function formatTime(value:Date|string|number,options?:Intl.DateTimeFormatOptions,locale=language){return new Date(value).toLocaleTimeString(locale==='en'?'en-US':'es-ES',options)}
export function formatLogTime(value:string|null,seconds=false,locale=language){
  if(!value)return '—'
  if(!value.includes('T'))return value
  const date=new Date(value)
  return Number.isNaN(date.getTime())?'—':formatTime(date,{hour:'2-digit',minute:'2-digit',...(seconds?{second:'2-digit'}:{})},locale)
}
