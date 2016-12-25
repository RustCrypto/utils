extern crate gcc;

use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    if target.contains("msvc") && host.contains("windows") {
        let mut config = gcc::Config::new();
        config.file("src/helpers.asm");
        config.file("src/helpers.asm");
        if target.contains("x86_64") {
            config.define("X64", None);
        }
        config.compile("lib_constant_op_helpers.a");
    }
    else {
        let mut cfg = gcc::Config::new();
        cfg.file("src/helpers.c");
        cfg.file("src/helpers.c");
        if env::var_os("CC").is_none() {
            if host.contains("openbsd") {
                // Use clang on openbsd since there have been reports that
                // GCC doesn't like some of the assembly that we use on that
                // platform.
                cfg.compiler(Path::new("clang"));
            } else if target == host {
                cfg.compiler(Path::new("cc"));
            }
        }
        cfg.compile("lib_constant_op_helpers.a");
    }
}

