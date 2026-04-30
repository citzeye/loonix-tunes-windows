$ErrorActionPreference = "Stop"

Write-Host "=========================================="
Write-Host "Loonix Tunes - Production Deploy"
Write-Host "=========================================="

$dist = "dist"
$vendorQt = "..\vendor\qt"
$vendorBin = "..\vendor\bin"

Write-Host "[1/4] Cleaning up dist folder..."
if (Test-Path $dist) { Remove-Item -Path $dist -Recurse -Force }
New-Item -Path $dist -ItemType Directory -Force | Out-Null

Write-Host "[2/4] Building release..."
cargo build --release

Write-Host "[3/4] Copying binary and dependencies..."
Copy-Item "..\target\release\LoonixTunesWin64v2.exe" "$dist\LoonixTunesWin64v2.exe"
Copy-Item "$vendorBin\*.dll" $dist

Write-Host "[4/4] Copying Qt Runtime (from vendor/qt)..."
Copy-Item "$vendorQt\*" $dist -Recurse -Force

Write-Host "=========================================="
Write-Host "Building Portable ZIP..."
$zipPath = "LoonixTunesWin64v2.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$dist\*" -DestinationPath $zipPath -Force
Write-Host " -> Portable ZIP created at $zipPath"

Write-Host "=========================================="
Write-Host "Building installer..."
$ISCC = "C:\dev\Inno Setup 6\ISCC.exe"
if (!(Test-Path $ISCC)) { throw "ISCC not found at $ISCC" }
& $ISCC "loonix-tunes-setup.iss"
if ($LASTEXITCODE -ne 0) { throw "ERROR: Inno Setup compilation failed!" }
Write-Host " -> Installer created!"

Write-Host "=========================================="
Write-Host "DONE. Output: ZIP & Installer"
Write-Host "=========================================="
