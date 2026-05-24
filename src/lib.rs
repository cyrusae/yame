// Library target — exposes all modules so integration tests and external callers can use them.
// The binary target (main.rs) compiles independently; this crate is also a library.
pub mod app;
pub mod clipboard;
pub mod config;
pub mod decoration;
pub mod layout;
pub mod renderer;
pub mod status;
