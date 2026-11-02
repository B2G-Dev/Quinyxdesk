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
