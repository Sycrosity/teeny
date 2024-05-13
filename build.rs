use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Only re-run the build script when build.rs is changed - aka never
    println!("cargo:rerun-if-changed=build.rs,.env");

    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    #[cfg(feature = "net")]
    println!("cargo:rustc-link-arg=-Trom_functions.x");

    dotenv::dotenv().ok();

    println!(
        "cargo::rustc-env=SSID={}",
        std::env::var("SSID").unwrap_or_default()
    );
    println!(
        "cargo::rustc-env=PASSWORD={}",
        std::env::var("PASSWORD").unwrap_or_default()
    );

    Ok(())
}
