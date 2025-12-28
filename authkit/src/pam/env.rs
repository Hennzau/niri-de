use crate::pam::{self, pam_handle};

use core::marker::PhantomData;
use core::ptr;
use core::ptr::NonNull;

use std::ffi::{CStr, CString, OsStr, OsString, c_char};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub struct PamEnv<'a> {
    source: &'a pam_handle,
}

pub struct PamEnvMut<'a> {
    source: &'a mut pam_handle,
}

impl<'a> PamEnv<'a> {
    pub(crate) fn new(source: &'a pam_handle) -> Self {
        Self { source }
    }
}

impl<'a> PamEnvMut<'a> {
    pub(crate) fn new(source: &'a mut pam_handle) -> Self {
        Self { source }
    }
}

impl PamEnv<'_> {
    pub fn get(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
        environ_get(self.source, key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OsString, OsString)> {
        environ_iter(self.source)
    }
}
impl PamEnvMut<'_> {
    pub fn get(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
        environ_get(self.source, key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OsString, OsString)> {
        environ_iter(self.source)
    }

    pub fn insert(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> Option<OsString> {
        environ_set(self.source, key.as_ref(), Some(val.as_ref()))
    }

    pub fn remove(&mut self, key: impl AsRef<OsStr>) -> Option<OsString> {
        environ_set(self.source, key.as_ref(), None)
    }
}

struct EnvList<'a> {
    start: NonNull<Option<EnvVar>>,

    current: NonNull<Option<EnvVar>>,
    _owner: PhantomData<&'a pam_handle>,
}

impl EnvList<'_> {
    fn empty() -> Self {
        let none: crate::pam::helper::CHeapBox<Option<EnvVar>> =
            crate::pam::helper::CHeapBox::new(None);
        let ptr = crate::pam::helper::CHeapBox::into_ptr(none);
        Self {
            start: ptr,
            current: ptr,
            _owner: PhantomData,
        }
    }
    unsafe fn from_ptr(ptr: NonNull<*mut c_char>) -> Self {
        Self {
            start: ptr.cast(),
            current: ptr.cast(),
            _owner: Default::default(),
        }
    }
}

impl Iterator for EnvList<'_> {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.current.as_mut() } {
            None => None,
            Some(item) => {
                let ret = item.as_kv();

                unsafe {
                    self.current = advance(self.current);
                    ptr::drop_in_place(item as *mut EnvVar);
                }
                Some(ret)
            }
        }
    }
}

impl Drop for EnvList<'_> {
    fn drop(&mut self) {
        unsafe {
            while let Some(var_ref) = self.current.as_mut() {
                self.current = advance(self.current);
                ptr::drop_in_place(var_ref as *mut EnvVar);
            }
            crate::pam::helper::free(self.start.as_ptr())
        }
    }
}

unsafe fn advance<T>(nn: NonNull<T>) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(nn.as_ptr().offset(1)) }
}

struct EnvVar(crate::pam::helper::CHeapString);

impl EnvVar {
    fn as_kv(&self) -> (OsString, OsString) {
        let bytes = self.0.to_bytes();
        let mut split = bytes.splitn(2, |&b| b == b'=');
        (
            OsString::from_vec(split.next().unwrap_or_default().into()),
            OsString::from_vec(split.next().unwrap_or_default().into()),
        )
    }
}

fn environ_get(pamh: &pam_handle, key: &OsStr) -> Option<OsString> {
    let key = CString::new(key.as_bytes()).ok()?;

    let src = unsafe { pam::pam_getenv(pamh, key.as_ptr()) };
    let val = match NonNull::new(src) {
        None => return None,
        Some(ptr) => ptr.as_ptr(),
    };

    let c_str = unsafe { CStr::from_ptr(val) };
    Some(OsString::from_vec(c_str.to_bytes().to_vec()))
}

fn environ_set(pamh: &mut pam_handle, key: &OsStr, value: Option<&OsStr>) -> Option<OsString> {
    let old = environ_get(pamh, key);
    if old.is_none() && value.is_none() {
        return None;
    }
    let total_len = key.len() + value.map(OsStr::len).unwrap_or_default() + 2;
    let mut result = Vec::with_capacity(total_len);
    result.extend(key.as_bytes());
    if let Some(value) = value {
        result.push(b'=');
        result.extend(value.as_bytes());
    }
    let put = CString::new(result).unwrap();

    let _ = unsafe { pam::pam_putenv(pamh, put.as_ptr()) };
    old
}

fn environ_iter(pamh: &pam_handle) -> impl Iterator<Item = (OsString, OsString)> {
    unsafe {
        NonNull::new(pam::pam_getenvlist(pamh))
            .map(|ptr| EnvList::from_ptr(ptr.cast()))
            .unwrap_or_else(EnvList::empty)
    }
}
