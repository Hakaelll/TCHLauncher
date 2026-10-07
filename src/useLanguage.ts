import {useSyncExternalStore} from 'react'
import {getLanguage,subscribeLanguage} from './locale.ts'
export function useLanguage(){return useSyncExternalStore(subscribeLanguage,getLanguage,getLanguage)}
