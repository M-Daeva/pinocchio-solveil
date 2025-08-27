use {crate::types::ErrorIndexOffset, strum_macros::EnumIter};

#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum AuthError {
    // "Sender doesn't have access permissions!"
    Unauthorized,
    // "New owner wasn't specified!"
    NoNewOwner,
    // "Sender can't be the new owner!"
    UselessRotation,
    // "It's too late to accept owner role!"
    TransferOwnerDeadline,
}

impl ErrorIndexOffset for AuthError {
    const OFFSET: u32 = 10_000;
}
