/* --- loonixtunesv2/build.rs | build --- */
fn main() {
    // --- WINDOWS BUILD ONLY ---
    #[cfg(windows)]
    {
        let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let vendor_lib = std::path::Path::new(&project_dir)
            .join("vendor")
            .join("lib");
        let vendor_bin = std::path::Path::new(&project_dir)
            .join("vendor")
            .join("bin");

        // 1. Tell linker to search for .lib files in vendor/lib
        println!("cargo:rustc-link-search=native={}", vendor_lib.display());

        // 2. Link libraries from vendor/lib
        println!("cargo:rustc-link-lib=rubberband");
        println!("cargo:rustc-link-lib=samplerate");
        println!("cargo:rustc-link-lib=sleef");
        println!("cargo:rustc-link-lib=sleefdft");
        println!("cargo:rustc-link-lib=avcodec");
        println!("cargo:rustc-link-lib=avdevice");
        println!("cargo:rustc-link-lib=avfilter");
        println!("cargo:rustc-link-lib=avformat");
        println!("cargo:rustc-link-lib=avutil");
        println!("cargo:rustc-link-lib=swresample");
        println!("cargo:rustc-link-lib=swscale");

        // 3. Copy DLLs from vendor/bin to target directory for cargo run
        copy_vendor_dlls(&vendor_bin, &project_dir);

        // Add Windows SDK include paths for ffmpeg-sys-next
        if let Some(sdk_path) = find_windows_sdk_include() {
            println!("cargo:rustc-cppflags=-I\"{}\"", sdk_path);
        }

        // Metadata & Icon Windows
        let icon_path = format!("{}/packaging/icon.ico", project_dir);
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = format!("{}/resource.rc", out_dir);
        let res_path = format!("{}/resource.res", out_dir);

        // Only create resource if icon exists
        if std::path::Path::new(&icon_path).exists() {
            // Copy icon to OUT_DIR so rc.exe can find it with relative path
            let icon_in_out = format!("{}/icon.ico", out_dir);
            if let Err(e) = std::fs::copy(&icon_path, &icon_in_out) {
                eprintln!("Warning: Failed to copy icon to OUT_DIR: {}", e);
            }

            // Write .rc file with relative icon path
            let rc_content = r#"
1 ICON "icon.ico"

1 VERSIONINFO
FILEVERSION     1,0,0,0
PRODUCTVERSION  1,0,0,0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904E4"
        BEGIN
            VALUE "CompanyName",      "citz"
            VALUE "FileDescription",  "Loonix-Tunes: High-Res Music Player for Windows"
            VALUE "FileVersion",      "1.0.0"
            VALUE "InternalName",     "loonix-tunes"
            VALUE "LegalCopyright",   "GPLv3"
            VALUE "OriginalFilename", "loonix-tunes.exe"
            VALUE "ProductName",      "Loonix-Tunes"
            VALUE "ProductVersion",   "1.0.0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1252
    END
END
"#;
            std::fs::write(&rc_path, rc_content).unwrap();

            // Find rc.exe from Windows SDK
            let rc_exe = find_rc_exe();
            let rc = std::process::Command::new(&rc_exe)
                .args(["/fo", &res_path, &rc_path])
                .output()
                .expect(&format!("Failed to run rc.exe at: {}", rc_exe));
            if !rc.status.success() {
                eprintln!("rc.exe stderr: {}", String::from_utf8_lossy(&rc.stderr));
                eprintln!("rc.exe stdout: {}", String::from_utf8_lossy(&rc.stdout));
            }

            // Link the .res file
            println!("cargo:rustc-link-arg={}", res_path);
        } else {
            eprintln!(
                "Warning: icon.ico not found at {}, skipping resource",
                icon_path
            );
        }
    }

    // Re-run if icon or resource file changes
    println!("cargo:rerun-if-changed=packaging/windows/icon.ico");
    println!("cargo:rerun-if-changed=vendor/bin");
}

// Copy DLLs from vendor/bin to target/{profile} for runtime
fn copy_vendor_dlls(vendor_bin: &std::path::Path, project_dir: &str) {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let dst = std::path::Path::new(project_dir)
        .join("target")
        .join(&profile);

    if let Ok(entries) = std::fs::read_dir(vendor_bin) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                let dest_file = dst.join(path.file_name().unwrap());
                if let Err(e) = std::fs::copy(&path, &dest_file) {
                    eprintln!("Warning: Failed to copy {}: {}", path.display(), e);
                }
            }
        }
    }
}

#[cfg(windows)]
fn find_windows_sdk_include() -> Option<String> {
    let common_paths = [
        "C:/Program Files (x86)/Windows Kits/10/include/10.0.26100.0/ucrt",
        "C:/Program Files (x86)/Windows Kits/10/include/10.0.22621.0/ucrt",
        "C:/Program Files (x86)/Windows Kits/10/include/10.0.22000.0/ucrt",
        "C:/Program Files (x86)/Windows Kits/10/include/10.0.19041.0/ucrt",
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // Try to find via WindowsSdkVerBinPath
    if let Ok(sdk_ver) = std::env::var("WindowsSdkVerBinPath") {
        // Extract version from path like "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"
        if let Some(version) = sdk_ver.rsplit("\\").next() {
            let include_path = format!(
                "C:/Program Files (x86)/Windows Kits/10/include/{}/ucrt",
                version
            );
            if std::path::Path::new(&include_path).exists() {
                return Some(include_path);
            }
        }
    }

    None
}

#[cfg(windows)]
fn find_rc_exe() -> String {
    // Try WindowsSdkVerBinPath environment variable first
    if let Ok(sdk_bin_path) = std::env::var("WindowsSdkVerBinPath") {
        let rc_path = format!("{}/x64/rc.exe", sdk_bin_path);
        if std::path::Path::new(&rc_path).exists() {
            return rc_path;
        }
    }

    // Try common Windows SDK installation paths
    let common_paths = [
        "C:/Program Files (x86)/Windows Kits/10/bin/x64/rc.exe",
        "C:/Program Files (x86)/Windows Kits/10/bin/10.0.26100.0/x64/rc.exe",
        "C:/Program Files (x86)/Windows Kits/10/bin/10.0.22621.0/x64/rc.exe",
        "C:/Program Files (x86)/Windows Kits/10/bin/10.0.22000.0/x64/rc.exe",
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Fallback to rc.exe in PATH
    "rc.exe".to_string()
}
