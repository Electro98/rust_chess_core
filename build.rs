use flapigen::{JavaConfig, LanguageConfig};
use rifgen::{Generator, Language, TypeCases};
use std::env;
use std::path::Path;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    // Generate interface definition
    let _out_dir = env::var("OUT_DIR").expect("no OUT_DIR, where to save?");
    let source_folder = Path::new("./src");
    let out_file = source_folder.join("auto-glue.rs.in");
    Generator::new(TypeCases::CamelCase, Language::Java, source_folder)
        .generate_interface(out_file);

    // Generate glue for interface
    let java_config = JavaConfig::new(
        "../../app/src/main/java/ru/electro98/dark_chess/rust_chess".into(),
        "ru.electro98.dark_chess.rust_chess".into(),
    );

    if target_os != "android" {
        println!("[INFO]: No glue build is executed!");
        return;
    }
    let source_in = source_folder.join("glue.rs.in");
    let source_out = source_folder.join("glue.rs");
    let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(
        java_config.use_null_annotation_from_package("androidx.annotation".into()),
    ))
    .rustfmt_bindings(true);
    swig_gen.expand("android bindings", source_in, source_out);
}
