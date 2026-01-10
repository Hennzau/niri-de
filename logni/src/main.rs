use std::io::{Write, stdin, stdout};

fn main() -> std::io::Result<()> {
    print!("login: ");
    stdout().flush()?;

    let mut username = String::new();
    stdin().read_line(&mut username)?;
    let username = username.trim();

    print!("password: ");
    stdout().flush()?;
    let password = rpassword::read_password()?;

    println!("Credentials: {}, {}", username, password);

    Ok(())
}
