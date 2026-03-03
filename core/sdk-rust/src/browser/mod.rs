//! Browser backend abstraction and manager.

pub mod backend;
pub mod manager;
#[cfg(feature = "native-browser")]
pub mod native;

pub use backend::BrowserBackend;
pub use manager::BrowserManager;
