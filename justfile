build:
    cargo build --bins --release

install-niri-lm:
    sudo cp ./target/release/niri-lm /usr/local/bin

    sudo cp ./resources/niri-lm.service /etc/systemd/system
    sudo cp ./resources/niri-lm /etc/pam.d
    sudo cp ./resources/niri-lm.kdl /usr/local/share/niri-de/niri-lm.kdl

daemon-reload:
    sudo systemctl daemon-reload

start: build install-niri-lm daemon-reload
    sudo systemctl start niri-lm

useradd:
    sudo useradd -M -G video niri-lm
