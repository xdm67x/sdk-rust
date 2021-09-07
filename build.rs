use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

const BINDGEN_OUTPUT_FILENAME: &str = "ctanker.rs";

fn main() -> Result<(), Box<dyn Error>> {
    let target_triplet = std::env::var("TARGET")?;
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let mut bindings_folder = PathBuf::from(manifest_dir);
    bindings_folder.push("native");
    bindings_folder.push(&target_triplet);

    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY")?;

    let lib_filename = "libctanker.a";
    if !bindings_folder.exists() {
        panic!(
            "Target platform {} is not supported ({} does not exist)",
            target_triplet,
            bindings_folder.to_string_lossy()
        );
    }
    if target_family == "unix" && !bindings_folder.join(lib_filename).exists() {
        panic!(
            "Couldn't find {} in {}",
            lib_filename,
            bindings_folder.to_string_lossy()
        );
    }
    if !bindings_folder.join(BINDGEN_OUTPUT_FILENAME).exists() {
        panic!(
            "Couldn't find the bindgen-generated {} in {}",
            BINDGEN_OUTPUT_FILENAME,
            bindings_folder.to_string_lossy()
        );
    }

    println!(
        "cargo:rerun-if-changed={}/{}",
        bindings_folder.to_string_lossy(),
        BINDGEN_OUTPUT_FILENAME
    );
    println!(
        "cargo:rerun-if-changed={}/{}",
        bindings_folder.to_string_lossy(),
        lib_filename
    );

    let bindings_folder = bindings_folder.to_string_lossy();

    // Paths can contain anything, but env vars are a liiitle more restricted. Sanity checks!
    assert!(!bindings_folder.contains(&"="));
    assert!(!bindings_folder.contains(&"\0"));
    assert!(!bindings_folder.contains(&"\n"));

    // Export an env var so we can include ctanker.rs in the source code
    println!("cargo:rustc-env=NATIVE_BINDINGS_FOLDER={}", bindings_folder);

    // Tell cargo to link with our native library
    if target_family == "unix" {
        println!("cargo:rustc-link-search={}", bindings_folder);
        println!("cargo:rustc-link-lib=static=ctanker",);
        match target_triplet.as_str() {
            "x86_64-unknown-linux-gnu" => println!("cargo:rustc-link-lib=dylib=stdc++"),
            "x86_64-apple-darwin" => {
                println!("cargo:rustc-link-lib=dylib=c++");
                println!("cargo:rustc-link-lib=dylib=c++abi");
            }
            _ => (),
        }
    }

    if target_family == "windows" {
        let build_type = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };

        let tankersdk_bin_path = format!("native/{}", target_triplet);
        let tankersdk_bin_path = Path::new(&tankersdk_bin_path);
        let unit_test_path = format!("target/{}/{}/deps/", target_triplet, build_type);
        let unit_test_path = Path::new(&unit_test_path);
        std::fs::create_dir_all(unit_test_path)?;
        let target_path = unit_test_path.join("ctanker.dll");
        if !target_path.exists() {
            std::fs::copy(tankersdk_bin_path.join("ctanker.dll"), target_path)?;
        }
    }

    Ok(())
}
