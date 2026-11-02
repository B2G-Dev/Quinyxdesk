fn main() {
    // Rebuild when any baked-in CI secret changes, so `option_env!` values stay fresh
    // across builds even with caching enabled.
    for var in [
        "RENDEZVOUS_SERVER",
        "RS_PUB_KEY",
        "API_SERVER",
        "PRESET_PASSWORD",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    bake_preset_password_hash();

    let out_dir = format!("{}/protos", std::env::var("OUT_DIR").unwrap());

    std::fs::create_dir_all(&out_dir).unwrap();

    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(out_dir)
        .inputs(["protos/rendezvous.proto"])
        .include("protos")
        .customize(protobuf_codegen::Customize::default().tokio_bytes(true))
        .run()
        .expect("Codegen failed.");
}

// Only the salted hash of PRESET_PASSWORD reaches the binary, in the preset storage format
// read by `decode_preset_password_h1_from_storage`: "00" + base64(sha256(password + salt)).
fn bake_preset_password_hash() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use sha2::{Digest, Sha256};

    const SALT: &str = "quinyxdesk-preset-v1";
    let Ok(password) = std::env::var("PRESET_PASSWORD") else {
        return;
    };
    let password = password.trim();
    if password.is_empty() {
        return;
    }
    let h1 = Sha256::new()
        .chain_update(password.as_bytes())
        .chain_update(SALT.as_bytes())
        .finalize();
    println!(
        "cargo:rustc-env=PRESET_PASSWORD_STORAGE=00{}",
        STANDARD.encode(h1)
    );
    println!("cargo:rustc-env=PRESET_PASSWORD_SALT={SALT}");
}
