use egui_winit::winit::error::OsError;
use thiserror::Error;

use crate::rebind::Rebind;

#[derive(Error, Debug)]
pub enum Error {
    #[error("processing rebind failed. Rebind: {0}")]
    RebindProcessingFailed(Rebind),

    #[error("window creation failed")]
    WindowCreateFailed {
        #[source]
        source: OsError,
    },

    #[error("vku error")]
    VkuError {
        #[source]
        source: vku::Error,
    },

    #[error("vk error")]
    VkError {
        #[source]
        source: vku::ash::vk::Result,
    },

    #[error("graphics error")]
    GraphicsError(String),

    #[error("vjoy error")]
    VJoyError {
        #[source]
        source: vjoy::Error,
    },
}

impl From<vku::Error> for Error {
    fn from(value: vku::Error) -> Self {
        Self::VkuError { source: value }
    }
}

impl From<vku::ash::vk::Result> for Error {
    fn from(value: vku::ash::vk::Result) -> Self {
        Self::VkError { source: value }
    }
}

impl From<vjoy::Error> for Error {
    fn from(value: vjoy::Error) -> Self {
        Self::VJoyError { source: value }
    }
}
