use std::env::{var, var_os};
use std::path::Path;
use std::process::Command;

fn main() {
    // Necessary env var to substitute in the final artifact depends on, regardless of the
    // build context; either in nix-shell or a basic nix build.

    // We need to support non-nix environment anyways, especially for VSCode and rust-analyser that
    // is being run by a VSCode extension and it doesn't have access to nix-shell to do a proper build.
    if var("IN_NIX").is_err() || var("IN_NIX").unwrap() != "1" {
        println!("cargo:rustc-env=CNS_ESSENTIALS=/nix/var/nix/profiles/default/bin:/usr/local/bin:/usr/bin:/usr/sbin:/bin:/sbin:/opt/homebrew/bin:/opt/homebrew/sbin");
        println!("cargo:rustc-env=CNS_BASH=bash");
        println!("cargo:rustc-env=CNS_NIX=");
    } else {
        println!(
            "cargo:rustc-env=CNS_ESSENTIALS={}",
            var("ESSENTIALS")
                .expect("Expect to the ESSENTIALS env var to be set.")
        );
        println!(
            "cargo:rustc-env=CNS_BASH={}",
            var("BASH_INTERACTIVE").expect("Expect to the BASH_INTERACTIVE env var to be set.")
        );
        println!(
            "cargo:rustc-env=CNS_NIX={}/",
            var("NIX_BIN").expect("Expect to the NIX_BIN env var to be set.")
        );
    }

    if var_os("CNS_IN_NIX_SHELL").is_none() {
        // Release build triggered by nix-build. Use paths relative to $out.
        let out = var("out").unwrap();
        println!("cargo:rustc-env=CNS_TRACE_NIX_SO={out}/lib/trace-nix.so");
        println!("cargo:rustc-env=CNS_VAR_EMPTY={out}/var/empty");
        println!(
            // This file is moved to /share/cached-nix-shell in the ./nix/Makefile#post-install
            "cargo:rustc-env=CNS_RCFILE={out}/share/cached-nix-shell/rcfile.sh"
        );
        println!(
            // This directory is created in the ./nix/Makefile#post-install
            "cargo:rustc-env=CNS_WRAP_PATH={out}/libexec/cached-nix-shell"
        );
    } else {
        // Developer build triggered by `nix-shell --run 'cargo build'`.
        // Use paths relative to the build directory. Additionally, place
        // trace-nix.so and a symlink to the build directory.
        let out_dir = var("OUT_DIR").unwrap();
        let cmd = Command::new("make")
            .args([
                "-C",
                "nix-trace",
                &format!("DESTDIR={out_dir}"),
                &format!("{out_dir}/trace-nix.so"),
            ])
            .status()
            .unwrap();
        assert!(cmd.success());

        println!("cargo:rustc-env=CNS_TRACE_NIX_SO={out_dir}/trace-nix.so");
        println!("cargo:rustc-env=CNS_VAR_EMPTY=/var/empty");
        println!(
            "cargo:rustc-env=CNS_RCFILE={}/rcfile.sh",
            var("CARGO_MANIFEST_DIR").unwrap()
        );

        if Path::new(&format!("{out_dir}/wrapper")).exists() {
            std::fs::remove_dir_all(format!("{out_dir}/wrapper")).unwrap();
        }
        std::fs::create_dir_all(format!("{out_dir}/wrapper")).unwrap();
        std::os::unix::fs::symlink(
            "../../../../cached-nix-shell",
            format!("{out_dir}/wrapper/nix-shell"),
        )
        .unwrap();
        println!("cargo:rustc-env=CNS_WRAP_PATH={out_dir}/wrapper");
    }
}
