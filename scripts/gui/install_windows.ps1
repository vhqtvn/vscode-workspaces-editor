#!/usr/bin/env pwsh
# VSCode Workspaces Editor GUI installer script for Windows

# Configuration
$AppName = "vscode-workspaces-editor-gui"
$Version = "1.0.0"
$GitHubRepo = "vhqtvn/vscode-workspaces-editor"
$InstallDir = "$env:LOCALAPPDATA\Programs\VSCodeWorkspacesEditor"
$DesktopShortcut = "$env:USERPROFILE\Desktop\VSCode Workspaces Editor.lnk"
$StartMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\VSCode Workspaces Editor"

# Create necessary directories
Write-Host "Creating installation directories..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $StartMenuDir | Out-Null

# Determine system architecture
$Architecture = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
if ($Architecture -eq "AMD64") {
    $ArchName = "x64"
}
elseif ($Architecture -eq "ARM64") {
    $ArchName = "arm64"
}
else {
    Write-Host "Error: Unsupported architecture: $Architecture" -ForegroundColor Red
    Write-Host "Currently only x64 (AMD64) and ARM64 are supported." -ForegroundColor Red
    exit 1
}

# Download the latest release
try {
    Write-Host "Downloading the latest GUI release..." -ForegroundColor Cyan
    $ReleasesUri = "https://api.github.com/repos/$GitHubRepo/releases/latest"
    $LatestRelease = Invoke-RestMethod -Uri $ReleasesUri -Method Get
    $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*vscode-workspaces-editor-gui-windows-$ArchName.msi" }
    
    if (-not $Asset) {
        Write-Host "No Windows MSI installer found for $ArchName in the latest release. Please check the repository or try manual installation." -ForegroundColor Red
        exit 1
    }
    
    $InstallerPath = "$env:TEMP\vscode-workspaces-editor-gui-installer.msi"
    Write-Host "Downloading from: $($Asset.browser_download_url)" -ForegroundColor Cyan
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $InstallerPath
    
    # Run the installer
    Write-Host "Running installer..." -ForegroundColor Cyan
    Start-Process -FilePath "msiexec.exe" -ArgumentList "/i", $InstallerPath, "/quiet" -Wait
    
    Write-Host "Installation completed successfully!" -ForegroundColor Green
    Write-Host "You can find VSCode Workspaces Editor GUI in your Start Menu and on your Desktop." -ForegroundColor Green
    Write-Host "You can launch it by running 'vscode-workspaces-editor-gui' from the command line." -ForegroundColor Green
} catch {
    Write-Host "An error occurred during installation:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
} 