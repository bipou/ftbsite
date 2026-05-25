use leptos_i18n_build::{Config, TranslationsInfos};
use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=locales");

    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    // 根据 feature 选择 locale 源目录，复制到 locales/ 供 leptos_i18n 读取
    let src_dir = if std::env::var("CARGO_FEATURE_OTH").is_ok() {
        "locales/oth"
    } else {
        "locales/cn"
    };
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let dest = PathBuf::from("locales").join(entry.file_name());
        fs::copy(entry.path(), &dest)?;
    }

    let locales: Vec<String> = fs::read_dir("locales")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .filter_map(|p| p.file_stem()?.to_str().map(String::from))
        .collect();

    let default = "zh".to_string();
    let default = if locales.contains(&default) {
        default
    } else {
        locales[0].clone()
    };

    let mut cfg = Config::new(&default)?;
    cfg.options.interpolate_display = true;
    for loc in &locales {
        if loc != &default {
            cfg = cfg.add_locale(loc)?;
        }
    }

    let infos = TranslationsInfos::parse(cfg)?;
    infos.emit_diagnostics();
    infos.rerun_if_locales_changed();
    infos.generate_i18n_module(out)?;

    Ok(())
}
