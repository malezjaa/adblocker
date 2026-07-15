[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
  [Parameter(Mandatory = $true)]
  [string]$Target,
  [string]$OutputDirectory = "release"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$stageRoot = Join-Path $projectRoot "target\release-package\vox-$Version-$Target"
$driverRoot = Join-Path $projectRoot "third_party\windivert\x64"
$releaseDirectory = Join-Path $projectRoot $OutputDirectory

function Assert-File([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "Required release file is missing: $Path"
  }
}

& (Join-Path $PSScriptRoot "verify-windivert.ps1") -DriverRoot $driverRoot

Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $stageRoot
New-Item -ItemType Directory -Force $stageRoot | Out-Null

$binRoot = Join-Path $projectRoot "target\$Target\release"
foreach ($file in @("daemon.exe", "cli.exe", "vox_windows_client.exe")) {
  Assert-File (Join-Path $binRoot $file)
  Copy-Item -LiteralPath (Join-Path $binRoot $file) -Destination $stageRoot
}
Copy-Item -LiteralPath (Join-Path $driverRoot "WinDivert.dll") -Destination $stageRoot
Copy-Item -LiteralPath (Join-Path $driverRoot "WinDivert64.sys") -Destination $stageRoot
Copy-Item -LiteralPath (Join-Path $driverRoot "LICENSE.txt") -Destination (Join-Path $stageRoot "WIN_DIVERT_LICENSE.txt")
Copy-Item -LiteralPath (Join-Path $projectRoot "docs\windows-release.md") -Destination (Join-Path $stageRoot "README-Windows.md")
Copy-Item -LiteralPath (Join-Path $projectRoot "third_party\windivert\NOTICE.txt") -Destination (Join-Path $stageRoot "THIRD_PARTY_NOTICES.txt")

$files = Get-ChildItem -LiteralPath $stageRoot -File | Sort-Object Name
$checksums = $files | ForEach-Object {
  "{0} *{1}" -f (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant(), $_.Name
}
Set-Content -LiteralPath (Join-Path $stageRoot "SHA256SUMS.txt") -Value ($checksums -join [Environment]::NewLine) -NoNewline

New-Item -ItemType Directory -Force $releaseDirectory | Out-Null
$zipPath = Join-Path $releaseDirectory "vox-$Version-$Target.zip"
Remove-Item -Force -ErrorAction SilentlyContinue $zipPath
Add-Type -AssemblyName System.IO.Compression.FileSystem
$stream = [System.IO.File]::Open($zipPath, [System.IO.FileMode]::CreateNew)
try {
  $zip = [System.IO.Compression.ZipArchive]::new($stream, [System.IO.Compression.ZipArchiveMode]::Create)
  try {
    foreach ($file in (Get-ChildItem -LiteralPath $stageRoot -File | Sort-Object Name)) {
      $entry = $zip.CreateEntry($file.Name, [System.IO.Compression.CompressionLevel]::Optimal)
      $entry.LastWriteTime = [DateTimeOffset]::new(1980, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
      $entryStream = $entry.Open()
      try {
        $sourceStream = [System.IO.File]::OpenRead($file.FullName)
        try { $sourceStream.CopyTo($entryStream) } finally { $sourceStream.Dispose() }
      } finally { $entryStream.Dispose() }
    }
  } finally { $zip.Dispose() }
} finally { $stream.Dispose() }

Copy-Item -LiteralPath (Join-Path $stageRoot "SHA256SUMS.txt") -Destination (Join-Path $releaseDirectory "SHA256SUMS.txt")
Write-Output "Created $zipPath"
