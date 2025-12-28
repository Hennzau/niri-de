use crate::pam::conversation::OwnedExchange;
use crate::pam::{self, Result};

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
            base: crate::pam::helper::calloc(value.len()),
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
            crate::pam::helper::free(self.base.as_ptr())
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct Answer {
    pub data: Option<crate::pam::helper::CHeapBox<c_void>>,

    return_code: c_int,
    _marker: crate::pam::helper::Immovable,
}

#[repr(transparent)]
#[derive(Debug)]
struct TextAnswer(Answer);

impl TextAnswer {
    fn fill(dest: &mut Answer, text: &OsStr) -> Result<()> {
        let allocated = crate::pam::helper::CHeapString::new(text.as_bytes());
        let _ = dest
            .data
            .replace(unsafe { crate::pam::helper::CHeapBox::cast(allocated.into_box()) });
        Ok(())
    }
}
