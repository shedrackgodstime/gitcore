# gity - Windows Installation Script
param(
    [switch]$Help
)

if ($Help -or $args.Contains("help") -or $args.Contains("-h")) {
    Write-Host "gity installer - Manage multiple Git accounts safely" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  iwr https://shedrackgodstime.github.io/gity/ps | iex"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  # Install gity"
    Write-Host "  iwr https://shedrackgodstime.github.io/gity/ps | iex"
    exit
}

$ErrorActionPreference = "Stop"

Write-Host "`n[*] Installing gity - Git Account Manager..." -ForegroundColor Cyan
Write-Host "--------------------------------------------------" -ForegroundColor Blue

$Arch = $Env:PROCESSOR_ARCHITECTURE
if ($Arch -eq "AMD64") {
    $Target = "x86_64-pc-windows-msvc"
} else {
    Write-Error "[-] Unsupported Architecture for Windows: $Arch (only x64 supported)"
}

$AssetName = "gity-${Target}.tar.gz"
$ReleaseUrl = "https://api.github.com/repos/shedrackgodstime/gity/releases/latest"

Write-Host "[*] Fetching latest release..."
$Response = Invoke-RestMethod $ReleaseUrl
$DownloadUrl = $Response.assets | Where-Object { $_.name -like "*${Target}*" } | Select-Object -First 1 -ExpandProperty browser_download_url

if (-not $DownloadUrl) {
    Write-Error "[-] Could not find asset: $AssetName in $ReleaseUrl"
}

$TempDir = Join-Path $env:TEMP "gity-install-$((Get-Random))"
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

Write-Host "[+] Downloading $AssetName..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile (Join-Path $TempDir "gity.tar.gz") -UseBasicParsing

Write-Host "[*] Unpacking..."
tar -xzf (Join-Path $TempDir "gity.tar.gz") -C $TempDir

$DestDir = Join-Path $HOME "AppData\Local\Programs\gity"
if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
}

Copy-Item (Join-Path $TempDir "gity.exe") $DestDir

# Update PATH safely
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$DestDir*") {
    if ([string]::IsNullOrWhiteSpace($UserPath)) {
        $NewPath = $DestDir
    } else {
        $NewPath = "$UserPath;$DestDir"
    }
    [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path += ";$DestDir"
}

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
Write-Host " * Uninstall: iwr https://shedrackgodstime.github.io/gity/uninstall-ps | iex" -ForegroundColor Gray
Write-Host ""
