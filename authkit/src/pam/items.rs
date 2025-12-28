use crate::pam::{
    self,
    constants::{self, ErrorCode, Result},
    pam_handle,
};

use core::ptr;

use std::ffi::{OsStr, OsString, c_int};

crate::pam::helper::num_enum! {
    #[non_exhaustive]
    pub enum ItemType {
        Service = constants::PAM_SERVICE,
        User = constants::PAM_USER,
        Tty = constants::PAM_TTY,
        RemoteHost = constants::PAM_RHOST,
        Conversation = constants::PAM_CONV,
        AuthTok = constants::PAM_AUTHTOK,
        OldAuthTok = constants::PAM_OLDAUTHTOK,
        RemoteUser = constants::PAM_RUSER,
        UserPrompt = constants::PAM_USER_PROMPT,
    }
}

pub struct PamItems<'a>(pub(crate) &'a pam_handle);
pub struct PamItemsMut<'a>(pub(crate) &'a mut pam_handle);

macro_rules! cstr_item {
    (get = $getter:ident, item = $item_type:path) => {
        pub fn $getter(&self) -> Result<Option<OsString>> {
            unsafe { get_cstr_item(&self.0, $item_type) }
        }
    };
    (set = $setter:ident, item = $item_type:path) => {
        pub fn $setter(&mut self, value: Option<&OsStr>) -> Result<()> {
            unsafe { set_cstr_item(&mut self.0, $item_type, value) }
        }
    };
}

impl PamItems<'_> {
    cstr_item!(get = user, item = ItemType::User);
    cstr_item!(get = service, item = ItemType::Service);
    cstr_item!(get = user_prompt, item = ItemType::UserPrompt);
    cstr_item!(get = tty_name, item = ItemType::Tty);
    cstr_item!(get = remote_user, item = ItemType::RemoteUser);
    cstr_item!(get = remote_host, item = ItemType::RemoteHost);
}

impl PamItemsMut<'_> {
    cstr_item!(get = user, item = ItemType::User);
    cstr_item!(get = service, item = ItemType::Service);
    cstr_item!(get = user_prompt, item = ItemType::UserPrompt);
    cstr_item!(get = tty_name, item = ItemType::Tty);
    cstr_item!(get = remote_user, item = ItemType::RemoteUser);
    cstr_item!(get = remote_host, item = ItemType::RemoteHost);
}

impl PamItemsMut<'_> {
    cstr_item!(set = set_user, item = ItemType::User);
    cstr_item!(set = set_service, item = ItemType::Service);
    cstr_item!(set = set_user_prompt, item = ItemType::UserPrompt);
    cstr_item!(set = set_tty_name, item = ItemType::Tty);
    cstr_item!(set = set_remote_user, item = ItemType::RemoteUser);
    cstr_item!(set = set_remote_host, item = ItemType::RemoteHost);
    cstr_item!(set = set_authtok, item = ItemType::AuthTok);
    cstr_item!(set = set_old_authtok, item = ItemType::OldAuthTok);
}

pub unsafe fn get_cstr_item(hdl: &pam_handle, item_type: ItemType) -> Result<Option<OsString>> {
    unsafe {
        let mut output = ptr::null();
        let ret = pam::pam_get_item(hdl, item_type as c_int, &mut output);
        ErrorCode::result_from(ret)?;
        Ok(crate::pam::helper::copy_pam_string(output.cast()))
    }
}

pub unsafe fn set_cstr_item(
    hdl: &mut pam_handle,
    item_type: ItemType,
    data: Option<&OsStr>,
) -> Result<()> {
    let data_str = crate::pam::helper::option_cstr_os(data);
    let ret = unsafe {
        pam::pam_set_item(
            hdl,
            item_type as c_int,
            crate::pam::helper::prompt_ptr(data_str.as_deref()).cast(),
        )
    };
    ErrorCode::result_from(ret)
}
