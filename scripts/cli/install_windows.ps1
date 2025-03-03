#!/usr/bin/env pwsh
# VSCode Workspaces Editor CLI installer script for Windows

# Configuration
$AppName = "vscode-workspaces-editor"
$Version = "1.0.0"
$GitHubRepo = "vhqtvn/vscode-workspaces-editor"
$InstallDir = "$env:LOCALAPPDATA\Programs\VSCodeWorkspacesEditor-CLI"
$BinDir = "$InstallDir\bin"

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
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# Download the latest release
try {
    Write-Host "Downloading the latest CLI release..." -ForegroundColor Cyan
    $ReleasesUri = "https://api.github.com/repos/$GitHubRepo/releases/latest"
    $LatestRelease = Invoke-RestMethod -Uri $ReleasesUri -Method Get
    $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*windows*x64*cli*.zip" -or $_.name -like "*cli*windows*x64*.zip" }
    
    if (-not $Asset) {
        # Try to find any Windows ZIP file as fallback
        $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*windows*x64*.zip" }
        
        if (-not $Asset) {
            Write-Host "No CLI installer found in the latest release. Please check the repository or try manual installation." -ForegroundColor Red
            exit 1
        }
    }
    
    $ZipPath = "$env:TEMP\vscode-workspaces-editor-cli.zip"
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath
    
    # Extract the ZIP file
    Write-Host "Extracting files..." -ForegroundColor Cyan
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
    
    # Find the CLI executable
    $CliExe = Get-ChildItem -Path $InstallDir -Recurse -Filter "vscode-workspaces-editor.exe" | Select-Object -First 1
    
    if (-not $CliExe) {
        Write-Host "CLI executable not found in the downloaded package." -ForegroundColor Red
        exit 1
    }
    
    # Copy the CLI executable to the bin directory
    Copy-Item -Path $CliExe.FullName -Destination "$BinDir\vscode-workspaces-editor.exe" -Force
    
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