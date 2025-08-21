// Build script for paradigm-core
// For now, we'll skip protobuf compilation until protoc is available

fn main() {
    println!("cargo:rerun-if-changed=proto/paradigm.proto");
    println!("cargo:rerun-if-changed=proto");
    
    // Check if protoc is available
    if std::process::Command::new("protoc")
        .arg("--version")
        .output()
        .is_ok()
    {
        // Only compile protos if protoc is available
        tonic_build::compile_protos("proto/paradigm.proto")
            .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
    } else {
        println!("cargo:warning=protoc not found - skipping protobuf compilation");
        println!("cargo:warning=Install protoc to enable gRPC features");
    }
}
