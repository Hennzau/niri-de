install:
    cargo build --bins --release
    sudo cp ./target/release/niri-lm /usr/local/bin

    sudo cp ./resources/niri-lm.service /etc/systemd/system
    sudo cp ./resources/niri-lm.kdl /usr/local/share/niri-de/niri-lm.kdl

    sudo systemctl daemon-reload

start: install
    sudo systemctl start niri-lm

log:
    journalctl -u niri-lm -b

useradd:
    sudo useradd -M -G video nirilm

enable:
    sudo systemctl enable --now niri-lm
