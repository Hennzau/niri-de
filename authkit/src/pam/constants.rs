#![allow(non_camel_case_types)]
#![allow(dead_code)]

use crate::pam;

use bitflags::bitflags;

use core::error::Error;
use core::ffi::c_int;
use core::fmt;

macro_rules! define {
    ($(#[$attr:meta])* $($name:ident = $value:expr);+$(;)?) => {
        define!(
            @meta { $(#[$attr])* }
            $(pub(crate) const $name: i32 = $value;)+
        );
    };
    (@meta $m:tt $($i:item)+) => { define!(@expand $($m $i)+); };
    (@expand $({ $(#[$m:meta])* } $i:item)+) => {$($(#[$m])* $i)+};
}

macro_rules! c_enum {
    ($(#[$attr:meta])* $($name:ident $(= $value:expr)?,)*) => {
        c_enum!(
            (0)
            $(#[$attr])*
            $($name $(= $value)?,)*
        );
    };
    (($n:expr) $(#[$attr:meta])* $name:ident, $($rest:ident $(= $rv:expr)?,)*) => {
        $(#[$attr])* pub(crate) const $name: i32 = $n;
        c_enum!(($n + 1) $(#[$attr])* $($rest $(= $rv)?,)*);
    };
    (($n:expr) $(#[$attr:meta])* $name:ident = $value:expr, $($rest:ident $(= $rv:expr)?,)*) => {
        $(#[$attr])* pub(crate) const $name: i32 = $value;
        c_enum!(($value + 1) $(#[$attr])* $($rest $(= $rv)?,)*);
    };
    (($n:expr) $(#[$attr:meta])*) => {};
}

c_enum!(
    PAM_SERVICE = 1,
    PAM_USER,
    PAM_TTY,
    PAM_RHOST,
    PAM_CONV,
    PAM_AUTHTOK,
    PAM_OLDAUTHTOK,
    PAM_RUSER,
    PAM_USER_PROMPT,
);

c_enum!(
    PAM_PROMPT_ECHO_OFF = 1,
    PAM_PROMPT_ECHO_ON,
    PAM_ERROR_MSG,
    PAM_TEXT_INFO,
);

pub(crate) const PAM_DISALLOW_NULL_AUTHTOK: i32 = 0x1;

c_enum!(
    PAM_OPEN_ERR = 1,
    PAM_SYMBOL_ERR,
    PAM_SERVICE_ERR,
    PAM_SYSTEM_ERR,
    PAM_BUF_ERR,
    PAM_PERM_DENIED,
    PAM_AUTH_ERR,
    PAM_CRED_INSUFFICIENT,
    PAM_AUTHINFO_UNAVAIL,
    PAM_USER_UNKNOWN,
    PAM_MAXTRIES,
    PAM_NEW_AUTHTOK_REQD,
    PAM_ACCT_EXPIRED,
    PAM_SESSION_ERR,
    PAM_CRED_UNAVAIL,
    PAM_CRED_EXPIRED,
    PAM_CRED_ERR,
    PAM_NO_MODULE_DATA,
    PAM_CONV_ERR,
    PAM_AUTHTOK_ERR,
    PAM_AUTHTOK_RECOVERY_ERR,
    PAM_AUTHTOK_LOCK_BUSY,
    PAM_AUTHTOK_DISABLE_AGING,
    PAM_TRY_AGAIN,
    PAM_IGNORE,
    PAM_ABORT,
    PAM_AUTHTOK_EXPIRED,
    PAM_MODULE_UNKNOWN,
    PAM_BAD_ITEM,
    PAM_CONV_AGAIN,
    PAM_INCOMPLETE,
    _PAM_RETURN_VALUES,
);

define!(
    PAM_SILENT = 0x8000;
    PAM_ESTABLISH_CRED = 0x0002;
    PAM_DELETE_CRED = 0x0004;
    PAM_REINITIALIZE_CRED = 0x0008;
    PAM_REFRESH_CRED = 0x0010;

    PAM_CHANGE_EXPIRED_AUTHTOK = 0x0020;

    PAM_PRELIM_CHECK = 0x4000;
    PAM_UPDATE_AUTHTOK = 0x2000;
    PAM_DATA_REPLACE = 0x20000000;
);

c_enum!(
    PAM_FAIL_DELAY = 10,
    PAM_XDISPLAY,
    PAM_XAUTHDATA,
    PAM_AUTHTOK_TYPE,
);

pub(crate) const PAM_DATA_SILENT: i32 = 0x40000000;

define!(
    PAM_RADIO_TYPE = 5;
    PAM_BINARY_PROMPT = 7;
);

pub(crate) const PAM_MODUTIL_NGROUPS: i32 = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum pam_modutil_redirect_fd {
    PAM_MODUTIL_IGNORE_FD,
    PAM_MODUTIL_PIPE_FD,
    PAM_MODUTIL_NULL_FD,
}

impl From<pam_modutil_redirect_fd> for i32 {
    fn from(value: pam_modutil_redirect_fd) -> Self {
        value as Self
    }
}

impl TryFrom<i32> for pam_modutil_redirect_fd {
    type Error = i32;
    fn try_from(value: i32) -> core::result::Result<Self, Self::Error> {
        match value {
            0..=2 => Ok(unsafe { *(&value as *const i32).cast() }),
            other => Err(other),
        }
    }
}

macro_rules! wrapper {
    (
        $(#[$m:meta])*
        $viz:vis $name:ident($wraps:ty);
    ) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        $viz struct $name($wraps);

        impl From<$wraps> for $name {
            fn from(value: $wraps) -> Self {
                Self(value)
            }
        }
        impl From<$name> for $wraps {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    }
}

wrapper! {
    pub RawFlags(c_int);
}
wrapper! {
    pub ReturnCode(c_int);
}

impl ReturnCode {
    pub const SUCCESS: Self = Self(0);
}

macro_rules! pam_flags {
    (
        $(#[$m:meta])*
        $name:ident {
            $(
                $(#[$m_ident:ident $($m_arg:tt)*])*
                const $item_name:ident = (link = $value_value:expr, else = $other_value:expr);
            )*
        }
    ) => {
        bitflags! {
            #[derive(Clone, Copy, Debug, Default, PartialEq)]
            $(#[$m])*
            pub struct $name: u16 {
                $(
                    $(#[$m_ident $($m_arg)*])*
                    const $item_name = $other_value;
                )*
            }
        }

        impl From<RawFlags> for $name {
            #[allow(unused_doc_comments)]
            fn from(value: RawFlags) -> Self {
                let value: c_int = value.into();
                let result = Self::empty();
                $(
                    $(#[$m_ident $($m_arg)*])*
                    let result = result | if value & $value_value == 0 {
                        Self::empty()
                    } else {
                        Self::$item_name
                    };
                )*
                result
            }
        }

        impl From<$name> for RawFlags {
            #[allow(unused_doc_comments)]
            fn from(value: $name) -> Self {
                let result = 0;
                $(
                    $(#[$m_ident $($m_arg)*])*
                    let result = result | if value.contains($name::$item_name) {
                        $value_value
                    } else {
                        0
                    };
                )*
                Self(result)
            }
        }
    }
}

pam_flags! {
    AuthnFlags {
        const SILENT = (link = PAM_SILENT, else = 0x8000);
        const DISALLOW_NULL_AUTHTOK = (link = PAM_DISALLOW_NULL_AUTHTOK, else = 0b1);
    }
}

pam_flags! {
    AuthtokFlags {
        const SILENT = (link = PAM_SILENT, else = 0x8000);
        const CHANGE_EXPIRED_AUTHTOK = (link = PAM_CHANGE_EXPIRED_AUTHTOK, else = 0b10);
    }
}

pam_flags! {
    BaseFlags {
        const SILENT = (link = PAM_SILENT, else = 0x8000);
    }
}

macro_rules! flag_enum {
    (
        $(#[$m:meta])*
        $name:ident {
            $(
                $(#[$item_m:meta])*
                $item_name:ident = $item_value:path,
            )*
        }
    ) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum $name {
            $(
                $(#[$item_m])*
                $item_name,
            )*
        }

        impl TryFrom<RawFlags> for $name {
            type Error = ErrorCode;
            fn try_from(value: RawFlags) -> Result<$name> {
                match value.0 {
                    $(
                        $item_value => Ok(Self::$item_name),
                    )*
                    _ => Err(ErrorCode::BAD_CONST),
                }
            }
        }

        impl From<$name> for RawFlags {
            fn from(value: $name) -> Self {
                match value {
                    $(
                        $name::$item_name => $item_value.into(),
                    )*
                }
            }
        }
    }
}

flag_enum! {
    #[derive(Default)]
    CredAction {
        #[default]
        Establish = PAM_ESTABLISH_CRED,
        Delete = PAM_DELETE_CRED,
        Reinitialize = PAM_REINITIALIZE_CRED,
        Refresh = PAM_REFRESH_CRED,
    }
}

flag_enum! {
    AuthtokAction {
        Validate = PAM_PRELIM_CHECK,
        Update = PAM_UPDATE_AUTHTOK,
    }
}

macro_rules! linky_enum {
    (
        $(#[$om:meta])*
        pub enum $name:ident($wrap:ty) {
            $(
                $(#[$im:meta])*
                $key:ident = $value:path,
            )*
        }
    ) => {
        $(#[$om])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $(#[$im])*
                $key,
            )*
        }

        impl TryFrom<$wrap> for $name {
            type Error = ErrorCode;
            fn try_from(value: $wrap) -> Result<Self> {
                match value.into() {
                    $(
                        $(#[$im])*
                        $value => Ok(Self::$key),
                    )*
                    _ => Err(ErrorCode::BAD_CONST),
                }
            }
        }

        impl From<$name> for $wrap {
            fn from(value: $name) -> Self {
                match value {
                    $(
                        $(#[$im])*
                        $name::$key => $value.into(),
                    )*
                }
            }
        }
    }
}

linky_enum! {
    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    pub enum ErrorCode(ReturnCode) {
        OpenError = PAM_OPEN_ERR,
        SymbolError = PAM_SYMBOL_ERR,
        ServiceError = PAM_SERVICE_ERR,
        SystemError = PAM_SYSTEM_ERR,
        BufferError = PAM_BUF_ERR,
        PermissionDenied = PAM_PERM_DENIED,
        AuthenticationError = PAM_AUTH_ERR,
        CredentialsInsufficient = PAM_CRED_INSUFFICIENT,
        AuthInfoUnavailable = PAM_AUTHINFO_UNAVAIL,
        UserUnknown = PAM_USER_UNKNOWN,
        MaxTries = PAM_MAXTRIES,
        NewAuthTokRequired = PAM_NEW_AUTHTOK_REQD,
        AccountExpired = PAM_ACCT_EXPIRED,
        SessionError = PAM_SESSION_ERR,
        CredentialsUnavailable = PAM_CRED_UNAVAIL,
        CredentialsExpired = PAM_CRED_EXPIRED,
        CredentialsError = PAM_CRED_ERR,
        NoModuleData = PAM_NO_MODULE_DATA,
        ConversationError = PAM_CONV_ERR,
        AuthTokError = PAM_AUTHTOK_ERR,
        AuthTokRecoveryError = PAM_AUTHTOK_RECOVERY_ERR,
        AuthTokLockBusy = PAM_AUTHTOK_LOCK_BUSY,
        AuthTokDisableAging = PAM_AUTHTOK_DISABLE_AGING,
        TryAgain = PAM_TRY_AGAIN,
        Ignore = PAM_IGNORE,
        Abort = PAM_ABORT,
        AuthTokExpired = PAM_AUTHTOK_EXPIRED,
    }
}

pub type Result<T> = core::result::Result<T, ErrorCode>;

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::ffi::CStr;
        use std::ptr;
        let retcode: ReturnCode = (*self).into();

        let got = unsafe { pam::pam_strerror(ptr::null(), retcode.into()) };
        if got.is_null() {
            write!(f, "PAM error: {self:?} ({:?})", retcode)
        } else {
            f.write_str(&unsafe { CStr::from_ptr(got) }.to_string_lossy())
        }
    }
}

impl Error for ErrorCode {}

impl ErrorCode {
    pub const BAD_CONST: ErrorCode = ErrorCode::SystemError;

    pub(crate) fn result_from(ret: c_int) -> Result<()> {
        match ret {
            0 => Ok(()),
            value => Err(ReturnCode(value).try_into().unwrap_or(Self::BAD_CONST)),
        }
    }
}

impl<T> From<Result<T>> for ReturnCode {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(_) => ReturnCode::SUCCESS,
            Err(otherwise) => otherwise.into(),
        }
    }
}
