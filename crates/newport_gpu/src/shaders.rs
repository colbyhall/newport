use hassle_rs::compile_hlsl;

use crate::ShaderVariant;

pub fn compile(name: &str, source: &str, main: &str, variant: ShaderVariant) -> Option<Vec<u8>> {
    let target_profile = match variant {
        ShaderVariant::Pixel => "ps_6_1",
        ShaderVariant::Vertex => "vs_6_1"
    };

    let mut args = Vec::with_capacity(4); // TODO: Temp allocator

    #[cfg(feature = "vulkan")]
    {
        args.push("-spirv");
        args.push("-Zpc"); // Column major matrices

        if variant == ShaderVariant::Vertex {
            args.push("-fvk-invert-y");
        }
    }

    Some(compile_hlsl(name, source, main, target_profile, &args[..], &[]).unwrap())
}