#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod ui;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
fn main() {
    let root = kd_desktop::data_root();
    if let Err(e) = std::fs::create_dir_all(&root) {
        rfd::MessageDialog::new()
            .set_title("KD konnte nicht starten")
            .set_description(e.to_string())
            .show();
        return;
    }
    let instance_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("app.lock"));
    let _instance = match instance_file {
        Ok(file) => match file.try_lock() {
            Ok(()) => file,
            Err(_) => {
                rfd::MessageDialog::new()
                    .set_title("KD läuft bereits")
                    .set_description(
                        "Dieser Datenordner wird bereits von einem KD-Fenster verwendet.",
                    )
                    .show();
                return;
            }
        },
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("KD konnte nicht starten")
                .set_description(e.to_string())
                .show();
            return;
        }
    };
    if let Err(e) = kd_desktop::store::Store::open(&root) {
        rfd::MessageDialog::new()
            .set_title("Chatdatenbank konnte nicht geöffnet werden")
            .set_description(format!("{e:#}"))
            .show();
        return;
    }
    let png = &include_bytes!("../assets/kd.ico")[22..];
    let icon_image = image::load_from_memory(png)
        .expect("KD-Icon ungültig")
        .into_rgba8();
    let icon = dioxus::desktop::tao::window::Icon::from_rgba(
        icon_image.to_vec(),
        icon_image.width(),
        icon_image.height(),
    )
    .expect("KD-Icon konnte nicht geladen werden");
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("KD Desktop")
                .with_inner_size(LogicalSize::new(1280., 840.))
                .with_min_inner_size(LogicalSize::new(560., 500.)),
        )
        .with_icon(icon)
        .with_menu(None)
        .with_data_directory(root.join("webview"))
        .with_background_color((33, 33, 33, 255))
        .with_disable_context_menu(true)
        .with_navigation_handler(|url| {
            url.starts_with("dioxus://")
                || url.starts_with("http://dioxus.index.html/")
                || url.starts_with("https://dioxus.index.html/")
        });
    let app = dioxus::LaunchBuilder::desktop();
    #[cfg(windows)]
    let (app, config) = {
        use dioxus::desktop::tao::event::{Event, WindowEvent};
        let updater = kd_desktop::updater::Updater::start(&root);
        let on_close = updater.clone();
        let config = config.with_custom_event_handler(move |event, _| {
            if matches!(
                event,
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                }
            ) && let Some(updater) = &on_close
            {
                updater.install_on_close();
            }
        });
        (app.with_context(updater), config)
    };
    app.with_cfg(config).launch(ui::app);
}
