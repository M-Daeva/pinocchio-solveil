use {
    crate::traits::ErrorIndexOffset, codama::CodamaErrors, strum_macros::EnumIter, thiserror::Error,
};

#[repr(u8)]
#[derive(CodamaErrors, Error, Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum AuthError {
    #[error("Sender doesn't have access permissions!")]
    Unauthorized,

    #[error("New owner wasn't specified!")]
    NoNewOwner,

    #[error("Sender can't be the new owner!")]
    UselessRotation,

    #[error("It's too late to accept owner role!")]
    TransferOwnerDeadline,
}

impl ErrorIndexOffset for AuthError {
    const OFFSET: u32 = 1_000;
}
