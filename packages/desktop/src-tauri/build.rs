fn main() {
    // Only the `ui` feature links Tauri (native WebKit); without it this is
    // a no-op so headless/workspace gates (`--no-default-features`) don't
    // need `tauri-build` or its `OUT_DIR` codegen.
    #[cfg(feature = "ui")]
    tauri_build::build();
}
