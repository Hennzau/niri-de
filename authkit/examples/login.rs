use authkit::{AuthnFlags, BaseFlags, CredAction, Pam, Result as PamResult};

fn authenticate(service: &str, username: &str, password: &str) -> PamResult<Pam> {
    let mut txn = Pam::start(service.into(), username.into(), password.into())?;

    txn.authenticate(AuthnFlags::empty())?;
    txn.account_management(AuthnFlags::empty())?;

    Ok(txn)
}

fn main() {
    if let Ok(mut txn) = authenticate("greetd-greeter", "greeter", "") {
        println!("Logged IN {:?}", txn.username(None));

        txn.open_session(BaseFlags::empty())
            .expect("Couldn't open session");

        txn.setcred(CredAction::Establish)
            .expect("Couldn't establish credentials");

        for (key, val) in txn.env().iter() {
            println!("\t{}={}", key.to_str().unwrap(), val.to_str().unwrap());
        }

        txn.close_session(BaseFlags::empty())
            .expect("Couldn't close session");
    }
}
