// src-tauri/build.rs
fn main() {
    println!(
        "cargo:rustc-env=ENGINE_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    let meta: serde_json::Value =
        serde_json::from_str(include_str!("../app.meta.json")).expect("app metadata");
    println!("cargo:rerun-if-changed=../app.meta.json");
    println!(
        "cargo:rustc-env=APP_NAME={}",
        meta["name"].as_str().unwrap()
    );
    println!(
        "cargo:rustc-env=APP_ID={}",
        meta["identifier"].as_str().unwrap()
    );
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "bootstrap",
            "open_settings",
            "drag_overlay",
            "translate",
            "cancel_translation",
            "save_settings",
            "set_credential",
            "remove_credential",
            "inspect_provider",
            "local_engine_status",
            "install_local_model",
            "copy_text",
            "insert_translation",
            "hide_overlay",
            "clear_session",
            "pause_shortcuts",
        ]),
    ))
    .expect("Tauri build")
}
