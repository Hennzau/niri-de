build:
    cargo build --bins --release

install:
    sudo cp ./target/release/niri-lm /usr/local/bin

    sudo cp ./resources/niri-lm.service /etc/systemd/system
    sudo cp ./resources/niri-lm /etc/pam.d
    sudo cp ./resources/niri-lm.kdl /usr/local/share/niri-de/niri-lm.kdl
    sudo cp ./resources/niri.kdl ~/.config/niri/config.kdl

daemon-reload:
    sudo systemctl daemon-reload

start: build install daemon-reload
    sudo systemctl start niri-lm

local:
    /usr/bin/niri -c ./resources/niri.kdl -- alacritty

useradd:
    sudo useradd -M -G video niri-lm
