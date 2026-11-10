//! Find the feature names that rustc accepts in target feature checks.

use core::fmt::Write;
use std::{env, fs, path::Path, process::Command};

const X86: &[&str] = &[
    "apxf",
    "avx10.1",
    "avx10.2",
    "avx512er",
    "avx512pf",
    "clflushopt",
    "fma4",
    "kl",
    "movdir64b",
    "movdiri",
    "movrs",
    "widekl",
];

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTC");

    // Listing features does not require the target's standard library.
    let (target, candidates): (_, &[&str]) = match env::var("CARGO_CFG_TARGET_ARCH")
        .expect("Cargo did not set CARGO_CFG_TARGET_ARCH")
        .as_str()
    {
        "x86" => ("i686-unknown-linux-gnu", X86),
        "x86_64" => ("x86_64-unknown-linux-gnu", X86),
        _ => return,
    };
    let output = Command::new(env::var_os("RUSTC").expect("Cargo did not set RUSTC"))
        .args(["--print", "target-features", "--target", target])
        .output()
        .expect("failed to list rustc target features");
    assert!(
        output.status.success(),
        "failed to list rustc target features: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The first section lists rustc feature names. The second lists backend
    // features, which rustc can accept for code generation without a cfg name.
    let known = core::str::from_utf8(&output.stdout)
        .expect("rustc target features were not valid UTF-8")
        .lines()
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .collect::<Vec<_>>();

    let mut rules = String::from("#[macro_export]\n#[doc(hidden)]\nmacro_rules! __feature_cfg {\n");
    for &feature in candidates {
        if !known.contains(&feature) {
            writeln!(
                rules,
                r#"    ($all:tt [{feature:?}, $($rest:tt)*] => $body:expr) => {{
        $crate::__unless_target_features!(@runtime $body)
    }};"#
            )
            .expect("failed to write a feature rule");
        }
    }
    rules.push_str(
        r#"    ($all:tt [$feature:tt, $($rest:tt)*] => $body:expr) => {
        $crate::__feature_cfg!($all [$($rest)*] => $body)
    };
    ($all:tt [] => $body:expr) => {
        $crate::__unless_target_features!(@check $all => $body)
    };
}
"#,
    );
    let out_dir = env::var_os("OUT_DIR").expect("Cargo did not set OUT_DIR");
    fs::write(Path::new(&out_dir).join("feature_cfg.rs"), rules)
        .expect("failed to write feature_cfg.rs");
}
