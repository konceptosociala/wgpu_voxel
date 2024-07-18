use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("The swap chain has been lost and needs to be recreated")]
    Lost,
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
    #[error("Error acquiring current texture")]
    SurfaceError(String),
    #[error("Render components are not set up: {0:?}")]
    SetupError(Vec<&'static str>),
    #[error("Error loading vox file")]
    LoadVoxError(&'static str),
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(value: wgpu::SurfaceError) -> Self {
        match value {
            wgpu::SurfaceError::Lost => RenderError::Lost,
            wgpu::SurfaceError::OutOfMemory => RenderError::OutOfMemory,
            _ => RenderError::SurfaceError(value.to_string()),
        }
    }
}