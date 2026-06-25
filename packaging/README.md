# Contrib

## Run shipd under systemd (Linux)

Copy the unit and enable it:

    mkdir -p ~/.config/systemd/user
    cp shipd.service ~/.config/systemd/user/
    systemctl --user daemon-reload
    systemctl --user enable --now shipd.service

`ship daemon status` will now show systemd-managed status.
`ship daemon start/stop/restart` still go through the legacy
supervisor — full systemctl delegation lands in a follow-up slice.
