fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/drfiley.proto")?;

    static_files::resource_dir("./res/static").build()?;

    Ok(())
}
