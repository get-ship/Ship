pub mod docker;
pub mod noop;
pub mod sandbox_runtime;

#[cfg(all(feature = "youki-backend", target_os = "linux"))]
pub mod youki;

#[cfg(all(feature = "youki-backend", feature = "sidecar-mode", target_os = "linux"))]
pub mod youki_sidecar;

pub use docker::{Docker, DockerConfig};
pub use noop::Noop;
pub use sandbox_runtime::{SandboxRuntime, SandboxRuntimeConfig};

#[cfg(all(feature = "youki-backend", target_os = "linux"))]
pub use youki::{Youki, YoukiConfig};

#[cfg(all(feature = "youki-backend", feature = "sidecar-mode", target_os = "linux"))]
pub use youki_sidecar::{SidecarConfig, SidecarSandbox};
