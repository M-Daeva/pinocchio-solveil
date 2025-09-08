#[macro_export]
macro_rules! err {
    ($error:expr) => {
        Err(AnyError::from($error))?
    };
}

// Macro to generate AnyError and its implementations
#[macro_export]
macro_rules! any_error {
    (
        $(
            $error_type:ty => $variant:ident
        ),* $(,)?
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum AnyError {
            $(
                $variant($error_type),
            )*
        }

        impl AnyError {
            // Get the global index (offset + variant index)
            pub fn index(self) -> u32 {
                match self {
                    $(
                        AnyError::$variant(err) => <$error_type>::OFFSET + err as u32,
                    )*
                }
            }

            // Iterator that yields all possible AnyError variants with their global indices
            pub fn all_with_indices() -> impl Iterator<Item = (u32, AnyError)> {
                let mut variants = Vec::new();

                $(
                    // Use fully qualified path
                    for variant in <$error_type as strum::IntoEnumIterator>::iter() {
                        let any_err = AnyError::$variant(variant);
                        variants.push((any_err.index(), any_err));
                    }
                )*

                variants.into_iter()
            }

            // Iterator that yields just the AnyError variants
            pub fn all() -> impl Iterator<Item = AnyError> {
                Self::all_with_indices().map(|(_, err)| err)
            }

            // Iterator that yields just the global indices
            pub fn all_indices() -> impl Iterator<Item = u32> {
                Self::all_with_indices().map(|(index, _)| index)
            }
        }

        // Generate From implementations for each error type to AnyError
        $(
            impl From<$error_type> for AnyError {
                fn from(err: $error_type) -> Self {
                    AnyError::$variant(err)
                }
            }
        )*

        impl From<AnyError> for pinocchio::program_error::ProgramError {
            fn from(e: AnyError) -> Self {
                pinocchio::program_error::ProgramError::Custom(e.index())
            }
        }
    };
}
