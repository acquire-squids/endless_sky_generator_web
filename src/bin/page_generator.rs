cfg_select! {
    all(target_family = "wasm", target_os = "unknown") => {
        const fn main() {}
    }
    _ => {
        use endless_sky_generator_web::html;

        use std::fs;

        fn main() -> std::io::Result<()> {
            let html = html::page_contents();

            fs::create_dir_all("output/")?;
            fs::write("output/index.html", html)
        }
    }
}
