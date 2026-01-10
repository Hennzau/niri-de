use super::{Error, Result};

fn get_current_user() -> Result<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|e| {
            eprintln!("{e}");
            Error::UnknownUserHost
        })
}

pub fn start_display_tty(tty_number: u16) -> Result<()> {
    println!("Starting RILM display in TTY mode on tty{}", tty_number);
    println!("Running as root on tty{}", tty_number);

    todo!(
        r#"
            - Check for a user named 'greeter'. If not present create it and add it to the video group.
            - Open a UnixSocket only readable for root, writable for user "greeter"
            - Open a PAM session for user "greeter"
            - Fork, exec this program again with commands 'rilm start greeter --user "greeter"'
            - ...
            - Wait for it to finish
            - ...
            - End the PAM session
            - Once it's done we read the socket, there should be a pair of (user, cred),
            - Close the socket
            - Open a PAM session for this user, add credentials to it
            - Fork, exec this program again with commands 'rilm start session --user <user>'
            - ...
            - Wait for it to finish
            - ...
            - End the PAM session
            - Once it's done repeat this program by running 'rilm start tty' (or 'rilm start display tty')
            "#
    )
}

pub fn start_display_winit() -> Result<()> {
    let current_user = get_current_user()?;
    println!("Starting RILM display in Winit mode");
    println!("Running in simulated window under user: {}", current_user);

    todo!(
        r#"
            - Open a UnixSocket only readable for <current_user>, writable for user <current_user>
            - Open a PAM session for user <current_user>
            - Fork, exec this program again with commands 'rilm start greeter'
            - ...
            - Wait for it to finish
            - ...
            - End the PAM session
            - Once it's done we read the socket, there should be a pair of (user, cred),
            - Close the socket
            - Open a PAM session for this user, add credentials to it
            - Fork, exec this program again with commands 'rilm start session'
            - ...
            - Wait for it to finish
            - ...
            - End the PAM session
            - Once it's done repeat this program by running 'rilm start winit' (or 'rilm start display tty')
            "#
    )
}

pub fn start_greeter(user: Option<String>) -> Result<()> {
    let current_user = get_current_user()?;

    println!(
        "Starting RILM greeter for user: {}",
        user.unwrap_or(current_user)
    );

    todo!(
        r#"
            < parent process should have filled the correct ENV vars to open a wayland session >

            - if <user> != None, change for this user
            - do an `execv` with niri and the greeter config
            "#
    )
}

pub fn start_session(user: Option<String>) -> Result<()> {
    let current_user = get_current_user()?;

    println!(
        "Starting RILM session for user: {}",
        user.unwrap_or(current_user)
    );

    todo!(
        r#"
            < parent process should have filled the correct ENV vars to open a wayland session >

            - if <user> != None, change for this user
            - do an `execv` with niri and the session config
            "#
    )
}
