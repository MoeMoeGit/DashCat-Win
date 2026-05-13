fn main() {
    // Link Windows resource file for icon and version info
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=resources.rc");
        println!("cargo:rerun-if-changed=assets/icons/app_icon.ico");
    }
}