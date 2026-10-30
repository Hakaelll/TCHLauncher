[CmdletBinding(SupportsShouldProcess)]
param()

$ErrorActionPreference='Stop'
$root=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (!(Test-Path -LiteralPath (Join-Path $root 'src-tauri/Cargo.toml'))) { throw 'Run this script from a TCHLauncher checkout.' }
$processes=Get-CimInstance Win32_Process
$debugRoot=Join-Path $root 'src-tauri\target\debug\'
$active=$processes | Where-Object {
    $_.Name -in @('cargo.exe','rustc.exe','link.exe','cl.exe') -or
    ($_.Name -eq 'node.exe' -and $_.CommandLine -match 'node_modules[\\/].*(?:vite|typescript|tauri)') -or
    ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($debugRoot,[StringComparison]::OrdinalIgnoreCase))
}
if ($active -and !$WhatIfPreference) { throw 'Stop builds, Vite and the development application before cleaning.' }

$candidates=@(
    'src-tauri\target\debug', 'src-tauri\target\ui-artwork',
    'src-tauri\target\release\deps', 'src-tauri\target\release\build',
    'src-tauri\target\release\.fingerprint', 'src-tauri\target\release\incremental',
    'src-tauri\target\release\examples', 'src-tauri\target\release\tchlauncher.pdb',
    'src-tauri\target\release\tchlauncher.d', 'dist', 'node_modules\.vite', 'tsconfig.tsbuildinfo',
    'artifacts\java-properties\ecj-3.44.0.jar',
    'artifacts\fabric-predicates\fabric-loader-0.19.5.jar',
    'artifacts\fabric-predicates\UnicodeData-15.0.0.txt'
)
# Validate every candidate before deleting anything. Never follow redirected paths.
$targets=foreach ($relative in $candidates) {
    $path=Join-Path $root $relative
    if (!(Test-Path -LiteralPath $path)) { continue }
    $resolved=(Resolve-Path -LiteralPath $path).Path
    if (!$resolved.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Cleanup path escaped the checkout.' }
    $ancestor=$resolved
    for ($depth=0;$depth -lt 64;$depth++) {
        if (((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Redirected cleanup path: $relative" }
        if ($ancestor -eq $root) { break }
        $ancestor=[IO.Directory]::GetParent($ancestor).FullName
    }
    if ($ancestor -ne $root) { throw 'Checkout ancestry was not confirmed.' }
    $item=Get-Item -LiteralPath $resolved -Force
    $contents=if ($item.PSIsContainer) { @(Get-ChildItem -LiteralPath $resolved -Force -Recurse) } else { @($item) }
    if ($contents | Where-Object {($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0}) { throw "Redirected cache entry: $relative" }
    [pscustomobject]@{path=$resolved;relative=$relative;bytes=($contents | Where-Object {!$_.PSIsContainer} | Measure-Object Length -Sum).Sum}
}
$removedBytes=0L
$removed=@()
foreach ($target in $targets) {
    if ($PSCmdlet.ShouldProcess($target.path,'Remove generated cache files')) {
        Remove-Item -LiteralPath $target.path -Recurse -Force
        $removedBytes+=$target.bytes
        $removed+=$target.relative
    }
}
[pscustomobject]@{removedLogicalBytes=$removedBytes;removedPaths=$removed}
