use {
    base::{error::AuthError, types::ErrorIndexOffset},
    macro_error::any_error,
    strum_macros::EnumIter,
};

// Generate AnyError with all error types
any_error! {
    CustomError => Custom,
    AuthError => Auth
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum CustomError {
    // "Parameters are not provided!"
    NoParameters,
    // "Wrong asset type!"
    WrongAssetType,
    // "Zero amount to send!"
    ZeroAmount,
    // "Exceeded available asset amount!"
    ExceededAvailableAssetAmount,
    // "The contract is temporary paused!"
    ContractIsPaused,
    // "Max data size is out of range!"
    MaxDataSizeIsOutOfRange,
    // "Max data size is exceeded!"
    MaxDataSizeIsExceeded,
    // "Wrong user ID!"
    WrongUserId,
    // "Account can't be activated twice!"
    ActivateAccountTwice,
    // "Account isn't activated!"
    AccountIsNotActivated,
    // "Account can't be opened twice!"
    OpenAccountTwice,
    // "Account isn't opened!"
    AccountIsNotOpened,
    // "Nonce must be unique!"
    BadNonce,
}

impl ErrorIndexOffset for CustomError {
    const OFFSET: u32 = 0;
}
