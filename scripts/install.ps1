# Cypher CLI Installer for Windows
# Run this script in PowerShell to build and install Cypher CLI

$ErrorActionPreference = "Stop"

Write-Host "[Cypher] CLI - Windows Installer" -ForegroundColor Blue
Write-Host "----------------------------------------"

# 1. Check if Rust/Cargo is installed
$CargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $CargoCheck) {
    Write-Host "Error: Rust / Cargo is not detected in your PATH." -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/ and try again."
    exit 1
}

Write-Host "[OK] Rust / Cargo detected." -ForegroundColor Green

# 2. Compile Cypher CLI
$InstallDir = Join-Path $HOME ".cypher\bin"

if ((Test-Path "Cargo.toml") -and (Get-Content "Cargo.toml" -Raw -ErrorAction SilentlyContinue | Select-String 'name = "cypher-cli"')) {
    Write-Host "Building directly from local source directory..." -ForegroundColor Blue
    cargo build --release
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }
    Copy-Item "target\release\cypher-cli.exe" -Destination (Join-Path $InstallDir "cypher.exe") -Force
} else {
    Write-Host "Cloning Cypher repository temporarily to build..." -ForegroundColor Blue
    $TempDir = Join-Path $env:TEMP "cypher-build"
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir
    }
    git clone --depth 1 https://github.com/sentinel-security/cypher-cli.git $TempDir
    
    $CurrentPath = Get-Location
    Set-Location $TempDir
    
    cargo build --release
    
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }
    Copy-Item "target\release\cypher-cli.exe" -Destination (Join-Path $InstallDir "cypher.exe") -Force
    
    Set-Location $CurrentPath
    Remove-Item -Recurse -Force $TempDir
}

Write-Host "[OK] Cypher CLI installed successfully as 'cypher'!" -ForegroundColor Green
Write-Host "----------------------------------------"

# 3. Add to User PATH if not already present
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to your environment PATH..." -ForegroundColor Blue
    $NewPath = $UserPath + ";" + $InstallDir
    # Remove any duplicate semicolons
    $NewPath = $NewPath -replace ';;+', ';'
    [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path += ";$InstallDir"
    Write-Host "[OK] Added to user PATH environment variable." -ForegroundColor Green
} else {
    Write-Host "[OK] $InstallDir is already in your PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "Please restart your terminal/PowerShell session for changes to take effect." -ForegroundColor Yellow
Write-Host "Then check the version by running:"
Write-Host "  cypher --version" -ForegroundColor Green
