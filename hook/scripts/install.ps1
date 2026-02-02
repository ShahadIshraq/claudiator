# Claudiator Hook Installer for Windows
#Requires -Version 5.1

# Banner
Write-Host "================================" -ForegroundColor Cyan
Write-Host "  Claudiator Hook Installer" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

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
$Platform = "windows"

# Set variables
$InstallDir = "$env:USERPROFILE\.claude\claudiator"
$BinaryName = "claudiator-hook.exe"
$Repo = "shahadishraq/claudiator"
$DownloadUrl = "https://github.com/${Repo}/releases/latest/download/claudiator-hook-${Target}.zip"
$ZipPath = "$env:TEMP\claudiator-hook.zip"

# Create install directory
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Download
Write-Host "Downloading claudiator-hook for ${Target}..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to download binary" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Extract
try {
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to extract archive" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Clean up zip
Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue

# Prompt for configuration
Write-Host ""
$ServerUrl = Read-Host "Server URL"
$SecureApiKey = Read-Host "API Key" -AsSecureString
$BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($SecureApiKey)
$ApiKey = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
[System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($BSTR)

# Set device info
$DeviceName = $env:COMPUTERNAME
$DeviceId = [System.Guid]::NewGuid().ToString()

# Write config.toml
$ConfigContent = @"
server_url = "$ServerUrl"
api_key = "$ApiKey"
device_name = "$DeviceName"
device_id = "$DeviceId"
platform = "$Platform"
"@

Set-Content -Path "$InstallDir\config.toml" -Value $ConfigContent -Encoding UTF8

# Test connection
Write-Host ""
Write-Host "Testing connection..." -ForegroundColor Yellow
try {
    & "$InstallDir\$BinaryName" test
    Write-Host "Connection test successful!" -ForegroundColor Green
} catch {
    Write-Host "Warning: Connection test failed. You can re-run: $InstallDir\$BinaryName test" -ForegroundColor Yellow
}

# Ask about hooks configuration
Write-Host ""
$ConfigureHooks = Read-Host "Auto-configure Claude Code hooks in ~/.claude/settings.json? [Y/n]"
if ([string]::IsNullOrWhiteSpace($ConfigureHooks)) {
    $ConfigureHooks = "Y"
}

$HooksConfigured = $false

if ($ConfigureHooks -match "^[Yy]$") {
    $SettingsFile = "$env:USERPROFILE\.claude\settings.json"
    $HookCommand = "~/.claude/claudiator/claudiator-hook send"
    $Events = @("SessionStart", "SessionEnd", "Stop", "Notification", "UserPromptSubmit")

    # Create settings directory if it doesn't exist
    $SettingsDir = Split-Path -Parent $SettingsFile
    if (-not (Test-Path $SettingsDir)) {
        New-Item -ItemType Directory -Force -Path $SettingsDir | Out-Null
    }

    # Load or create settings
    if (Test-Path $SettingsFile) {
        try {
            $Settings = Get-Content $SettingsFile -Raw | ConvertFrom-Json
        } catch {
            Write-Host "Warning: Could not parse existing settings.json, creating new one" -ForegroundColor Yellow
            $Settings = [PSCustomObject]@{}
        }
    } else {
        $Settings = [PSCustomObject]@{}
    }

    # Ensure hooks property exists
    if (-not ($Settings.PSObject.Properties.Name -contains "hooks")) {
        $Settings | Add-Member -NotePropertyName "hooks" -NotePropertyValue ([PSCustomObject]@{}) -Force
    }

    # Add hooks for each event
    foreach ($Event in $Events) {
        # Ensure event array exists
        if (-not ($Settings.hooks.PSObject.Properties.Name -contains $Event)) {
            $Settings.hooks | Add-Member -NotePropertyName $Event -NotePropertyValue @() -Force
        }

        # Check if hook already exists
        $Existing = $Settings.hooks.$Event | Where-Object { $_.command -eq $HookCommand }

        if (-not $Existing) {
            # Add the hook
            $NewHook = [PSCustomObject]@{
                matcher = ""
                command = $HookCommand
            }
            $Settings.hooks.$Event += $NewHook
        }
    }

    # Write back
    try {
        $Settings | ConvertTo-Json -Depth 10 | Set-Content -Path $SettingsFile -Encoding UTF8
        $HooksConfigured = $true
    } catch {
        Write-Host "Error: Failed to write settings.json" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
}

# Print summary
Write-Host ""
Write-Host "================================" -ForegroundColor Green
Write-Host "  Installation Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host "  ✓ Binary installed to: $InstallDir\$BinaryName"
Write-Host "  ✓ Config written to: $InstallDir\config.toml"
if ($HooksConfigured) {
    Write-Host "  ✓ Claude Code hooks configured in ~/.claude/settings.json"
}
Write-Host ""
Write-Host "  To test: $InstallDir\$BinaryName test"
Write-Host "  To uninstall: Remove-Item -Recurse -Force $InstallDir"
Write-Host "================================" -ForegroundColor Green
Write-Host ""
