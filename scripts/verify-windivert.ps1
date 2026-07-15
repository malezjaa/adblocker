[CmdletBinding()]
param(
  [string]$DriverRoot = (Join-Path (Split-Path -Parent $PSScriptRoot) "third_party\windivert\x64")
)

$ErrorActionPreference = "Stop"
$checksumsPath = Join-Path $DriverRoot "SHA256SUMS.txt"

function Assert-File([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "Required WinDivert build input is missing: $Path"
  }
}

function Assert-Checksum([string]$Name) {
  $expected = Get-Content -LiteralPath $checksumsPath |
    Where-Object { $_ -match "\s\*?$([regex]::Escape($Name))$" } |
    Select-Object -First 1
  if (-not $expected) { throw "No pinned SHA-256 entry exists for $Name" }

  $expectedHash = ($expected -split "\s+")[0].ToLowerInvariant()
  $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $DriverRoot $Name)).Hash.ToLowerInvariant()
  if ($actualHash -ne $expectedHash) { throw "SHA-256 mismatch for $Name" }
}

function Find-SignTool() {
  $candidate = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($candidate) { return $candidate.Source }

  $candidate = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Filter signtool.exe -Recurse |
    Sort-Object FullName -Descending |
    Select-Object -First 1
  if (-not $candidate) {
    throw "signtool.exe is required to verify the Microsoft-signed WinDivert driver"
  }
  return $candidate.FullName
}

foreach ($file in @("WinDivert.dll", "WinDivert.lib", "WinDivert64.sys", "LICENSE.txt", "SHA256SUMS.txt")) {
  Assert-File (Join-Path $DriverRoot $file)
}
foreach ($file in @("WinDivert.dll", "WinDivert.lib", "WinDivert64.sys")) {
  Assert-Checksum $file
}

& (Find-SignTool) verify /pa /v (Join-Path $DriverRoot "WinDivert64.sys")
if ($LASTEXITCODE -ne 0) {
  throw "WinDivert64.sys does not have a valid Authenticode signature"
}

Write-Output "Verified WinDivert DLL, import library, driver, license, checksums, and driver signature."
