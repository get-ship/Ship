pub mod apple_container;
pub mod docker;
pub mod lima;
pub mod srt;
pub mod wsl;

#[cfg(target_os = "linux")]
pub mod firecracker;

#[cfg(all(feature = "youki-backend", target_os = "linux"))]
pub mod youki;

#[cfg(all(feature = "youki-backend", feature = "sidecar-mode", target_os = "linux"))]
pub mod youki_sidecar;

pub use apple_container::{AppleContainer, AppleContainerConfig};
pub use docker::{Docker, DockerConfig};
pub use lima::{Lima, LimaConfig};
pub use srt::{render_settings, SrtSettings};
pub use wsl::{Wsl, WslConfig};

#[cfg(target_os = "linux")]
pub use firecracker::{Firecracker, FirecrackerConfig};
// Noop now lives in substrate alongside the trait — re-export so
// existing call sites keep working.
pub use isolation_port::Noop;

#[cfg(all(feature = "youki-backend", target_os = "linux"))]
pub use youki::{Youki, YoukiConfig};

#[cfg(all(feature = "youki-backend", feature = "sidecar-mode", target_os = "linux"))]
pub use youki_sidecar::{SidecarConfig, SidecarSandbox};
