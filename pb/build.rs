fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "../api/proto/auth.proto",
        "../api/proto/shop.proto",
        "../api/proto/confirmation.proto",
    ];

    let includes = &[
        "../api/proto/",
        "../third-party/grpc-gateway/third_party/googleapis/",
    ];

    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    tonic_build::configure().format(false).compile(protos, includes)?;

    Ok(())
}
