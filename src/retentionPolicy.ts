import {t} from './locale.ts'
import type {BackupPolicy} from './api'

export function retentionPolicyErrors(policy:BackupPolicy){
  return {
    maxCount:policy.maxCount===null||(Number.isInteger(policy.maxCount)&&policy.maxCount>=1&&policy.maxCount<=1000)?null:t("Elige entre 1 y 1000 copias o deja el campo vacío."),
    maxBytes:policy.maxBytes===null||(Number.isSafeInteger(policy.maxBytes)&&policy.maxBytes>=1024**2&&policy.maxBytes<=2048*1024**3)?null:t("Elige entre 1 MB y 2048 GB o deja el campo vacío."),
    limits:policy.enabled&&policy.maxCount===null&&policy.maxBytes===null?t("Activa al menos un límite para eliminar copias automáticamente."):null,
  }
}
export function sameRetentionPolicy(a:BackupPolicy,b:BackupPolicy){
  return a.enabled===b.enabled&&a.maxCount===b.maxCount&&a.maxBytes===b.maxBytes
}
