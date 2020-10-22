fn main() {
    if std::env::var("WYCHEPROOF_DIR").is_err() {
        // No WYCHEPROOF_DIR set in the environment, so try to clone out the repo.
        let mut wycheproof_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        wycheproof_dir.push("wycheproof");
        if !wycheproof_dir.is_dir() {
            git2::Repository::clone("https://github.com/google/wycheproof", wycheproof_dir)
                .unwrap();
        }
    }
}
