use egui_winit::winit::error::OsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("rebind is empty.")]
    EmptyRebindOrInvalidID(),

    #[error("processing rebind failed. Rebind name: {0}")]
    RebindProcessingFailed(String),

    #[error("validating rebind failed. Physical src device: {0} | src button: {1}")]
    RebindValidatePhysicalButtonFailed(String, u32),

    #[error("validating rebind failed. Physical src device: {0} | src hat: {1}")]
    RebindValidatePhysicalHatFailed(String, u32),

    #[error("validating rebind failed. Physical src device: {0} | src axis: {1}")]
    RebindValidatePhysicalAxisFailed(String, u32),

    #[error("validating rebind failed. Virtual src device: {0} | src button: {1}")]
    RebindValidateVirtualButtonFailed(u32, u32),

    #[error("validating rebind failed. Virtual src device: {0} | src hat: {1}")]
    RebindValidateVirtualHatFailed(u32, u32),

    #[error("validating rebind failed. Virtual src device: {0} | src axis: {1}")]
    RebindValidateVirtualAxisFailed(u32, u32),

    #[error("window creation failed. Reason: {}", source)]
    WindowCreateFailed {
        #[from]
        source: OsError,
    },

    #[error("vku error. Reason: {}", source)]
    Vku {
        #[from]
        source: vku::Error,
    },

    #[error("vk error. Reason: {}", source)]
    Vk {
        #[from]
        source: vku::ash::vk::Result,
    },

    #[error("vjoy error. Reason: {}", source)]
    VJoy {
        #[from]
        source: vjoy::Error,
    },

    #[error("sdl2 error. Reason: {}", source)]
    SDL2 {
        #[from]
        source: sdl2::IntegerOrSdlError,
    },

    #[error("io error. Reason: {}", source)]
    IO {
        #[from]
        source: std::io::Error,
    },

    #[error("failed to serialize config file. Reason: {}", source)]
    Serialization {
        #[from]
        source: toml::ser::Error,
    },

    #[error("failed to deserialize config file. Reason: {}", source)]
    Deserialization {
        #[from]
        source: toml::de::Error,
    },

    #[error("untyped error. Reason: {0}")]
    Catch(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Catch(value)
    }
}
