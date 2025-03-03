#!/usr/bin/env pwsh
# VSCode Workspaces Editor GUI installer script for Windows

# Configuration
$AppName = "vscode-workspaces-editor-gui"
$Version = "1.0.0"
$GitHubRepo = "vhqtvn/vscode-workspaces-editor"
$InstallDir = "$env:LOCALAPPDATA\Programs\VSCodeWorkspacesEditor"
$DesktopShortcut = "$env:USERPROFILE\Desktop\VSCode Workspaces Editor.lnk"
$StartMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\VSCode Workspaces Editor"

# Check architecture
$Architecture = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
if ($Architecture -eq "ARM64") {
    Write-Host "Error: ARM architecture is not yet supported." -ForegroundColor Red
    Write-Host "Currently only x64 (64-bit) Windows is supported. ARM support is planned for future releases." -ForegroundColor Red
    exit 1
}

# Create necessary directories
Write-Host "Creating installation directories..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $StartMenuDir | Out-Null

# Download the latest release
try {
    Write-Host "Downloading the latest GUI release..." -ForegroundColor Cyan
    $ReleasesUri = "https://api.github.com/repos/$GitHubRepo/releases/latest"
    $LatestRelease = Invoke-RestMethod -Uri $ReleasesUri -Method Get
    $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*windows*x64*.msi" }
    
    if (-not $Asset) {
        Write-Host "No Windows installer found in the latest release. Please check the repository or try manual installation." -ForegroundColor Red
        exit 1
    }
    
    $InstallerPath = "$env:TEMP\vscode-workspaces-editor-gui-installer.msi"
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