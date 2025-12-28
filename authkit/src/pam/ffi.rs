#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use core::fmt;
use core::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub(crate) struct pam_handle {
    _value: (),
    _marker: PhantomData<(PhantomPinned, *mut c_void)>,
}

impl fmt::Debug for pam_handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pam_handle({self:p}")
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct pam_conv {
    pub(crate) conv: unsafe extern "C" fn(
        num_msg: c_int,
        msg: *const *const pam_message,
        resp: *mut *mut pam_response,
        appdata: *mut c_void,
    ) -> c_int,
    pub(crate) appdata_ptr: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct pam_message {
    pub(crate) msg_style: c_int,
    pub(crate) msg: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct pam_response {
    pub(crate) resp: *mut c_char,

    pub(crate) resp_retcode: c_int,
}

#[link(name = "pam")]
unsafe extern "C" {

    pub(crate) fn pam_acct_mgmt(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_authenticate(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_chauthtok(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_close_session(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_end(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_getenv(pamh: *const pam_handle, name: *const c_char) -> *mut c_char;

    pub(crate) fn pam_getenvlist(pamh: *const pam_handle) -> *mut *mut c_char;

    pub(crate) fn pam_get_item(
        pamh: *const pam_handle,
        item_type: c_int,
        item: *mut *const c_void,
    ) -> c_int;

    pub(crate) fn pam_get_user(
        pamh: *mut pam_handle,
        user: *mut *const c_char,
        prompt: *const c_char,
    ) -> c_int;

    pub(crate) fn pam_open_session(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_putenv(pamh: *mut pam_handle, namevalue: *const c_char) -> c_int;

    pub(crate) fn pam_setcred(pamh: *mut pam_handle, flags: c_int) -> c_int;

    pub(crate) fn pam_set_item(
        pamh: *mut pam_handle,
        item_type: c_int,
        item: *const c_void,
    ) -> c_int;

    pub(crate) fn pam_start(
        service: *const c_char,
        user: *const c_char,
        pam_conv: *mut pam_conv,
        pamh: *mut *mut pam_handle,
    ) -> c_int;

    pub(crate) fn pam_strerror(pamh: *const pam_handle, error_number: c_int) -> *mut c_char;
}
