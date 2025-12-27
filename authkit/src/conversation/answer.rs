use crate::conversation::OwnedExchange;
use crate::pam;
use crate::{ErrorCode, Result};

use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use core::{iter, ptr, slice};

use std::ffi::{OsStr, c_int, c_void};
use std::os::unix::ffi::OsStrExt;

#[derive(Debug)]
pub struct Answers {
    base: NonNull<Answer>,
    count: usize,
}

impl Answers {
    pub fn build(value: Vec<OwnedExchange>) -> Result<Self> {
        let mut outputs = Self {
            base: crate::helper::calloc(value.len()),
            count: value.len(),
        };

        for (input, output) in iter::zip(value, outputs.as_mut_slice().iter_mut()) {
            match input {
                OwnedExchange::MaskedPrompt(p) => TextAnswer::fill(output, &p.answer()?)?,
                OwnedExchange::Prompt(p) => TextAnswer::fill(output, &p.answer()?)?,
                OwnedExchange::Error(p) => {
                    TextAnswer::fill(output, p.answer().map(|_| "".as_ref())?)?
                }
                OwnedExchange::Info(p) => {
                    TextAnswer::fill(output, p.answer().map(|_| "".as_ref())?)?
                }

                OwnedExchange::RadioPrompt(p) => TextAnswer::fill(output, &(p.answer()?))?,
                OwnedExchange::BinaryPrompt(p) => {
                    BinaryAnswer::fill(output, (&p.answer()?).into())?
                }
            }
        }
        Ok(outputs)
    }

    pub fn into_ptr(self) -> *mut pam::pam_response {
        ManuallyDrop::new(self).base.as_ptr().cast()
    }

    fn as_mut_slice(&mut self) -> &mut [Answer] {
        unsafe { slice::from_raw_parts_mut(self.base.as_ptr(), self.count) }
    }
}

impl Drop for Answers {
    fn drop(&mut self) {
        unsafe {
            for answer in self.as_mut_slice().iter_mut() {
                ptr::drop_in_place(answer)
            }
            crate::helper::free(self.base.as_ptr())
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct Answer {
    pub data: Option<crate::helper::CHeapBox<c_void>>,

    return_code: c_int,
    _marker: crate::helper::Immovable,
}

#[repr(transparent)]
#[derive(Debug)]
struct TextAnswer(Answer);

impl TextAnswer {
    fn fill(dest: &mut Answer, text: &OsStr) -> Result<()> {
        let allocated = crate::helper::CHeapString::new(text.as_bytes());
        let _ = dest
            .data
            .replace(unsafe { crate::helper::CHeapBox::cast(allocated.into_box()) });
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct BinaryAnswer(Answer);

impl BinaryAnswer {
    fn fill(dest: &mut Answer, (data, type_): (&[u8], u8)) -> Result<()> {
        let payload =
            crate::helper::CHeapPayload::new(data, type_).map_err(|_| ErrorCode::BufferError)?;
        let _ = dest
            .data
            .replace(unsafe { crate::helper::CHeapBox::cast(payload.into_inner()) });
        Ok(())
    }
}
