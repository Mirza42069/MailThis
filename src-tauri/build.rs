/// The Google OAuth client baked into the app (`BUILT_IN` in lib.rs) comes from
/// the environment or from a git-ignored `.env` at the repo root.
const KEYS: [&str; 2] = ["MAILTHIS_GOOGLE_CLIENT_ID", "MAILTHIS_GOOGLE_CLIENT_SECRET"];

fn main() {
    println!("cargo:rerun-if-changed=../.env");
    for key in KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
    if let Ok(env) = std::fs::read_to_string("../.env") {
        for (key, value) in env.lines().filter_map(|line| line.split_once('=')) {
            let (key, value) = (key.trim(), value.trim().trim_matches('"'));
            if KEYS.contains(&key) && !value.is_empty() && std::env::var_os(key).is_none() {
                println!("cargo:rustc-env={key}={value}");
            }
        }
    }
    tauri_build::build()
}
