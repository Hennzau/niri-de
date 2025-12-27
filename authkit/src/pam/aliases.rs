use crate::pam::ffi::{pam_message, pam_response};

use core::ffi::{c_int, c_void};

pub(crate) type ConversationCallback = unsafe extern "C" fn(
    num_msg: c_int,
    msg: *const *const pam_message,
    resp: *mut *mut pam_response,
    appdata: *mut c_void,
) -> c_int;
