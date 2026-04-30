$ErrorActionPreference = "Stop"

Write-Host "=========================================="
Write-Host "Loonix Tunes - Production Deploy"
Write-Host "=========================================="

Write-Host "[1/8] Building release..."
cargo build --release

Write-Host "[2/8] Recreate dist folder..."
$dist = "dist"
if (Test-Path $dist) {
    Remove-Item $dist -Recurse -Force
}
New-Item -ItemType Directory -Path $dist | Out-Null

Write-Host "[3/8] Copy binary..."
Copy-Item "..\..\target\release\loonix-tunes.exe" $dist

Write-Host "[4/8] Copy QML source files..."
Copy-Item -Path "..\..\qml" -Destination "$dist\qml" -Recurse -Force

Write-Host "[5/8] Deploying Qt runtime..."
$WindeployQt = "C:\dev\6.8.3\msvc2022_64\bin\windeployqt.exe"
if (!(Test-Path $WindeployQt)) {
    throw "windeployqt not found at $WindeployQt"
}
& $WindeployQt --release --qmldir "..\..\qml" "$dist\loonix-tunes.exe"

Write-Host "[6/8] Copy FFmpeg runtime..."
Copy-Item "C:\dev\ffmpeg\bin\av*.dll" $dist
Copy-Item "C:\dev\ffmpeg\bin\sw*.dll" $dist

Write-Host "[7/8] Build Portable ZIP..."
$zipPath = "loonix-tunes-portable.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}
Compress-Archive -Path "$dist\*" -DestinationPath $zipPath -Force
Write-Host " -> Portable ZIP created at $zipPath"

Write-Host "[8/8] Build installer..."
$ISCC = "C:\dev\Inno Setup 6\ISCC.exe"
if (!(Test-Path $ISCC)) {
    throw "ISCC not found at $ISCC"
}
& $ISCC "loonix-tunes-setup.iss"
Write-Host " -> Installer created!"

Write-Host "=========================================="
Write-Host "DONE. Output: Portable ZIP & Installer"
Write-Host "=========================================="