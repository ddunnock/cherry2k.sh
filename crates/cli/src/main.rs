//! Cherry2K CLI Application
//!
//! Zsh terminal AI assistant with provider-agnostic architecture.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Initialize tracing with environment filter
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    println!("cherry2k v{}", env!("CARGO_PKG_VERSION"));
}
