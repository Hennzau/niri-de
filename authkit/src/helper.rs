use core::error::Error;
use core::marker::{PhantomData, PhantomPinned};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::NonNull;
use core::{any, fmt, mem, slice};

use std::ffi::{CStr, CString, OsStr, OsString, c_char};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

#[allow(clippy::missing_safety_doc)]
pub(crate) unsafe fn iter_over<'a, T, Src>(
    ptr_ptr: *const *const Src,
    count: usize,
) -> impl Iterator<Item = &'a T>
where
    T: 'a,
{
    unsafe {
        assert_eq!(
            mem::size_of::<T>(),
            mem::size_of::<Src>(),
            "type {t} is not the size of {that}",
            t = any::type_name::<T>(),
            that = any::type_name::<Src>(),
        );
        slice::from_raw_parts(ptr_ptr.cast::<&T>(), count)
            .iter()
            .copied()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct TooBigError {
    pub(crate) size: usize,
    pub(crate) max: usize,
}

impl Error for TooBigError {}

impl fmt::Display for TooBigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "can't allocate a message of {size} bytes (max {max})",
            size = self.size,
            max = self.max
        )
    }
}

#[allow(clippy::wrong_self_convention)]
pub(crate) trait Buffer {
    fn allocate(len: usize) -> Self;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn as_mut_slice(this: &mut Self, len: usize) -> &mut [u8];
}

impl Buffer for Vec<u8> {
    fn allocate(bytes: usize) -> Self {
        vec![0; bytes]
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn as_mut_slice(this: &mut Self, bytes: usize) -> &mut [u8] {
        &mut this[..bytes]
    }
}

pub(crate) struct BinaryPayload {
    pub(crate) total_bytes_u32be: [u8; 4],
    pub(crate) data_type: u8,
    pub(crate) _marker: PhantomData<PhantomPinned>,
}

impl BinaryPayload {
    pub(crate) const MAX_SIZE: usize = (u32::MAX - 5) as usize;

    pub(crate) fn fill(buf: &mut [u8], data: &[u8], data_type: u8) {
        let ptr: *mut Self = buf.as_mut_ptr().cast();

        let me = unsafe { ptr.as_mut().unwrap_unchecked() };
        me.total_bytes_u32be = u32::to_be_bytes(buf.len() as u32);
        me.data_type = data_type;
        buf[5..].copy_from_slice(data)
    }
}

#[derive(Debug)]
pub(crate) struct OwnedBinaryPayload<Owner: Buffer>(Owner);

impl<O: Buffer> OwnedBinaryPayload<O> {
    pub(crate) fn new(data: &[u8], type_: u8) -> Result<Self, TooBigError> {
        let total_len: u32 = (data.len() + 5).try_into().map_err(|_| TooBigError {
            size: data.len(),
            max: BinaryPayload::MAX_SIZE,
        })?;
        let total_len = total_len as usize;
        let mut buf = O::allocate(total_len);

        BinaryPayload::fill(
            unsafe { Buffer::as_mut_slice(&mut buf, total_len) },
            data,
            type_,
        );
        Ok(Self(buf))
    }

    pub(crate) fn into_inner(self) -> O {
        self.0
    }
}

macro_rules! num_enum {
    (
        $(#[$m:meta])*
        $viz:vis enum $name:ident {
            $(
                $(#[$im:meta])*
                $item_name:ident = $item_value:path,
            )*
        }
    ) => {


        $(#[$m])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        #[repr(i32)]
        $viz enum $name {
            $(
                $(#[$im])*
                $item_name = $item_value,
            )*
        }

        impl TryFrom<c_int> for $name {
            type Error = crate::pam::constants::ErrorCode;

            #[allow(unused_doc_comments)]
            fn try_from(value: c_int) -> crate::pam::constants::Result<$name> {
                match value {
                    $(
                        $(#[$im])*
                        $item_value => Ok(Self::$item_name),
                    )*
                    _ => Err(crate::pam::constants::ErrorCode::BAD_CONST),
                }
            }
        }

        impl From<$name> for c_int {
            fn from(value: $name) -> c_int {
                value as c_int
            }
        }
    }
}

pub(crate) use num_enum;

#[inline]
pub fn calloc<T>(count: usize) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(libc::calloc(count, mem::size_of::<T>()).cast()) }
}

#[inline]
pub unsafe fn free<T>(p: *mut T) {
    unsafe { libc::free(p.cast()) }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Immovable(pub PhantomData<(*mut u8, PhantomPinned)>);

pub fn option_cstr(prompt: Option<&[u8]>) -> Option<CString> {
    prompt.map(|p| CString::new(p).expect("nul is not allowed"))
}

pub fn option_cstr_os(prompt: Option<&OsStr>) -> Option<CString> {
    option_cstr(prompt.map(OsStr::as_bytes))
}

pub fn prompt_ptr(prompt: Option<&CStr>) -> *const c_char {
    match prompt {
        Some(c_str) => c_str.as_ptr(),
        None => ptr::null(),
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CHeapBox<T>(NonNull<T>);

#[allow(clippy::wrong_self_convention)]
impl<T> CHeapBox<T> {
    pub fn new(value: T) -> Self {
        let memory = calloc(1);
        unsafe { ptr::write(memory.as_ptr(), value) }

        Self(memory)
    }

    pub unsafe fn from_ptr(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    pub fn into_ptr(this: Self) -> NonNull<T> {
        ManuallyDrop::new(this).0
    }

    pub fn as_ptr(this: &Self) -> NonNull<T> {
        this.0
    }

    pub unsafe fn cast<R>(this: Self) -> CHeapBox<R> {
        unsafe { mem::transmute(this) }
    }
}

impl<T: Default> Default for CHeapBox<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl crate::helper::Buffer for CHeapBox<u8> {
    fn allocate(len: usize) -> Self {
        unsafe { Self::from_ptr(calloc(len)) }
    }

    unsafe fn as_mut_slice(this: &mut Self, len: usize) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(this.0.as_ptr(), len) }
    }
}

pub type CHeapPayload = crate::helper::OwnedBinaryPayload<CHeapBox<u8>>;

impl<T> Deref for CHeapBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { Self::as_ptr(self).as_ref() }
    }
}

impl<T> DerefMut for CHeapBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Self::as_ptr(self).as_mut() }
    }
}

impl<T> Drop for CHeapBox<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.0.as_ptr();
            ptr::drop_in_place(ptr);
            free(ptr)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CHeapString(CHeapBox<c_char>);

impl CHeapString {
    pub fn new(text: impl AsRef<[u8]>) -> Self {
        let data = text.as_ref();
        if data.contains(&0) {
            panic!("you're not allowed to create a cstring with a nul inside!");
        }

        let data_alloc: NonNull<c_char> = calloc(data.len() + 1);

        unsafe {
            let dest = slice::from_raw_parts_mut(data_alloc.as_ptr().cast(), data.len());
            dest.copy_from_slice(data);
            Self(CHeapBox::from_ptr(data_alloc))
        }
    }

    pub fn into_box(self) -> CHeapBox<c_char> {
        unsafe { mem::transmute(self) }
    }

    pub unsafe fn zero(ptr: NonNull<c_char>) {
        unsafe {
            let cstr = ptr.as_ptr();
            let len = libc::strlen(cstr.cast());
            for x in 0..len {
                ptr::write_volatile(cstr.byte_offset(x as isize), mem::zeroed())
            }
        }
    }
}

impl Drop for CHeapString {
    fn drop(&mut self) {
        unsafe { Self::zero(CHeapBox::as_ptr(&self.0)) }
    }
}

impl Deref for CHeapString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        let ptr = CHeapBox::as_ptr(&self.0).as_ptr();
        unsafe { CStr::from_ptr(ptr) }
    }
}

pub unsafe fn copy_pam_string(result_ptr: *const c_char) -> Option<OsString> {
    unsafe {
        NonNull::new(result_ptr.cast_mut())
            .map(NonNull::as_ptr)
            .map(|p| CStr::from_ptr(p))
            .map(CStr::to_bytes)
            .map(Vec::from)
            .map(OsString::from_vec)
    }
}
