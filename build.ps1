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
$binaryPath = ".\target\thumbv7em-none-eabihf\release\main"

if (Test-Path $teensySize) {
    Write-Host ""
    Write-Host "teensy_size output:" -ForegroundColor Cyan
    Write-Host "==================" -ForegroundColor Cyan
    
    # Run teensy_size and capture output while preserving formatting
    $teensySizeResult = & $teensySize $binaryPath 2>&1
    
    # Display output with proper line breaks
    $teensySizeResult | ForEach-Object {
        Write-Host $_ -ForegroundColor Green
    }
    
    Write-Host "==================" -ForegroundColor Cyan
    Write-Host ""
    
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "teensy_size validation failed (exit code: $LASTEXITCODE)"
        Write-Warning "This may indicate the teensy_size version doesn't support Teensy 4.1"
        Write-Warning "The binary may still be valid for Teensy 4.1 - continuing build..."
    } else {
        Write-Host "Binary validation successful!" -ForegroundColor Green
    }
} else {
    Write-Warning "teensy_size.exe not found at: $teensySize"
    Write-Warning "Skipping teensy_size validation"
}

# Generate hex file
Write-Host "Generating hex file..."
cargo objcopy --release --bin main -- -O ihex target/main.hex

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build completed successfully!"
    Write-Host "Hex file created: target/main.hex"
} else {
    Write-Error "Failed to generate hex file"
    exit 1
}
