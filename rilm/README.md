# `niri-login`

This is a really simple login manager for a `niri-session`.

# Install

You must create the login user:

```bash
sudo useradd -M -G video nirilogin
```

`niri`, `niri-session` `niri-login-root` and `niri-login` must be accessible from path

# How it works

The `niri-login/root` program will start as a `systemd` unit and:
- Create a `UnixSocket` with READ permissions for `root` and WRITE permissions for `nirilogin`
- Run the `niri-login/login` app on a `niri` TTY screen
- Waits for a pair of `(uid, passwd)` on the socket
- Run PAM to check the correctness of the login
- Run `niri-session` on the provided user
