fn main() {
    // Link against GNOME Online Accounts library when the goa feature is enabled
    #[cfg(feature = "goa")]
    {
        println!("cargo:rustc-link-lib=goa-1.0");
    }
    
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/goa_ffi.rs");
}
