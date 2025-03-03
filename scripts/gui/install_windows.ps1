#!/usr/bin/env pwsh
# VSCode Workspaces Editor GUI installer script for Windows

# Configuration
$AppName = "vscode-workspaces-editor-gui"
$Version = "1.0.0"
$GitHubRepo = "vhqtvn/vscode-workspaces-editor"
$InstallDir = "$env:LOCALAPPDATA\Programs\VSCodeWorkspacesEditor"
$StartMenuFolder = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\VSCode Workspaces Editor"

# Create necessary directories
Write-Host "Creating installation directories..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $StartMenuFolder | Out-Null

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
    
    # More precise asset selection
    $AssetPattern = "vscode-workspaces-editor-gui-windows-$ArchName"
    $Asset = $LatestRelease.assets | Where-Object { $_.name -like "*$AssetPattern*.exe" -or $_.name -like "*$AssetPattern*.msi" } | Select-Object -First 1
    
    if (-not $Asset) {
        Write-Host "No Windows GUI installer found for $ArchName in the latest release. Please check the repository or try manual installation." -ForegroundColor Red
        exit 1
    }
    
    $InstallerPath = "$env:TEMP\vscode-workspaces-editor-gui-installer.exe"
    Write-Host "Downloading from: $($Asset.browser_download_url)" -ForegroundColor Cyan
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $InstallerPath
    
    # Run the installer
    Write-Host "Running the installer..." -ForegroundColor Cyan
    if ($Asset.name -like "*.msi") {
        Start-Process -FilePath "msiexec.exe" -ArgumentList "/i", "`"$InstallerPath`"", "/qb" -Wait
    } else {
        Start-Process -FilePath $InstallerPath -ArgumentList "/S", "/D=$InstallDir" -Wait
    }
    
    # Create shortcuts
    Write-Host "Creating shortcuts..." -ForegroundColor Cyan
    $WshShell = New-Object -ComObject WScript.Shell
    $ExePath = "$InstallDir\$AppName.exe"
    if (-not (Test-Path $ExePath)) {
        # Try to find the executable in the installation directory
        $ExePath = Get-ChildItem -Path $InstallDir -Filter "*.exe" -Recurse | Select-Object -First 1 -ExpandProperty FullName
    }
    
    if (-not $ExePath) {
        Write-Host "Warning: Could not find the application executable. Shortcuts won't be created." -ForegroundColor Yellow
    } else {
        $Shortcut = $WshShell.CreateShortcut("$StartMenuFolder\VSCode Workspaces Editor.lnk")
        $Shortcut.TargetPath = $ExePath
        $Shortcut.Save()
        
        $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\VSCode Workspaces Editor.lnk")
        $Shortcut.TargetPath = $ExePath
        $Shortcut.Save()
    }
    
    # Clean up
    Remove-Item -Path $InstallerPath -Force
    
    Write-Host "Installation completed successfully!" -ForegroundColor Green
    Write-Host "You can find 'VSCode Workspaces Editor' in your Start Menu and on your Desktop." -ForegroundColor Green
} catch {
    Write-Host "An error occurred during installation:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
} 