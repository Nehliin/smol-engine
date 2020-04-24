use anyhow::{Context, Result};
use glsl_to_spirv::ShaderType;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;
use wgpu::{Device, ProgrammableStageDescriptor, ShaderModule};

#[derive(Error, Debug)]
pub enum ShaderLoadError {
    #[error("Failed to find shader file")]
    FailedToFind(#[from] std::io::Error),
    #[error("Failed to compile, {0}")]
    FailedToCompile(String),
}

pub struct Shader {
    module: ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, path: impl AsRef<Path>, shader_type: ShaderType) -> Result<Self> {
        // find and compile vertex shader
        let mut file = File::open(path.as_ref())
            .with_context(|| format!("Failed to find {:x?}, shader file", path.as_ref()))?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;
        let spirv =
            glsl_to_spirv::compile(&src, shader_type).map_err(ShaderLoadError::FailedToCompile)?;

        let data = wgpu::read_spirv(spirv)?;

        let module = device.create_shader_module(&data);

        Ok(Self { module })
    }

    pub fn get_descriptor(&self) -> ProgrammableStageDescriptor {
        ProgrammableStageDescriptor {
            module: &self.module,
            entry_point: "main",
        }
    }
}
