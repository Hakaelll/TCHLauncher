import {t} from './locale.ts'

export function windowsFileAdvice(detail:string):string|null {
  switch(detail.trimEnd().match(/(?:^|\s)\(os error (\d+)\)$/)?.[1]){
    case '39':case '112':return t('Windows informó de falta de espacio. Comprueba el estado de la operación y libera espacio en la unidad de destino antes de reintentar.')
    case '5':return t('Windows denegó el acceso. Comprueba los permisos de la carpeta y revisa el estado de la operación antes de reintentar.')
    case '32':return t('Windows indica que otro programa usa el archivo. Cierra ese archivo o programa y comprueba el estado de la operación antes de reintentar.')
    default:return null
  }
}
