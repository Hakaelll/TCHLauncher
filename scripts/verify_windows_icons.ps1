param(
  [string]$Executable = 'src-tauri/target/release/tchlauncher.exe',
  [string]$Installer = 'src-tauri/target/release/bundle/nsis/TCHLauncher_0.15.0_x64-setup.exe',
  [string]$ExpectedIcon = 'src-tauri/icons/icon.ico'
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TchIconResource {
 [DllImport("shell32.dll", CharSet=CharSet.Unicode)] public static extern uint ExtractIconEx(string path, int index, IntPtr[] large, IntPtr[] small, uint count);
 [DllImport("user32.dll")] public static extern bool DestroyIcon(IntPtr icon);
}
'@
function Read-IconPixels([string]$Path) {
  $resolved = (Resolve-Path -LiteralPath $Path).Path
  $large = New-Object IntPtr[] 1
  $small = New-Object IntPtr[] 1
  if ([TchIconResource]::ExtractIconEx($resolved,0,$large,$small,1) -eq 0) { throw "No embedded icon: $resolved" }
  try {
    $pixels = [System.Collections.Generic.List[int]]::new()
    foreach ($handle in @($large[0],$small[0])) {
      $bitmap = [System.Drawing.Icon]::FromHandle($handle).ToBitmap()
      try {
        $pixels.Add($bitmap.Width); $pixels.Add($bitmap.Height)
        for ($y=0; $y -lt $bitmap.Height; $y++) {
          for ($x=0; $x -lt $bitmap.Width; $x++) { $pixels.Add($bitmap.GetPixel($x,$y).ToArgb()) }
        }
      } finally { $bitmap.Dispose() }
    }
    return ,$pixels.ToArray()
  } finally {
    [void][TchIconResource]::DestroyIcon($large[0])
    [void][TchIconResource]::DestroyIcon($small[0])
  }
}
$expected = Read-IconPixels $ExpectedIcon
foreach ($candidate in @($Executable,$Installer)) {
  $actual = Read-IconPixels $candidate
  if ($actual.Length -ne $expected.Length) { throw "Icon dimensions differ: $candidate" }
  for ($index=0; $index -lt $actual.Length; $index++) {
    if ($actual[$index] -ne $expected[$index]) { throw "Embedded icon differs from expected logo: $candidate" }
  }
  Write-Output "Embedded icon matches expected logo: $candidate"
}
