/* --- loonixtunesv2/src/ui/reportbug.rs | reportbug --- */
use qmetaobject::prelude::*;
use std::process::Command;

#[derive(QObject, Default)]
pub struct BugReportManager {
    base: qt_base_class!(trait QObject),
    
    // Method untuk dipanggil dari QML
    pub report_bug: qt_method!(fn(&self, title: QString, desc: QString)),
}

impl BugReportManager {
    pub fn report_bug(&self, title: QString, desc: QString) {
        // Ganti URL ini sesuai dengan repository target lu
        let repo_url = "https://github.com/citzeye/loonixtunesv2/issues/new";
        
        let os_info = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let version = env!("CARGO_PKG_VERSION");

        let title_str = title.to_string();
        let desc_str = desc.to_string();

        // Format body Issue dengan info sistem otomatis
        let body = format!(
            "### Describe the bug\n{}\n\n### System Info\n- OS: {}\n- Arch: {}\n- Version: v{}",
            desc_str, os_info, arch, version
        );

        // Encode biar aman masuk ke URL parameter
        let encoded_title = urlencoding::encode(&title_str);
        let encoded_body = urlencoding::encode(&body);
        let final_url = format!("{}?title={}&body={}", repo_url, encoded_title, encoded_body);

        // Buka browser default di Linux
        let _ = Command::new("xdg-open")
            .arg(final_url)
            .spawn();
    }
}