use {
    base::{error::AuthError, traits::ErrorIndexOffset},
    codama::CodamaErrors,
    macro_error::any_error,
    strum_macros::EnumIter,
    thiserror::Error,
};

// Generate AnyError with all error types
any_error! {
    CustomError => Custom,
    AuthError => Auth
}

#[repr(u8)]
#[derive(CodamaErrors, Error, Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum CustomError {
    #[error("Parameters are not provided!")]
    NoParameters,

    #[error("Wrong asset type!")]
    WrongAssetType,

    #[error("Zero amount to send!")]
    ZeroAmount,

    #[error("Exceeded available asset amount!")]
    ExceededAvailableAssetAmount,

    #[error("The contract is temporary paused!")]
    ContractIsPaused,

    #[error("Max data size is out of range!")]
    MaxDataSizeIsOutOfRange,

    #[error("Max data size is exceeded!")]
    MaxDataSizeIsExceeded,

    #[error("Wrong user ID!")]
    WrongUserId,

    #[error("Account can't be activated twice!")]
    ActivateAccountTwice,

    #[error("Account isn't activated!")]
    AccountIsNotActivated,

    #[error("Account can't be opened twice!")]
    OpenAccountTwice,

    #[error("Account isn't opened!")]
    AccountIsNotOpened,

    #[error("Nonce must be unique!")]
    BadNonce,
}

impl ErrorIndexOffset for CustomError {
    const OFFSET: u32 = 0;
}
