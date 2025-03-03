#!/usr/bin/env pwsh
# VSCode Workspaces Editor CLI installer script for Windows

# Configuration
$AppName = "vscode-workspaces-editor"
$Version = "1.0.0"
$GitHubRepo = "vhqtvn/vscode-workspaces-editor"
$InstallDir = "$env:LOCALAPPDATA\Programs\VSCodeWorkspacesEditor-CLI"
$BinDir = "$InstallDir\bin"

# Create necessary directories
Write-Host "Creating installation directories..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# Determine system architecture
$Architecture = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
if ($Architecture -eq "AMD64") {
    $ArchName = "amd64"
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
    Write-Host "Downloading the latest CLI release..." -ForegroundColor Cyan
    $ReleasesUri = "https://api.github.com/repos/$GitHubRepo/releases/latest"
    $LatestRelease = Invoke-RestMethod -Uri $ReleasesUri -Method Get
    $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*vscode-workspaces-editor-windows-$ArchName.exe" }
    
    if (-not $Asset) {
        Write-Host "No Windows CLI executable found for $ArchName in the latest release. Please check the repository or try manual installation." -ForegroundColor Red
        exit 1
    }
    
    $ExePath = "$BinDir\vscode-workspaces-editor.exe"
    Write-Host "Downloading from: $($Asset.browser_download_url)" -ForegroundColor Cyan
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ExePath
    
    # Add to PATH
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$BinDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$BinDir", "User")
        Write-Host "Added $BinDir to your PATH." -ForegroundColor Cyan
    }
    
    Write-Host "Installation completed successfully!" -ForegroundColor Green
    Write-Host "You can run 'vscode-workspaces-editor' from the command line." -ForegroundColor Green
    Write-Host "Note: You may need to restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
} catch {
    Write-Host "An error occurred during installation:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
} 