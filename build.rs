fn main() {
    // Registrar la cfg personalizada usada en atributos #[cfg(not(tarpaulin_include))]
    // para que el compilador no emita warnings de unexpected cfg.
    println!("cargo:rustc-check-cfg=cfg(tarpaulin_include)");
}

