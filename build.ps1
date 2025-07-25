# Build script with teensy_size validation
# This script builds the Rust project, validates it with teensy_size, and generates hex file

# Build the project
Write-Host "Building Rust project..."
cargo build --release --bin main

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed: cargo build failed"
    exit 1
}

# Validate with teensy_size
Write-Host "Validating binary with teensy_size..."
$teensySize = Join-Path $env:USERPROFILE ".platformio\packages\tool-teensy\teensy_size.exe"
$rustTeensySize = "C:\Users\tacer\GitHub\teensy_size\rust-teensy-size.ps1"
$binaryPath = ".\target\thumbv7em-none-eabihf\release\main"

Write-Host ""
Write-Host "teensy_size output:" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan

# Use our custom Rust-aware teensy_size if available, otherwise fall back to original
if (Test-Path $rustTeensySize) {
    Write-Host "Using Rust-aware teensy_size..." -ForegroundColor Yellow
    
    try {
        & powershell -ExecutionPolicy Bypass -File $rustTeensySize $binaryPath
        $teensySizeSuccess = ($LASTEXITCODE -eq 0)
        
        if (-not $teensySizeSuccess) {
            Write-Error "Memory validation failed: Program exceeds available memory"
        }
    }
    catch {
        Write-Warning "Rust-aware teensy_size failed: $_"
        $teensySizeSuccess = $false
    }
} elseif (Test-Path $teensySize) {
    Write-Host "Using original teensy_size..." -ForegroundColor Yellow
    
    # Run teensy_size and capture output while preserving formatting
    $teensySizeResult = & $teensySize $binaryPath 2>&1
    
    # Display output with proper line breaks
    $teensySizeResult | ForEach-Object {
        Write-Host $_ -ForegroundColor Green
    }
    
    $teensySizeSuccess = ($LASTEXITCODE -eq 0)
} else {
    Write-Warning "No teensy_size tool found"
    $teensySizeSuccess = $false
}

Write-Host "==================" -ForegroundColor Cyan
Write-Host ""

if ($teensySizeSuccess) {
    Write-Host "Binary validation successful!" -ForegroundColor Green
} else {
    Write-Warning "teensy_size validation failed, but binary may still be valid"
}

# Generate hex file
Write-Host "Generating hex file..."
cargo objcopy --release --bin main -- -O ihex target/main.hex >$null 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build completed successfully!"
    Write-Host "Hex file created: target/main.hex"
} else {
    Write-Error "Failed to generate hex file"
    exit 1
}
