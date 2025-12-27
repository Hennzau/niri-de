use crate::ErrorCode;
use crate::Result;
use crate::conversation::OwnedExchange;
use crate::conversation::{ErrorMsg, Exchange, InfoMsg, MaskedQAndA, QAndA};
use crate::pam;

use core::ptr::NonNull;

use std::ffi::{CStr, OsStr, c_int, c_void};
use std::os::unix::ffi::OsStrExt;

crate::helper::num_enum! {
    enum Style {
        PromptEchoOff = pam::PAM_PROMPT_ECHO_OFF,
        PromptEchoOn = pam::PAM_PROMPT_ECHO_ON,
        ErrorMsg = pam::PAM_ERROR_MSG,
        TextInfo = pam::PAM_TEXT_INFO,
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Question {
    pub style: c_int,
    pub data: Option<NonNull<c_void>>,
}

impl Question {
    #[allow(clippy::missing_safety_doc)]
    unsafe fn string_data(&self) -> &OsStr {
        unsafe {
            match self.data.as_ref() {
                None => "".as_ref(),
                Some(data) => OsStr::from_bytes(CStr::from_ptr(data.as_ptr().cast()).to_bytes()),
            }
        }
    }
}

impl TryFrom<&Exchange<'_>> for Question {
    type Error = ErrorCode;

    fn try_from(msg: &Exchange) -> Result<Self> {
        let alloc = |style, text: &OsStr| -> Result<_> {
            Ok((style, unsafe {
                crate::helper::CHeapBox::cast(
                    crate::helper::CHeapString::new(text.as_bytes()).into_box(),
                )
            }))
        };

        let (style, data): (_, crate::helper::CHeapBox<c_void>) = match *msg {
            Exchange::MaskedPrompt(p) => alloc(Style::PromptEchoOff, p.question()),
            Exchange::Prompt(p) => alloc(Style::PromptEchoOn, p.question()),
            Exchange::Error(p) => alloc(Style::ErrorMsg, p.question()),
            Exchange::Info(p) => alloc(Style::TextInfo, p.question()),
            Exchange::RadioPrompt(_) | Exchange::BinaryPrompt(_) => {
                Err(ErrorCode::ConversationError)
            }
        }?;
        Ok(Self {
            style: style.into(),
            data: Some(crate::helper::CHeapBox::into_ptr(data)),
        })
    }
}

impl Drop for Question {
    fn drop(&mut self) {
        unsafe {
            if let Ok(style) = Style::try_from(self.style) {
                let _ = match style {
                    Style::TextInfo
                    | Style::ErrorMsg
                    | Style::PromptEchoOff
                    | Style::PromptEchoOn => self
                        .data
                        .as_mut()
                        .map(|p| crate::helper::CHeapString::zero(p.cast())),
                };
            };
            let _ = self.data.map(|p| crate::helper::CHeapBox::from_ptr(p));
        }
    }
}

impl<'a> TryFrom<&'a Question> for OwnedExchange<'a> {
    type Error = ErrorCode;
    fn try_from(question: &'a Question) -> Result<Self> {
        let style: Style = question
            .style
            .try_into()
            .map_err(|_| ErrorCode::ConversationError)?;

        let prompt = unsafe {
            match style {
                Style::PromptEchoOff => {
                    Self::MaskedPrompt(MaskedQAndA::new(question.string_data()))
                }
                Style::PromptEchoOn => Self::Prompt(QAndA::new(question.string_data())),
                Style::ErrorMsg => Self::Error(ErrorMsg::new(question.string_data())),
                Style::TextInfo => Self::Info(InfoMsg::new(question.string_data())),
            }
        };
        Ok(prompt)
    }
}
