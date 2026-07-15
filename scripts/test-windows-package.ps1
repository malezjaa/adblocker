[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$Archive
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $Archive -PathType Leaf)) { throw "Package not found: $Archive" }

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $Archive))
try {
  $expected = @(
    "daemon.exe", "cli.exe", "vox_windows_client.exe", "WinDivert.dll", "WinDivert64.sys",
    "WIN_DIVERT_LICENSE.txt", "THIRD_PARTY_NOTICES.txt", "README-Windows.md", "SHA256SUMS.txt"
  )
  $actual = @($zip.Entries | ForEach-Object FullName | Sort-Object)
  $missing = $expected | Where-Object { $_ -notin $actual }
  if ($missing) { throw "Package is missing: $($missing -join ', ')" }
  if ($actual.Count -ne $expected.Count) { throw "Package contains unexpected files: $($actual -join ', ')" }
} finally {
  $zip.Dispose()
}
