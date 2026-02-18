# Claudiator Hook Updater for Windows
#Requires -Version 5.1

param(
    [switch]$Check
)

# Banner
Write-Host "================================" -ForegroundColor Cyan
Write-Host "  Claudiator Hook Updater" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Variables
$Repo = "shahadishraq/claudiator"
$InstallDir = "$env:USERPROFILE\.claude\claudiator"
$Binary = "$InstallDir\claudiator-hook.exe"

# Detect architecture
$Arch = $env:PROCESSOR_ARCHITECTURE
if ($Arch -eq "AMD64") {
    $ArchTarget = "x86_64"
} elseif ($Arch -eq "ARM64") {
    $ArchTarget = "aarch64"
} else {
    Write-Host "Error: Unsupported architecture: $Arch" -ForegroundColor Red
    exit 1
}

# Build target string
$Target = "${ArchTarget}-pc-windows-msvc"

# Verify binary exists
if (-not (Test-Path $Binary)) {
    Write-Host "Error: claudiator-hook not found at $Binary" -ForegroundColor Red
    Write-Host "Run install.ps1 first." -ForegroundColor Red
    exit 1
}

# Get current version
Write-Host "Checking current version..." -ForegroundColor Yellow
try {
    $VersionOutput = & $Binary version 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Binary exited with code $LASTEXITCODE"
    }
    if ($VersionOutput -match '(\d+\.\d+\.\d+)') {
        $CurrentVersion = $Matches[1]
    } else {
        throw "Could not parse version from output: $VersionOutput"
    }
} catch {
    Write-Host "Error: Could not determine current version" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
Write-Host "Current version: $CurrentVersion" -ForegroundColor White

# Query GitHub API for the latest hook-v* release
Write-Host "Checking for updates..." -ForegroundColor Yellow
try {
    $ReleasesJson = Invoke-RestMethod -Uri "https://api.github.com/repos/${Repo}/releases" -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to query GitHub releases API" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

$LatestRelease = $ReleasesJson | Where-Object { $_.tag_name -match '^hook-v' } | Select-Object -First 1
if (-not $LatestRelease) {
    Write-Host "Error: No hook-v* release found on GitHub" -ForegroundColor Red
    exit 1
}

$LatestTag = $LatestRelease.tag_name
if ($LatestTag -match 'hook-v(\d+\.\d+\.\d+)') {
    $LatestVersion = $Matches[1]
} else {
    Write-Host "Error: Could not parse version from tag: $LatestTag" -ForegroundColor Red
    exit 1
}
Write-Host "Latest version:  $LatestVersion" -ForegroundColor White

# Compare versions using [System.Version]
try {
    $CurrentSysVersion = [System.Version]$CurrentVersion
    $LatestSysVersion = [System.Version]$LatestVersion
} catch {
    Write-Host "Error: Could not parse version numbers for comparison" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

if ($CurrentSysVersion -ge $LatestSysVersion) {
    Write-Host ""
    Write-Host "Already up to date." -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "Update available: $CurrentVersion -> $LatestVersion" -ForegroundColor Yellow

# If -Check, exit after reporting
if ($Check) {
    exit 0
}

# Download new version
$DownloadUrl = "https://github.com/${Repo}/releases/download/${LatestTag}/claudiator-hook-${Target}.zip"
$ZipPath = "$env:TEMP\claudiator-hook-update.zip"
$BackupPath = "${Binary}.bak"

Write-Host ""
Write-Host "Downloading claudiator-hook ${LatestVersion} for ${Target}..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to download binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Backup current binary
Write-Host "Backing up current binary..." -ForegroundColor Yellow
try {
    Copy-Item -Path $Binary -Destination $BackupPath -Force -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to back up current binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue
    exit 1
}

# Extract and install new binary
Write-Host "Installing new binary..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to extract archive" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host "Restoring backup..." -ForegroundColor Yellow
    Copy-Item -Path $BackupPath -Destination $Binary -Force -ErrorAction SilentlyContinue
    Remove-Item $BackupPath -Force -ErrorAction SilentlyContinue
    Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue
    exit 1
}

# Verify new binary works
Write-Host "Verifying new binary..." -ForegroundColor Yellow
try {
    $NewVersionOutput = & $Binary version 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Binary exited with code $LASTEXITCODE"
    }
    if ($NewVersionOutput -notmatch '\d+\.\d+\.\d+') {
        throw "Could not parse version from output: $NewVersionOutput"
    }
} catch {
    Write-Host "Error: New binary verification failed" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host "Rolling back to previous version..." -ForegroundColor Yellow
    try {
        Copy-Item -Path $BackupPath -Destination $Binary -Force -ErrorAction Stop
        Write-Host "Rollback successful. Previous version restored." -ForegroundColor Yellow
    } catch {
        Write-Host "Error: Rollback failed. Manual restore may be needed from: $BackupPath" -ForegroundColor Red
    }
    Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue
    exit 1
}

# Clean up on success
Remove-Item $BackupPath -Force -ErrorAction SilentlyContinue
Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue

# Print success
Write-Host ""
Write-Host "================================" -ForegroundColor Green
Write-Host "  Update Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host "  Updated: $CurrentVersion -> $LatestVersion"
Write-Host "================================" -ForegroundColor Green
Write-Host ""
