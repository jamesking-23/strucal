fn main() {
    #[cfg(windows)]
    {
        if let Err(e) = winresource::WindowsResource::new().set_icon("assets/icon.ico").compile() {
            println!("cargo:warning=could not embed icon: {e}");
        }
    }
}
