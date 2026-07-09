# Cypher CLI Installer for Windows
# Run this script in PowerShell to install Cypher CLI

$ErrorActionPreference = "Stop"

Write-Host "[Cypher] CLI - Windows Installer" -ForegroundColor Blue
Write-Host "----------------------------------------"

$InstallDir = Join-Path $HOME ".cypher\bin"

# 1. Check if Rust/Cargo is installed (for building from source)
$CargoCheck = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $CargoCheck) {
    Write-Host "[INFO] Rust / Cargo not detected. Installing Rust via rustup..." -ForegroundColor Yellow
    $RustupInstaller = Join-Path $env:TEMP "rustup-init.exe"
    iwr -useb "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $RustupInstaller
    Start-Process -Wait -FilePath $RustupInstaller -ArgumentList "-y --default-toolchain stable --profile default"
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    $CargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $CargoCheck) {
        Write-Host "Error: Failed to install Rust." -ForegroundColor Red
        exit 1
    }
}

Write-Host "[OK] Rust / Cargo detected." -ForegroundColor Green

# 2. Build Cypher CLI
if ((Test-Path "Cargo.toml") -and (Get-Content "Cargo.toml" -Raw -ErrorAction SilentlyContinue | Select-String 'name = "cypher-cli"')) {
    Write-Host "Building from local source directory..." -ForegroundColor Blue
    cargo build --release
} else {
    Write-Host "Cloning Cypher repository..." -ForegroundColor Blue
    $TempDir = Join-Path $env:TEMP "cypher-build"
    if (Test-Path $TempDir) { Remove-Item -Recurse -Force $TempDir }
    git clone --depth 1 https://github.com/Ayan-Flash/Cypher.git $TempDir
    Set-Location $TempDir
    cargo build --release
    Set-Location $HOME
    Remove-Item -Recurse -Force $TempDir
}

if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null }
Copy-Item "target\release\cypher.exe" -Destination (Join-Path $InstallDir "cypher.exe") -Force

Write-Host "[OK] Cypher CLI installed as 'cypher.exe'!" -ForegroundColor Green

# 3. Add to User PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
    Write-Host "[OK] Added to PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "Done! Restart your terminal, then run:" -ForegroundColor Yellow
Write-Host "  cypher --version" -ForegroundColor Green
