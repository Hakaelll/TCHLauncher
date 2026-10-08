export const fmtBytes = (n?: number | null, gbDigits=1) => n == null ? '—' : n < 1024 ** 2 ? `${Math.round(n / 1024)} KB` : n < 1024 ** 3 ? `${(n / 1024 ** 2).toFixed(1)} MB` : `${(n / 1024 ** 3).toFixed(gbDigits)} GB`
export const errorText = (error: unknown) => error instanceof Error ? error.message : String(error)
