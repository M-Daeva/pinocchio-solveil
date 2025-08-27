#[cfg(test)]
pub mod counter;

pub mod helpers {
    pub mod extensions {
        pub mod counter;
    }

    pub mod suite {
        pub mod core;
        pub mod decimal;
        pub mod types;
    }
}
