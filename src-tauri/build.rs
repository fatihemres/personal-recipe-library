fn main() {
    #[cfg(target_os = "windows")]
    {
        // The app and Cargo's separate test harness both import Common Controls v6.
        // Embed the same dependency in both binaries; tauri-build's resource manifest
        // only covers the app binary and conflicts with a linker-embedded manifest.
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("windows-common-controls.manifest");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        let attributes = tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        tauri_build::try_build(attributes).expect("failed to run tauri-build");
    }

    #[cfg(not(target_os = "windows"))]
    tauri_build::build();
}
