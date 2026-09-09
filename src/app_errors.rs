use esp_idf_hal::gpio::GpioError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("SPI error: {0:?}")]
    Spi(mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>> for AppError {
    fn from(e: mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, GpioError>) -> Self {
        Self::Spi(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
