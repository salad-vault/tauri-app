pub mod account;
pub mod register_cmd;
pub mod session;

// Re-export helpers used from other command modules with the legacy path
// `crate::commands::auth::verify_master_password_inner`.
pub use session::verify_master_password_inner;
