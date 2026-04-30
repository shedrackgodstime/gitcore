# gity - Windows Uninstall Script

Write-Host "`n[*] Uninstalling gity..." -ForegroundColor Cyan

$Paths = @("C:\Program Files\gity\gity.exe", "$env:LOCALAPPDATA\gity\gity.exe")
$Removed = $false

foreach ($path in $Paths) {
    if (Test-Path $path) {
        Remove-Item $path -Force
        Write-Host "[+] Removed $path" -ForegroundColor Green
        $Removed = $true
        $parent = Split-Path $path
        if ((Get-ChildItem $parent -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) {
            Remove-Item $parent -Force
        }
    }
}

if (-not $Removed) {
    Write-Host "[-] gity not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[+] Done" -ForegroundColor Green
Write-Host "SSH keys and config were left untouched" -ForegroundColor Gray
Write-Host ""