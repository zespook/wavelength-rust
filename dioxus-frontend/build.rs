fn main() {
    println!("cargo:rerun-if-env-changed=WAVELENGTH_SERVICES_SHA256");
    println!("cargo:rerun-if-env-changed=WAVELENGTH_SERVICES_ARCHIVE");
}
