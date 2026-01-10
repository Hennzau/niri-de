pub enum Error {
    UnknownUserHost,
}

pub type Result<T> = core::result::Result<T, Error>;
