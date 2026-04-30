# gity - Windows Installation Script
param(
    [switch]$Help
)

if ($Help -or $args.Contains("help") -or $args.Contains("-h")) {
    Write-Host "gity installer - Manage multiple Git accounts safely" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  iwr gity.pages.dev/ps | iex"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  # Install gity"
    Write-Host "  iwr gity.pages.dev/ps | iex"
    exit
}

$ErrorActionPreference = "Stop"

Write-Host "`n[*] Installing gity - Git Account Manager..." -ForegroundColor Cyan
Write-Host "--------------------------------------------------" -ForegroundColor Blue

$Arch = $Env:PROCESSOR_ARCHITECTURE
if ($Arch -eq "AMD64") {
    $TargetArch = "x86_64"
} elseif ($Arch -eq "ARM64") {
    $TargetArch = "aarch64"
} else {
    Write-Error "[-] Unsupported Architecture: $Arch"
}

$AssetName = "gity-${TargetArch}-pc-windows-msvc.tar.gz"
$ReleaseUrl = "https://api.github.com/repos/kristency/gity/releases/latest"

Write-Host "[*] Fetching latest release..."
$Response = Invoke-RestMethod $ReleaseUrl
$DownloadUrl = $Response.assets | Where-Object { $_.name -eq $AssetName } | Select-Object -ExpandProperty browser_download_url

if (-not $DownloadUrl) {
    Write-Error "[-] Could not find asset: $AssetName"
}

$TempDir = Join-Path $env:TEMP "gity-install-$((Get-Random))"
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

Write-Host "[+] Downloading $AssetName..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile (Join-Path $TempDir "gity.tar.gz") -UseBasicParsing

Write-Host "[*] Unpacking..."
tar -xzf (Join-Path $TempDir "gity.tar.gz") -C $TempDir

$DestDir = "C:\Program Files\gity"
if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
}

Copy-Item (Join-Path $TempDir "gity.exe") $DestDir
$env:Path += ";$DestDir"
[System.Environment]::SetEnvironmentVariable("Path", $env:Path, "User")

Write-Host "[+] Installed gity to $DestDir" -ForegroundColor Green

Remove-Item $TempDir -Recurse -Force

Write-Host ""
Write-Host "[+] Success! gity installed" -ForegroundColor Green
Write-Host "--------------------------------------------------" -ForegroundColor Blue
Write-Host ""
Write-Host " * Add account:   gity add" -ForegroundColor White
Write-Host " * List accounts: gity list" -ForegroundColor White
Write-Host " * Clone repo:    gity clone" -ForegroundColor White
Write-Host " * Security:      gity audit" -ForegroundColor White
Write-Host " * Help:          gity --help" -ForegroundColor White
Write-Host ""
Write-Host " * Restart your terminal to use gity from anywhere" -ForegroundColor Yellow
Write-Host ""
Write-Host " * Uninstall: iwr gity.pages.dev/uninstall-ps | iex" -ForegroundColor Gray
Write-Host ""