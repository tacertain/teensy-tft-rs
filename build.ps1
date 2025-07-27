# Build script with teensy_size validation
# This script builds the Rust project, validates it with teensy_size, and generates hex files

# Build both main and DMA versions
Write-Host "Building main Rust project..."
cargo build --release --bin main

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed: cargo build main failed"
    exit 1
}

Write-Host "Building DMA Rust project..."
cargo build --release --bin main_dma

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed: cargo build main_dma failed"
    exit 1
}

# Validate with teensy_size for both binaries
Write-Host "Validating main binary with teensy_size..."
$teensySize = Join-Path $env:USERPROFILE ".platformio\packages\tool-teensy\teensy_size.exe"
$rustTeensySize = "C:\Users\tacer\GitHub\teensy_size\rust-teensy-size.ps1"
$mainBinaryPath = ".\target\thumbv7em-none-eabihf\release\main"
$dmaBinaryPath = ".\target\thumbv7em-none-eabihf\release\main_dma"

Write-Host ""
Write-Host "=== MAIN BINARY ====" -ForegroundColor Cyan
Write-Host "teensy_size output:" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan

# Validate main binary
$mainTeensySizeSuccess = $false
# Use our custom Rust-aware teensy_size if available, otherwise fall back to original
if (Test-Path $rustTeensySize) {
    Write-Host "Using Rust-aware teensy_size..." -ForegroundColor Yellow
    
    try {
        & powershell -ExecutionPolicy Bypass -File $rustTeensySize $mainBinaryPath
        $mainTeensySizeSuccess = ($LASTEXITCODE -eq 0)
        
        if (-not $mainTeensySizeSuccess) {
            Write-Error "Memory validation failed: Main program exceeds available memory"
        }
    }
    catch {
        Write-Warning "Rust-aware teensy_size failed: $_"
        $mainTeensySizeSuccess = $false
    }
} elseif (Test-Path $teensySize) {
    Write-Host "Using original teensy_size..." -ForegroundColor Yellow
    
    # Run teensy_size and capture output while preserving formatting
    $teensySizeResult = & $teensySize $mainBinaryPath 2>&1
    
    # Display output with proper line breaks
    $teensySizeResult | ForEach-Object {
        Write-Host $_ -ForegroundColor Green
    }
    
    $mainTeensySizeSuccess = ($LASTEXITCODE -eq 0)
} else {
    Write-Warning "No teensy_size tool found"
    $mainTeensySizeSuccess = $false
}

Write-Host "==================" -ForegroundColor Cyan
Write-Host ""

# Validate DMA binary
Write-Host "=== DMA BINARY ====" -ForegroundColor Cyan
Write-Host "teensy_size output:" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan

$dmaTeensySizeSuccess = $false
if (Test-Path $rustTeensySize) {
    Write-Host "Using Rust-aware teensy_size..." -ForegroundColor Yellow
    
    try {
        & powershell -ExecutionPolicy Bypass -File $rustTeensySize $dmaBinaryPath
        $dmaTeensySizeSuccess = ($LASTEXITCODE -eq 0)
        
        if (-not $dmaTeensySizeSuccess) {
            Write-Error "Memory validation failed: DMA program exceeds available memory"
        }
    }
    catch {
        Write-Warning "Rust-aware teensy_size failed: $_"
        $dmaTeensySizeSuccess = $false
    }
} elseif (Test-Path $teensySize) {
    Write-Host "Using original teensy_size..." -ForegroundColor Yellow
    
    # Run teensy_size and capture output while preserving formatting
    $teensySizeResult = & $teensySize $dmaBinaryPath 2>&1
    
    # Display output with proper line breaks
    $teensySizeResult | ForEach-Object {
        Write-Host $_ -ForegroundColor Green
    }
    
    $dmaTeensySizeSuccess = ($LASTEXITCODE -eq 0)
} else {
    Write-Warning "No teensy_size tool found"
    $dmaTeensySizeSuccess = $false
}

Write-Host "==================" -ForegroundColor Cyan
Write-Host ""

if ($mainTeensySizeSuccess -and $dmaTeensySizeSuccess) {
    Write-Host "Both binary validations successful!" -ForegroundColor Green
} elseif ($mainTeensySizeSuccess -or $dmaTeensySizeSuccess) {
    Write-Warning "Some teensy_size validations failed, but binaries may still be valid"
} else {
    Write-Warning "teensy_size validation failed for both binaries, but they may still be valid"
}

# Generate hex files for both binaries
Write-Host "Generating main hex file..."
cargo objcopy --release --bin main -- -O ihex target/main.hex >$null 2>&1
$mainHexSuccess = ($LASTEXITCODE -eq 0)

Write-Host "Generating DMA hex file..."
cargo objcopy --release --bin main_dma -- -O ihex target/main_dma.hex >$null 2>&1
$dmaHexSuccess = ($LASTEXITCODE -eq 0)

if ($mainHexSuccess -and $dmaHexSuccess) {
    Write-Host "Build completed successfully!" -ForegroundColor Green
    Write-Host "Hex files created:" -ForegroundColor Green
    Write-Host "  - target/main.hex (Regular version)" -ForegroundColor Cyan
    Write-Host "  - target/main_dma.hex (DMA version)" -ForegroundColor Cyan
} elseif ($mainHexSuccess) {
    Write-Host "Main hex file created: target/main.hex" -ForegroundColor Green
    Write-Error "Failed to generate DMA hex file"
    exit 1
} elseif ($dmaHexSuccess) {
    Write-Host "DMA hex file created: target/main_dma.hex" -ForegroundColor Green
    Write-Error "Failed to generate main hex file"
    exit 1
} else {
    Write-Error "Failed to generate both hex files"
    exit 1
}
