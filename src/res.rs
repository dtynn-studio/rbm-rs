use std::borrow::Cow;
use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    NotEnoughData {
        want: usize,
        got: usize,
        msg: Option<Cow<'static, str>>,
    },
    NotOK {
        code: RetCode,
        errcode: Option<u16>,
        msg: Option<Cow<'static, str>>,
    },
    InvalidData(Cow<'static, str>),
    Other(Cow<'static, str>),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

macro_rules! ensure_buf_size {
    ($buf:expr, $size:expr) => {
        if $buf.len() < $size {
            return Err($crate::Error::NotEnoughData {
                want: $size,
                got: $buf.len(),
                msg: None,
            });
        }
    };

    ($buf:expr, $size:expr, $msg:expr) => {
        if $buf.len() < $size {
            return Err($crate::Error::NotEnoughData {
                want: $size,
                got: $buf.len(),
                msg: Some($msg.into()),
            });
        }
    };
}

pub(crate) use ensure_buf_size;

#[derive(Debug, Default, Clone, Copy)]
pub struct RetCode(pub u8);

impl RetCode {
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }
}

impl From<u8> for RetCode {
    fn from(code: u8) -> Self {
        RetCode(code)
    }
}

macro_rules! ensure_ok {
    (sized: $buf:expr) => {{
        let retcode = $crate::RetCode::from($buf[0]);
        if !retcode.is_ok() {
            let errcode = if $buf.len() >= 3 {
                Some(u16::from_le_bytes([$buf[1], $buf[2]]))
            } else {
                None
            };
            return Err($crate::Error::NotOK {
                code: retcode,
                errcode,
                msg: None,
            });
        }

        retcode
    }};

    (sized: $buf:expr, $msg:expr) => {{
        let retcode = $crate::RetCode::from($buf[0]);
        if !retcode.is_ok() {
            let errcode = if $buf.len() >= 3 {
                Some(u16::from_le_bytes([$buf[1], $buf[2]]))
            } else {
                None
            };
            return Err($crate::Error::NotOK {
                code: retcode,
                errcode,
                msg: Some($msg.into()),
            });
        }

        retcode
    }};

    ($buf:expr) => {
        $crate::ensure_buf_size!($buf, 1);
        ensure_ok!(sized: $buf)
    };

    ($buf:expr, $msg:expr) => {
        $crate::ensure_buf_size!($buf, 1);
        ensure_ok!(sized: $buf, $msg)
    };
}

pub(crate) use ensure_ok;
