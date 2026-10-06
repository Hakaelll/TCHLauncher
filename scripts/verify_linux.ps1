param([string]$OutputDirectory = 'artifacts/linux')
$ErrorActionPreference = 'Stop'
$tchWorkspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$tchOutput = [IO.Path]::GetFullPath((Join-Path $tchWorkspace $OutputDirectory))
$tchWorkspacePrefix = $tchWorkspace.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if (-not $tchOutput.StartsWith($tchWorkspacePrefix,[StringComparison]::OrdinalIgnoreCase)) {
  throw 'Choose an output directory inside the project workspace.'
}
$tchAncestor = $tchOutput
while ($tchAncestor -and $tchAncestor -ne $tchWorkspace) {
  if (Test-Path -LiteralPath $tchAncestor) {
    $tchItem = Get-Item -LiteralPath $tchAncestor -Force
    if (-not $tchItem.PSIsContainer -or ($tchItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
      throw 'The output path must use ordinary workspace directories, without files or junctions.'
    }
  }
  $tchAncestor = [IO.Path]::GetDirectoryName($tchAncestor)
}
if (Test-Path -LiteralPath $tchOutput) {
  if (@(Get-ChildItem -LiteralPath $tchOutput -Force).Count -gt 0) {
    throw 'The output directory is not empty. Choose a new directory to preserve previous verification artifacts.'
  }
}
Push-Location -LiteralPath $tchWorkspace
try {
  $tchDockerOs = & docker info --format '{{.OSType}}'
  if ($LASTEXITCODE -ne 0) { throw 'Docker is unavailable. Start Docker Desktop with its Linux engine and retry.' }
  if ($tchDockerOs.Trim() -ne 'linux') { throw 'This verification requires a Linux Docker engine.' }
  & docker build --progress plain --file scripts/linux/Dockerfile --target artifacts --output "type=local,dest=$tchOutput" .
  if ($LASTEXITCODE -ne 0) { throw 'Linux verification failed. No successful package is claimed.' }
  Write-Output "Linux packages and verification logs: $tchOutput"
} finally { Pop-Location }
