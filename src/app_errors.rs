use esp_idf_hal::{gpio::GpioError, sys::EspError};
use heapless::CapacityError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    App(String),

    #[error("SPI error: {0:?}")]
    Spi(mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>),

    #[error("Esp error: {0:?}")]
    Esp(EspError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>> for AppError {
    fn from(e: mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>) -> Self {
        Self::Spi(e)
    }
}

impl From<EspError> for AppError {
    fn from(e: EspError) -> Self {
        Self::Esp(e)
    }
}

impl From<CapacityError> for AppError {
    fn from(e: CapacityError) -> Self {
        Self::Other(anyhow::anyhow!(e))
    }
}

pub type AppResult<T> = Result<T, AppError>;
