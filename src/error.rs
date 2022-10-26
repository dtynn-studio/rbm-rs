use std::borrow::Cow;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    NotEnoughData {
        want: usize,
        got: usize,
        msg: Option<Cow<'static, str>>,
    },
    NotOK {
        code: u8,
        msg: Option<Cow<'static, str>>,
    },
    InvalidData(Cow<'static, str>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}
