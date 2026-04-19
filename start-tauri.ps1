# LLM Wiki Tauri Startup Script
# Set required environment variables

# Add protoc, MSVC toolchain and Cargo to PATH
$env:PATH = "D:\environment\c++\protoc\bin;C:\Users\Administrator\.cargo\bin;D:\environment\c++\product\VC\Tools\MSVC\14.50.35717\bin\Hostx64\x64;D:\environment\c++\product\Windows Kits\10\bin\10.0.26100.0\x64;" + $env:PATH

# Set LIB environment variable (library paths for linker)
$env:LIB = "D:\environment\c++\product\VC\Tools\MSVC\14.50.35717\lib\x64;D:\environment\c++\product\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64;D:\environment\c++\product\Windows Kits\10\Lib\10.0.26100.0\um\x64"

# Set INCLUDE environment variable (header file paths for compiler)
$env:INCLUDE = "D:\environment\c++\product\VC\Tools\MSVC\14.50.35717\include;D:\environment\c++\product\Windows Kits\10\Include\10.0.26100.0\ucrt;D:\environment\c++\product\Windows Kits\10\Include\10.0.26100.0\um;D:\environment\c++\product\Windows Kits\10\Include\10.0.26100.0\shared"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  LLM Wiki - Tauri Dev Server" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Verify environment
Write-Host "Verifying environment..." -ForegroundColor Yellow

$protocOutput = & protoc --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] protoc: $protocOutput" -ForegroundColor Green
} else {
    Write-Host "[ERROR] protoc not found" -ForegroundColor Red
    exit 1
}

$cargoOutput = & cargo --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] cargo: $cargoOutput" -ForegroundColor Green
} else {
    Write-Host "[ERROR] cargo not found" -ForegroundColor Red
    exit 1
}

$rustcOutput = & rustc --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] rustc: $rustcOutput" -ForegroundColor Green
} else {
    Write-Host "[ERROR] rustc not found" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Environment ready! Starting dev server..." -ForegroundColor Green
Write-Host ""

# Navigate to project directory
Set-Location $PSScriptRoot

# Start Tauri dev server
npm run tauri dev
