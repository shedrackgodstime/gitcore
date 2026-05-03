# gity - Windows Uninstall Script

Write-Host "`n[*] Uninstalling gity..." -ForegroundColor Cyan

$DestDir = Join-Path $HOME "AppData\Local\Programs\gity"
$ExePath = Join-Path $DestDir "gity.exe"

if (Test-Path $ExePath) {
    Remove-Item $ExePath -Force
    Write-Host "[+] Removed $ExePath" -ForegroundColor Green
    
    # Remove from User PATH
    $UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -like "*$DestDir*") {
        $NewPath = $UserPath -replace [regex]::Escape(";$DestDir"), "" -replace [regex]::Escape($DestDir), ""
        [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        Write-Host "[+] Removed $DestDir from PATH" -ForegroundColor Green
    }

    # Optional: remove directory if empty
    if ((Get-ChildItem $DestDir).Count -eq 0) {
        Remove-Item $DestDir -Force
        Write-Host "[+] Removed $DestDir" -ForegroundColor Green
    }
    
    Write-Host "`n[+] gity has been uninstalled." -ForegroundColor Green
} else {
    Write-Host "[-] gity.exe not found in $DestDir" -ForegroundColor Yellow
}

Write-Host "`nSSH keys and gity configuration were left untouched for safety."
Write-Host "To remove them manually:"
Write-Host "  Remove-Item -Recurse -Force `$HOME\.config\gity"
Write-Host "  Remove-Item `$HOME\.ssh\id_ed25519_* (Careful!)"
Write-Host ""
