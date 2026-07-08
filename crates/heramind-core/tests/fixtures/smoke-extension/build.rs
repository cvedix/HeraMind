// Set the macOS dylib install name at link time so that the runner's
// dylib validation accepts this fixture (it requires `@rpath/extension.dylib`).
fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/extension.dylib");
    }
}
