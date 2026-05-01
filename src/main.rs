/* --- loonixtunesv2/src/main.rs | main --- */
// TEMPORARY: Enable console for debugging silent crashes
// #![cfg_attr(
//     all(target_os = "windows", not(debug_assertions)),
//     windows_subsystem = "windows"
// )]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "console"
)]
#![cfg_attr(
    all(target_os = "windows", debug_assertions),
    windows_subsystem = "console"
)]
use cstr::cstr;
use qmetaobject::*;

fn setup_panic_logger() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(mut appdata) = dirs::data_dir() {
            let _ = std::fs::create_dir_all(&appdata);
            let log_path = appdata.join("loonix-tunes").join("panic_dump.log");
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                use std::io::Write;
                let now = std::time::SystemTime::now();
                let _ = writeln!(&mut file, "=== PANIC at {:?} ===", now);
                let _ = writeln!(&mut file, "Panic: {:?}", panic_info);
            }
        }
    }));
}

pub mod audio;
pub mod core;
pub mod ui;

use crate::ui::bridge::DspController;
use crate::ui::bridge::MusicModel;
use crate::ui::bridge::PlayerBridge;
use crate::ui::components::ThemeManager;
use crate::ui::reportbug::BugReportManager;
use crate::ui::CustomThemeListModel;
use crate::ui::PopupMenu;

struct App {
    music_model: QObjectBox<MusicModel>,
    dsp_model: QObjectBox<DspController>,
    theme: QObjectBox<ThemeManager>,
    custom_theme_list: QObjectBox<CustomThemeListModel>,
    popup: QObjectBox<PopupMenu>,
    bridge: QObjectBox<PlayerBridge>,
    bug_report: QObjectBox<BugReportManager>,
}

impl App {
    fn new() -> Self {
        let music_raw = MusicModel::new();
        let ffmpeg = music_raw.get_ffmpeg();
        let config = music_raw.get_shared_config();

        let music_model = QObjectBox::new(music_raw);
        let dsp_model = QObjectBox::new(DspController::new(ffmpeg));

        if let Some(ref shared_config) = config {
            if let Ok(cfg) = shared_config.lock() {
                dsp_model.pinned().borrow_mut().init_from_config(&cfg);
            }
        }

        let theme = QObjectBox::new(ThemeManager::new());

        if let Some(shared_config) = config {
            theme.pinned().borrow_mut().set_config(shared_config);
        }

        let custom_list = CustomThemeListModel::default();
        let custom_theme_list = QObjectBox::new(custom_list);

        crate::audio::config::AppConfig::set_initializing(false);

        Self {
            music_model,
            dsp_model,
            theme,
            custom_theme_list,
            popup: QObjectBox::new(PopupMenu::default()),
            bridge: QObjectBox::new(PlayerBridge::new()),
            bug_report: QObjectBox::new(BugReportManager::default()),
        }
    }
}

fn setup_env() {
    std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Fusion");

    let exe_path = std::env::current_exe().unwrap_or_default();
    let base_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
    let plugin_path = base_dir.join("platforms");
    if plugin_path.exists() {
        std::env::set_var(
            "QT_QPA_PLATFORM_PLUGIN_PATH",
            plugin_path.to_string_lossy().as_ref(),
        );
    } else {
        std::env::set_var(
            "QT_QPA_PLATFORM_PLUGIN_PATH",
            "C:\\dev\\6.8.3\\msvc2022_64\\plugins\\platforms",
        );
    }
}

fn main() {
    setup_panic_logger();
    setup_env();

    init_resources_v4();

    let app = App::new();

    let mut engine = QmlEngine::new();

    qml_register_type::<MusicModel>(cstr!("Loonix"), 1, 0, cstr!("MusicModel"));
    qml_register_type::<DspController>(cstr!("Loonix"), 1, 0, cstr!("DspController"));
    qml_register_type::<PopupMenu>(cstr!("Loonix"), 1, 0, cstr!("PopupMenu"));
    qml_register_type::<ThemeManager>(cstr!("Loonix"), 1, 0, cstr!("ThemeManager"));
    qml_register_type::<CustomThemeListModel>(cstr!("Loonix"), 1, 0, cstr!("CustomThemeListModel"));
    qml_register_type::<BugReportManager>(cstr!("Loonix"), 1, 0, cstr!("BugReportManager"));
    qml_register_type::<PlayerBridge>(cstr!("Loonix"), 1, 0, cstr!("PlayerBridge"));

    engine.set_object_property("musicModel".into(), app.music_model.pinned());
    engine.set_object_property("dspModel".into(), app.dsp_model.pinned());
    engine.set_object_property("theme".into(), app.theme.pinned());
    engine.set_object_property("customThemeList".into(), app.custom_theme_list.pinned());
    engine.set_object_property("popupMenu".into(), app.popup.pinned());
    engine.set_object_property("playerBridge".into(), app.bridge.pinned());
    engine.set_object_property("bugReport".into(), app.bug_report.pinned());

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let files: Vec<String> = args[1..].to_vec();
        crate::ui::bridge::core::set_command_line_files(files);
    }

    // Use Qt Resource system (qrc) instead of relative path
    // The qrc! macro at bottom embeds QML files into the binary
    engine.load_file("qrc:/qml/Ui.qml".into());
    engine.exec();
}

qmetaobject::qrc!(init_resources_v4,
    "/" {
        "qtquickcontrols2.conf",
        "qml/Ui.qml",
        "qml/ui/tabs/Tab.qml",
        "qml/ui/tabs/TabMusic.qml",
        "qml/ui/tabs/TabFavorites.qml",
        "qml/ui/tabs/TabQueue.qml",
        "qml/ui/tabs/TabCustom.qml",
        "qml/ui/Dsp.qml",
        "qml/ui/components/TrackInfo.qml",
        "qml/ui/contextmenu/TabContextMenu.qml",
        "qml/ui/contextmenu/PlaylistContextMenu.qml",
        "qml/ui/contextmenu/AppearanceContextMenu.qml",
        "qml/ui/Playlist.qml",
        "qml/ui/Pref.qml",
        "qml/ui/pref/PrefAbout.qml",
        "qml/ui/pref/PrefDonate.qml",
        "qml/ui/pref/PrefAppearance.qml",
        "qml/ui/pref/PrefTab.qml",
        "qml/ui/pref/PrefSwitch.qml",
        "qml/ui/pref/PrefDropdown.qml",
        "qml/ui/pref/PrefSlider.qml",
        "qml/ui/pref/PrefCollapsibleSection.qml",
        "qml/ui/pref/PrefButton.qml",
        "qml/ui/pref/PrefReportBug.qml",
        "qml/ui/pref/PrefThemeEditor.qml",
        "qml/ui/components/ThemeSlider.qml",
        "qml/ui/components/RenameDialog.qml",
        "qml/ui/qmldir",
        "assets/LoonixTunes.png",
        "assets/fonts/KodeMono-VariableFont_wght.ttf",
        "assets/fonts/SymbolsNerdFont-Regular.ttf",
        "assets/fonts/twemoji.ttf",
        "assets/fonts/Oswald-Regular.ttf",
        "assets/images/saweriaqrcode.png",
        "assets/images/kofiqrcode.png",
    }
);
/* --- END --- */
