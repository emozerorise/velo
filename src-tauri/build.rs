fn main() {
    // Link libmpv using pkg-config if available
    if let Err(e) = pkg_config::Config::new().atleast_version("0.30.0").probe("mpv") {
        println!("cargo:warning=pkg-config could not find libmpv: {}", e);
        println!("cargo:rustc-link-lib=mpv");
    }

    tauri_build::build();
}
