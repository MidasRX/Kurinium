$Host.UI.RawUI.WindowTitle = "Kurinium Builder"

Write-Host ""
Write-Host "KURINIUM BUILDER" -ForegroundColor Cyan
Write-Host ""

Write-Host "[1] Discord Bot Token" -ForegroundColor Yellow
Write-Host "    Get this from: https://discord.com/developers/applications" -ForegroundColor Gray
$token = Read-Host "    Enter Bot Token"

if ([string]::IsNullOrWhiteSpace($token)) {
    Write-Host "ERROR: Token cannot be empty!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[2] Discord Server (Guild) ID" -ForegroundColor Yellow
Write-Host "    Right-click your server -> Copy Server ID" -ForegroundColor Gray
$guildId = Read-Host "    Enter Guild ID"

if ([string]::IsNullOrWhiteSpace($guildId)) {
    Write-Host "ERROR: Guild ID cannot be empty!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[3] Show Console Output? (for debugging)" -ForegroundColor Yellow
$showConsole = Read-Host "    Enable console? (y/N)"
$consoleValue = if ($showConsole -eq 'y' -or $showConsole -eq 'Y') { "true" } else { "false" }

Write-Host ""
Write-Host "Updating configuration..." -ForegroundColor Cyan

$configPath = Join-Path $PSScriptRoot "src\config.rs"
$configContent = Get-Content $configPath -Raw
$configContent = $configContent -replace 'pub const GUILD_ID: u64 = \d+;', "pub const GUILD_ID: u64 = $guildId;"

if ($consoleValue -eq "true") {
    $configContent = $configContent -replace 'pub const SHOW_CONSOLE: bool = false;', 'pub const SHOW_CONSOLE: bool = true;'
}
else {
    $configContent = $configContent -replace 'pub const SHOW_CONSOLE: bool = true;', 'pub const SHOW_CONSOLE: bool = false;'
}

Set-Content $configPath -Value $configContent -NoNewline

Write-Host "Configuration updated!" -ForegroundColor Green
Write-Host ""
Write-Host "Building release..." -ForegroundColor Cyan
Write-Host "This may take a minute..." -ForegroundColor Gray
Write-Host ""

$env:KURINIUM_TOKEN = $token
Set-Location $PSScriptRoot
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "BUILD SUCCESSFUL!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your agent is ready at:" -ForegroundColor White
    Write-Host "  .\target\release\kurinium.exe" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Configuration:" -ForegroundColor White
    Write-Host "  Guild ID:    $guildId" -ForegroundColor Gray
    Write-Host "  Console:     $consoleValue" -ForegroundColor Gray
    Write-Host "  Protection:  $(if ($protectionEnabled) { 'Enabled' } else { 'Disabled' })" -ForegroundColor Gray
}
else {
    Write-Host ""
    Write-Host "BUILD FAILED!" -ForegroundColor Red
}
