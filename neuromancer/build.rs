fn main() {
    println!("cargo:rerun-if-changed=./protos/base.proto");
    println!("cargo:rerun-if-changed=./protos/executor.proto");
    println!("cargo:rerun-if-changed=./protos/librarian.proto");
    tonic_build::configure()
        .compile(
            &[
                "./protos/executor.proto",
                "./protos/librarian.proto",
                "./protos/base.proto",
            ],
            &["./protos"],
        )
        .unwrap();
}
