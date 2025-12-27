# `niri-de`

# Installation

```bash
cargo build --bins release

sudo cp ./target/release/niri-login-root /usr/local/bin
sudo cp ./target/release/niri-login /usr/local/bin
sudo cp ./niri-login/niri-login.service /etc/systemd/system
sudo systemctl enable niri-login
```

# Uninstallation

```bash
sudo rm /usr/local/bin/niri-login-root
sudo rm /usr/local/bin/niri-login
sudo systemctl disable niri-login
sudo rm /etc/systemd/system/niri-login.service
```
