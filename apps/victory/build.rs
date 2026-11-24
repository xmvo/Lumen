fn main() {
    // The D3D12 loader reads these from the process executable to pick up an
    // Agility SDK runtime from D3D12SDKPath; executables export nothing by default.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
    }
}
