use spirv_builder::SpirvBuilder;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../newport_shaders")
        .spirv_version(1, 0)
        .build()?;
    Ok(())
}