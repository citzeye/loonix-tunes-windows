$ErrorActionPreference = "Stop"

$appName = "LoonixTunesWin64v2"
$dist = ".\dist"
$vendorDist = "$dist\vendor"
$output = "..\Output"
$ffmpegMinimal = "C:\dev\ffmpeg-8.1-minimal\bin"
$stripTool = "C:\msys64\ucrt64\bin\strip.exe"
$targetDebug = "..\target\debug"

Write-Host "--- BUILDING CLEAN MASTER ZIP (NO DUPLICATES) ---" -ForegroundColor Cyan

# [0] Kill running processes
Get-Process $appName -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

# [1] Clean and create structure
if (Test-Path $dist) { Remove-Item -Path $dist -Recurse -Force }
New-Item -Path $vendorDist -ItemType Directory -Force | Out-Null

# [2] Build release with size optimization
cargo build --release

# [3] Copy Core Binary as app-core.bin (hidden from users in vendor)
Write-Host "Moving binary as app-core.bin..."
Copy-Item "..\target\release\$appName.exe" "$vendorDist\app-core.bin"

# [4] Copy Surgical FFmpeg DLLs to vendor/ and target\debug
Write-Host "Copying surgical FFmpeg DLLs..." -ForegroundColor Cyan
$surgicalDlls = @("avcodec-62.dll", "avformat-62.dll", "avutil-60.dll", "swresample-6.dll", "avfilter-11.dll")
foreach ($dll in $surgicalDlls) {
    Copy-Item "$ffmpegMinimal\$dll" "$vendorDist\" -Force
    Copy-Item "$ffmpegMinimal\$dll" "$targetDebug\" -Force
    Write-Host "  [OK] Copied $dll to vendor/ and target\debug" -ForegroundColor Green
}

# Verify avfilter-11.dll exists (fix crash)
if (!(Test-Path "$vendorDist\avfilter-11.dll")) { throw "ERROR: avfilter-11.dll missing from vendor!" }
if (!(Test-Path "$targetDebug\avfilter-11.dll")) { throw "ERROR: avfilter-11.dll missing from target\debug!" }
Write-Host "  [OK] avfilter-11.dll verified in both locations" -ForegroundColor Green

# Also copy Qt dependencies if they exist
if (Test-Path "..\vendor\qt") { Copy-Item "..\vendor\qt\*" "$vendorDist" -Recurse -Force }
if (Test-Path "..\qml") { Copy-Item "..\qml" "$vendorDist\qml" -Recurse -Force }

# [4.5] DIET MODE: Operasi Lemak Qt
Write-Host "--- STARTING EXTREME DIET ---" -ForegroundColor Yellow

# 1. Buang ICU (English only, so this is safe)
$icuFiles = Get-ChildItem "$vendorDist\icu*.dll" -ErrorAction SilentlyContinue
if ($icuFiles) {
    $icuSize = ($icuFiles | Measure-Object Length -Sum).Sum
    $icuFiles | Remove-Item -Force
    Write-Host "  [OK] Deleted ICU DLLs ($([math]::Round($icuSize/1MB,2)) MB)" -ForegroundColor Green
}

# 2. Buang Translations
if (Test-Path "$vendorDist\translations") {
    $transSize = (Get-ChildItem "$vendorDist\translations" -Recurse | Measure-Object Length -Sum).Sum
    Remove-Item "$vendorDist\translations" -Recurse -Force
    Write-Host "  [OK] Deleted translations/ ($([math]::Round($transSize/1MB,2)) MB)" -ForegroundColor Green
}

# 3. Trim QML Styles (KEEP: Fusion, Basic, impl - DELETE others)
$stylesToRemove = @("FluentWinUI3", "Imagine", "Material", "Universal", "Windows")
foreach ($style in $stylesToRemove) {
    $stylePath = "$vendorDist\qml\QtQuick\Controls\$style"
    if (Test-Path $stylePath) {
        $size = (Get-ChildItem $stylePath -Recurse -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
        Remove-Item $stylePath -Recurse -Force
        Write-Host "  [OK] Deleted Style: $style ($([math]::Round($size/1MB,2)) MB)" -ForegroundColor Green
    }
}

# 3.5 Remove qmodernwindowsstyle.dll (unused Qt style)
if (Test-Path "$vendorDist\styles\qmodernwindowsstyle.dll") {
    Remove-Item "$vendorDist\styles\qmodernwindowsstyle.dll" -Force
    Write-Host "  [OK] Deleted qmodernwindowsstyle.dll" -ForegroundColor Green
}

# 4. Buang SQL Drivers
if (Test-Path "$vendorDist\plugins\sqldrivers") {
    Remove-Item "$vendorDist\plugins\sqldrivers" -Recurse -Force
    Write-Host "  [OK] Deleted sqldrivers/" -ForegroundColor Green
}

# 5. Delete opengl32sw.dll (software OpenGL, not needed)
if (Test-Path "$vendorDist\opengl32sw.dll") {
    Remove-Item "$vendorDist\opengl32sw.dll" -Force
    Write-Host "  [OK] Deleted opengl32sw.dll" -ForegroundColor Green
}

# 6. Delete non-surgical FFmpeg DLLs (swscale-9.dll, avdevice-62.dll)
foreach ($dll in @("swscale-9.dll", "avdevice-62.dll")) {
    if (Test-Path "$vendorDist\$dll") {
        Remove-Item "$vendorDist\$dll" -Force
        Write-Host "  [OK] Deleted $dll (not in surgical build)" -ForegroundColor Green
    }
}

# 7. Delete libclang.dll if present (95MB build-time only)
$libclang = Get-ChildItem "$vendorDist\libclang.dll" -ErrorAction SilentlyContinue
if ($libclang) {
    $libclang | Remove-Item -Force
    Write-Host "  [OK] Deleted libclang.dll (build-time only)" -ForegroundColor Green
}

# Verify Fusion & Basic preserved
if (Test-Path "$vendorDist\qml\QtQuick\Controls\Fusion") {
    Write-Host "  [OK] Fusion style preserved" -ForegroundColor Green
}
if (Test-Path "$vendorDist\qml\QtQuick\Controls\Basic") {
    Write-Host "  [OK] Basic style preserved (safety net)" -ForegroundColor Green
}

Write-Host "--- DIET COMPLETE ---" -ForegroundColor Cyan

# [5] Strip all DLLs in vendor using MSYS2 strip tool
if (Test-Path $stripTool) {
    Write-Host "Stripping DLLs with $stripTool..." -ForegroundColor Cyan
    Get-ChildItem "$vendorDist\*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
        & $stripTool -s $_.FullName
        Write-Host "  [OK] Stripped: $($_.Name)" -ForegroundColor Green
    }
} else {
    Write-Host "WARNING: strip tool not found at $stripTool, skipping strip" -ForegroundColor Yellow
}

# [5.5] Copy README-FIRST.txt to dist root
Copy-Item "..\packaging\README-FIRST.txt" "$dist\README-FIRST.txt"

# [6] Inno Setup (packages from dist/vendor/)
& "C:\dev\Inno Setup 6\ISCC.exe" "loonix-tunes-setup.iss"
if ($LASTEXITCODE -ne 0) { throw "ERROR: Inno Setup compilation failed!" }

# [7] Get Installer & Put in root as LoonixTunesWin64v2.exe
$setupFile = Get-ChildItem "$output\*.exe" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if ($setupFile) {
    Copy-Item $setupFile.FullName "$dist\$appName.exe"
} else {
    throw "ERROR: Installer .exe not found in $output"
}

# [8] BuildPortable.ps1 (User-Facing - created in dist/)
$portableScript = @"
# BuildPortable.ps1 - Creates portable version from vendor folder
`$scriptDir = Split-Path -Parent `$MyInvocation.MyCommand.Path
`$target = Join-Path `$scriptDir "$($appName)_Portable"

Write-Host "Setting up Portable..." -ForegroundColor Cyan

# Create portable directory
if (!(Test-Path `$target)) {
    New-Item -ItemType Directory -Path `$target -Force | Out-Null
}

# Copy all files from vendor/ to portable folder
Copy-Item "`$scriptDir\vendor\*" `$target -Recurse -Force

# Rename app-core.bin back to .exe so it can be used
Rename-Item "`$target\app-core.bin" "$appName.exe" -Force

Write-Host "Done! You can run $appName.exe inside: `$target" -ForegroundColor Green
Pause
"@
$portableScript | Out-File -FilePath "$dist\BuildPortable.ps1" -Encoding utf8

# [9] ZIP everything in dist/ to Master ZIP
Compress-Archive -Path "$dist\*" -DestinationPath ".\$appName.zip" -Force

# [10] VERIFICATION: List all DLLs with sizes in MB
Write-Host "--- FINAL VERIFICATION: DLLs in dist\vendor ---" -ForegroundColor Cyan
Get-ChildItem "$vendorDist\*.dll" | ForEach-Object {
    $sizeMB = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  $($_.Name): $sizeMB MB" -ForegroundColor Green
}
$totalSize = (Get-ChildItem "$vendorDist\*.dll" | Measure-Object Length -Sum).Sum
Write-Host "  TOTAL DLL size: $([math]::Round($totalSize/1MB,2)) MB" -ForegroundColor Yellow

Write-Host "DONE! Master ZIP: $appName.zip" -ForegroundColor Green
