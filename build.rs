use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when build.rs is changed - aka never
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    #[cfg(feature = "net")]
    println!("cargo:rustc-link-arg=-Trom_functions.x");

    dotenv::dotenv()?;

    println!("cargo::rustc-env=SSID={}", std::env::var("SSID")?);
    println!("cargo::rustc-env=PASSWORD={}", std::env::var("PASSWORD")?);

    Ok(())
}
