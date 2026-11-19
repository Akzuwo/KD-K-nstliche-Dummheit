fn main() {
    println!("cargo:rerun-if-env-changed=KD_UPDATE_REPOSITORY");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/kd.ico")
            .set("ProductName", "KD Desktop")
            .set("FileDescription", "KD Desktop")
            .compile()
            .expect("Windows-Ressourcen konnten nicht eingebettet werden");
    }
}
