fn main() {
    // Locate libmpv through pkg-config when it is available. Failing that,
    // fall back to asking the linker for `mpv` directly: that works wherever
    // the library sits on the default search path, and otherwise fails at
    // link time with a bare "cannot open input file", so spell out the fix
    // here while there is still context to explain it.
    //
    // `cargo:warning=` is one directive per line, and pkg-config's own error
    // is multi-line boilerplate that only repeats "not found", so it is left
    // out in favour of the actionable hint below.
    if pkg_config::Config::new()
        .atleast_version("0.30.0")
        .probe("mpv")
        .is_err()
    {
        println!("cargo:warning=libmpv was not found via pkg-config; linking `mpv` directly.");
        println!(
            "cargo:warning=If linking fails, install libmpv where the toolchain can find it: \
             macOS `brew install mpv pkg-config`, Linux `apt install libmpv-dev`, \
             Windows: unpack the libmpv dev files and add their lib directory to LIB."
        );
        println!("cargo:rustc-link-lib=mpv");
    }

    tauri_build::build();
}
