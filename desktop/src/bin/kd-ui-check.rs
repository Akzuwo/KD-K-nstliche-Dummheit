//! Regression driver: the real Dioxus UI in an isolated native WebView2.
//! No external desktop input, mock handlers, or production user data.
#[path = "../ui.rs"]
mod ui;
fn main() -> anyhow::Result<()> {
    let root = kd_desktop::data_root();
    anyhow::ensure!(
        std::env::var_os("KD_DATA_DIR").is_some(),
        "UI check requires isolated KD_DATA_DIR"
    );
    std::fs::create_dir_all(&root)?;
    let store = kd_desktop::store::Store::open(&root)?;
    if store.chats("")?.is_empty() {
        for (title, text) in [
            ("Navigationstest Alpha", "Suchwort Abakus"),
            ("Navigationstest Beta", "Anderes Gesprächsthema"),
        ] {
            let id = store.create_chat(title)?;
            store.add(&id, "user", text, "complete", "")?;
            store.add(
                &id,
                "assistant",
                "Lokaler UI-Testdatensatz.",
                "complete",
                "Test",
            )?;
        }
    }
    let width = std::env::var("KD_CHECK_WIDTH")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1280.);
    let config = dioxus::desktop::Config::new()
        .with_menu(None)
        .with_data_directory(root.join("webview"))
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("KD Desktop UI Regression")
                .with_visible(false)
                .with_inner_size(dioxus::desktop::LogicalSize::new(width, 840.)),
        );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui::app);
    Ok(())
}
