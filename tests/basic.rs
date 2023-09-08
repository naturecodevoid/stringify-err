use std::io::{Error, ErrorKind};

fn main() {
    assert_eq!(basic_ffi().unwrap_err(), String::from("test"));
    assert_eq!(Test::err_ffi().unwrap_err(), String::from("test"));
    assert_eq!(Test.err_self_ffi().unwrap_err(), String::from("test"));
}

#[stringify_err::stringify_err]
fn basic() -> Result<(), Error> {
    Err(Error::new(ErrorKind::Other, "test"))
}

struct Test;

impl Test {
    #[stringify_err::stringify_err(Self)]
    pub fn err() -> Result<(), Error> {
        Err(Error::new(ErrorKind::Other, "test"))
    }

    #[stringify_err::stringify_err]
    pub fn err_self(&self) -> Result<(), Error> {
        Err(Error::new(ErrorKind::Other, "test"))
    }
}
