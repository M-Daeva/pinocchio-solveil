use {
    base::{error::AuthError, types::ErrorIndexOffset},
    macro_error::any_error,
    strum_macros::EnumIter,
};

// Generate AnyError with all your error types
any_error! {
    CustomError => Custom,
    AuthError => Auth
}

// Your existing error enums - now they need EnumIter to work with the macro
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum CustomError {
    // "No parameters were provided"
    NoParameters,
    // "Wrong asset type was used"
    WrongAssetType,
    // "Zero amount can't be accepted"
    ZeroAmount,
}

impl ErrorIndexOffset for CustomError {
    const OFFSET: u32 = 0;
}
