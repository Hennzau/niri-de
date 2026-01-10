# `rilm`

A lightweight, all-included `niri` login/session manager and shell: one single executable with immutable configuration for Fedora 43 workstation.

[!NOTE]
This software is mostly design just for me. If you find it fun, you can use it. I will accept PR to make it more general for everyone only if I don't have to make lots of changes on my machine to make it work again.

# Installation

```
sudo dnf install xwayland-satellite niri alacritty rilm
```

[!NOTE]
If `rilm` is not available or if you want the latest updates, please build it from source: `cargo install --locked --git https://github.com/Hennzau/rilm`

# Getting started

## First time

Once installed it run those:

```bash
rilm patch-config

sudo systemctl disable gdm
sudo systemctl enable rilm
```

## Usage

At this stage, starting your Fedora should greet you directly in a `niri` instance where you need to log your user credentials. Once done, it should quickly start your `niri` session!

# Development

It's possible to test all the pipeline by doing a `winit` simulation on an already running compositor:

```bash
rilm start display winit
```

# License

MIT
