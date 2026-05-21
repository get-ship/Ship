//! Single-source `settings.*` ops — descriptors + dispatch via `ship_ops!`.

use crate::services::settings::view_state::{build_ctx, SettingsCtx};

ship_ops! {
    SETTINGS_OPS;
    {
        name: "settings.state",
        desc: "Harnesses, defaults, terminal, about — everything the settings view renders.",
        perm: "ship.settings.read",
        args: &[],
        ret: SettingsCtx,
        run: |_args, _state, _files| Ok(build_ctx()),
    },
}
