use crate::conversation::OwnedConversation;
use crate::environ::{PamEnv, PamEnvMut};
use crate::items::{PamItems, PamItemsMut};
use crate::pam::constants::{ErrorCode, RawFlags, Result, ReturnCode};
use crate::{AuthnFlags, AuthtokFlags, conversation::Conversation};
use crate::{BaseFlags, CredAction, pam};

use core::cell::Cell;
use core::ptr::NonNull;
use core::{any, fmt, ptr};

use std::ffi::{CString, OsStr, OsString, c_char, c_int};
use std::os::unix::ffi::OsStrExt;

pub struct Pam<C: Conversation> {
    handle: *mut pam::pam_handle,
    last_return: Cell<Result<()>>,
    conversation: Box<OwnedConversation<C>>,
}

impl<C: Conversation> fmt::Debug for Pam<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("handle", &format!("{:p}", self.handle))
            .field("last_return", &self.last_return.get())
            .field("conversation", &format!("{:p}", self.conversation))
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub struct PamBuilder {
    service_name: OsString,
    username: Option<OsString>,
}

impl PamBuilder {
    pub fn new(service_name: impl AsRef<OsStr>) -> Self {
        Self {
            service_name: service_name.as_ref().into(),
            username: None,
        }
    }

    pub fn username(mut self, username: impl AsRef<OsStr>) -> Self {
        self.username = Some(username.as_ref().into());
        self
    }

    pub fn build<C: Conversation>(self, conv: C) -> Result<Pam<C>> {
        Pam::start(self.service_name, self.username, conv)
    }
}

impl<C: Conversation> Pam<C> {
    fn start(service_name: OsString, username: Option<OsString>, conversation: C) -> Result<Self> {
        let mut conv = Box::new(OwnedConversation::new(conversation));
        let service_cstr = CString::new(service_name.as_bytes()).expect("null is forbidden");
        let username_cstr = crate::helper::option_cstr_os(username.as_deref());
        let username_cstr = crate::helper::prompt_ptr(username_cstr.as_deref());

        let mut handle: *mut pam::pam_handle = ptr::null_mut();
        let conv_ptr: *mut OwnedConversation<_> = conv.as_mut() as _;

        let result = unsafe {
            pam::pam_start(
                service_cstr.as_ptr(),
                username_cstr,
                conv_ptr.cast(),
                &mut handle,
            )
        };

        ErrorCode::result_from(result)?;

        Ok(Self {
            handle: NonNull::new(handle)
                .ok_or(ErrorCode::BufferError)?
                .as_ptr()
                .cast(),
            last_return: Cell::new(Ok(())),
            conversation: conv,
        })
    }
}

impl<C: Conversation> Drop for Pam<C> {
    fn drop(&mut self) {
        self.end(Ok(()))
    }
}

fn split<T>(result: &Result<T>) -> Result<()> {
    result.as_ref().map(drop).map_err(|&e| e)
}

impl<C: Conversation> Pam<C> {
    pub fn authenticate(&mut self, flags: AuthnFlags) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_authenticate(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn account_management(&mut self, flags: AuthnFlags) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_acct_mgmt(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn change_authtok(&mut self, flags: AuthtokFlags) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_chauthtok(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn open_session(&mut self, flags: BaseFlags) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_open_session(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn close_session(&mut self, flags: BaseFlags) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_close_session(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn setcred(&mut self, flags: CredAction) -> Result<()> {
        let result = {
            let flags: RawFlags = flags.into();
            ErrorCode::result_from(unsafe { pam::pam_setcred(self.handle, flags.into()) })
        };
        self.last_return.set(split(&result));
        result
    }

    pub fn end(&mut self, result: Result<()>) -> () {
        let code: ReturnCode = result.into();
        unsafe { pam::pam_end(self.handle, code.into()) };
    }

    pub fn end_silent(&mut self, result: Result<()>) -> () {
        let result: c_int = ReturnCode::from(result).into();
        let result = result | pam::PAM_DATA_SILENT;
        unsafe {
            pam::pam_end(self.handle, result);
        }
    }

    pub fn username(&mut self, prompt: Option<&OsStr>) -> Result<OsString> {
        let prompt = crate::helper::option_cstr_os(prompt);
        let mut output: *const c_char = ptr::null();
        let ret = unsafe {
            pam::pam_get_user(
                self.handle,
                &mut output,
                crate::helper::prompt_ptr(prompt.as_deref()),
            )
        };
        ErrorCode::result_from(ret)?;
        Ok(unsafe { crate::helper::copy_pam_string(output).ok_or(ErrorCode::ConversationError)? })
    }

    pub fn environ(&self) -> PamEnv<'_> {
        PamEnv::new(unsafe { &*self.handle })
    }

    pub fn environ_mut(&mut self) -> PamEnvMut<'_> {
        PamEnvMut::new(unsafe { &mut *self.handle })
    }

    pub fn items(&self) -> PamItems<'_> {
        PamItems(unsafe { &*self.handle })
    }

    pub fn items_mut(&mut self) -> PamItemsMut<'_> {
        PamItemsMut(unsafe { &mut *self.handle })
    }
}
