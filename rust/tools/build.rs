use std::{
    process::{exit, Command},
    str,
};

static CARGOENV: &str = "cargo:rustc-env=";

#[cfg(feature = "bindings")]
fn cbindgen() {
    use std::env;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = format!(
        "{}/../target/include",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    let trailer = include_str!("../src/bindings/trailer.inc");
    let header_c = include_str!("../src/bindings/header_c.inc");
    let header_cpp = include_str!("../src/bindings/header_cpp.inc");

    // Generate C headers.
    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_language(cbindgen::Language::C)
        .with_no_includes()
        .with_header(header_c)
        .with_trailer(trailer)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/dqcsim.h", out_dir));

    // Generate C++ minimal API headers.
    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_language(cbindgen::Language::Cxx)
        .with_no_includes()
        .with_header(header_cpp)
        .with_namespaces(&["dqcsim", "raw"])
        .with_trailer(trailer)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/cdqcsim", out_dir));

    // Generate SWIG header.
    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_language(cbindgen::Language::C)
        .with_line_length(100000)
        .with_documentation(false)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/dqcsim-py.h", out_dir));
}

fn main() {
    let time_c = Command::new("date").args(&["+%F %T"]).output();

    match time_c {
        Ok(t) => {
            let time;
            unsafe {
                time = str::from_utf8_unchecked(&t.stdout);
            }
            println!("{}COMPILED_AT={}", CARGOENV, time);
        }
        Err(_) => exit(1),
    }

    #[cfg(feature = "bindings")]
    cbindgen();
}
