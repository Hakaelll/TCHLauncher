import {t} from './locale.ts'
import type {CrashPolicy} from './api'

export const defaultCrashPolicy:CrashPolicy={enabled:false,delaySeconds:30,maxAttempts:3}
export function sameCrashPolicy(a:CrashPolicy,b:CrashPolicy){
  return a.enabled===b.enabled&&a.delaySeconds===b.delaySeconds&&a.maxAttempts===b.maxAttempts
}
export function crashPolicyErrors(policy:CrashPolicy){
  return {
    delaySeconds:Number.isInteger(policy.delaySeconds)&&policy.delaySeconds>=5&&policy.delaySeconds<=3600?null:t("Elige una espera entera entre 5 y 3600 segundos."),
    maxAttempts:Number.isInteger(policy.maxAttempts)&&policy.maxAttempts>=1&&policy.maxAttempts<=10?null:t('Elige entre 1 y 10 intentos.'),
  }
}
// Reconcile observed policy changes without overwriting an unsaved local edit.
export function reconcileCrashDraft(draft:CrashPolicy,previous:CrashPolicy|null,observed:CrashPolicy){
  return !previous||sameCrashPolicy(draft,previous)?{...observed}:draft
}
