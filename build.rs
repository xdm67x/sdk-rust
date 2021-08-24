use std::error::Error;
use std::io::Write;
#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_family = "windows")]
use std::path::Path;
use std::path::PathBuf;

const BINDGEN_OUTPUT_FILENAME: &str = "ctanker.rs";
const TANKER_LIB_BASENAME: &str = "ctanker";

fn main() -> Result<(), Box<dyn Error>> {
    let target_triplet = std::env::var("TARGET")?;
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let mut bindings_folder = PathBuf::from(manifest_dir);
    bindings_folder.push("native");
    bindings_folder.push(&target_triplet);

    #[cfg(target_family = "unix")]
    let tanker_lib_filename = &format!("lib{}.a", TANKER_LIB_BASENAME);
    #[cfg(not(target_family = "unix"))]
    let tanker_lib_filename = "ctanker.lib";
    if !bindings_folder.exists() {
        panic!(
            "Target platform {} is not supported ({} does not exist)",
            target_triplet,
            bindings_folder.to_string_lossy()
        );
    }
    if !bindings_folder.join(tanker_lib_filename).exists() {
        panic!(
            "Couldn't find {} in {}",
            tanker_lib_filename,
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
        tanker_lib_filename
    );

    // Paths can contain anything, but env vars are a liiitle more restricted. Sanity checks!
    #[cfg(target_family = "unix")]
    let bindings_folder = bindings_folder.as_os_str().as_bytes();
    #[cfg(target_family = "unix")]
    {
        assert!(!bindings_folder.contains(&b'='));
        assert!(!bindings_folder.contains(&b'\0'));
        assert!(!bindings_folder.contains(&b'\n'));
    }

    #[cfg(not(target_family = "unix"))]
    let bindings_folder = bindings_folder.to_string_lossy();
    #[cfg(not(target_family = "unix"))]
    let bindings_folder = bindings_folder.as_bytes();
    // Export an env var so we can include ctanker.rs in the source code
    print!("cargo:rustc-env=NATIVE_BINDINGS_FOLDER=");
    std::io::stdout().write_all(bindings_folder).unwrap();
    println!();

    // Tell cargo to link with our native library
    print!("cargo:rustc-link-search=");
    std::io::stdout().write_all(bindings_folder).unwrap();
    println!();
    println!("cargo:rustc-link-lib=static={}", TANKER_LIB_BASENAME);
    match target_triplet.as_str() {
        "x86_64-unknown-linux-gnu" => println!("cargo:rustc-link-lib=dylib=stdc++"),
        "x86_64-apple-darwin" => {
            println!("cargo:rustc-link-lib=dylib=c++");
            println!("cargo:rustc-link-lib=dylib=c++abi");
        }
        _ => (),
    }

    #[cfg(target_family = "windows")]
    {
        let build_type = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let tankersdk_bin_path = Path::new("./native/x86_64-pc-windows-msvc");
        let target_path = format!("target/x86_64-pc-windows-msvc/{}/deps/", build_type);
        let target_path = Path::new(&target_path);
        std::fs::create_dir_all(target_path)?;
        // copy the DLL alongside unit tests
        std::fs::copy(
            tankersdk_bin_path.join("ctanker.dll"),
            target_path.join("ctanker.dll"),
        )?;
    }

    Ok(())
}
