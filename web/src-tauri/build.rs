// Tauri build script
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // Setup extension runner for Tauri bundling
    let binaries_dir = PathBuf::from("binaries");
    fs::create_dir_all(&binaries_dir).expect("Failed to create binaries directory");

    // Detect target platform
    let target = env::var("TARGET").unwrap_or_else(|_| String::from("unknown"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("release"));

    // Get project root (web/src-tauri)
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    // Navigate to workspace root (HeraMind/)
    // web/src-tauri -> web -> HeraMind
    let workspace_root = project_root
        .parent() // web/
        .and_then(|p| p.parent()) // HeraMind/
        .unwrap_or(&project_root);

    // Windows binaries carry the .exe suffix
    let exe_suffix = if target.contains("windows") { ".exe" } else { "" };

    // Tauri expects the sidecar at: binaries/heramind-extension-runner-{target_triple}[.exe]
    let platform_runner = binaries_dir
        .join(format!("heramind-extension-runner-{}{}", target, exe_suffix));

    // 1. If CI (or a local dev) already staged the sidecar in binaries/, use it as-is.
    //    fs::copy overwrites, so no need to remove a stale file first.
    if platform_runner.exists() {
        println!("cargo:warning=✅ Extension runner sidecar ready: {}", target);
        return;
    }

    // 2. Otherwise, copy it from the workspace build output if present.
    let source_runner = workspace_root
        .join("target")
        .join(&profile)
        .join(format!("heramind-extension-runner{}", exe_suffix));

    if source_runner.exists() {
        fs::copy(&source_runner, &platform_runner).expect("Failed to copy extension runner");

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&platform_runner).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&platform_runner, perms).expect("Failed to set permissions");
        }

        println!("cargo:warning=✅ Extension runner copied: {}", target);
        return;
    }

    // 3. Neither staged nor buildable — this is a genuine problem.
    println!("cargo:warning=⚠️  Extension runner not found. Looked for one of:");
    println!("cargo:warning=   staged sidecar : {}", platform_runner.display());
    println!("cargo:warning=   workspace build: {}", source_runner.display());
    println!(
        "cargo:warning=   Build it first: cargo build --{} -p heramind-extension-runner",
        profile
    );

    // NOTE: heramind-cli is no longer bundled as a Tauri sidecar. The agent's
    // shell tool dispatches data commands in-process via heramind-cli-ops
    // (compiled into this binary), eliminating the need for a separate CLI
    // binary in PATH or bundled as a sidecar. The standalone heramind-cli is
    // still built for server/Docker distributions via CI.
}
