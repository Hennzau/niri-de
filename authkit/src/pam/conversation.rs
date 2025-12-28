use crate::pam;
use crate::pam::ErrorCode;
use crate::pam::Result;
use crate::pam::constants::ReturnCode;

use core::cell::Cell;
use core::ffi::{c_int, c_void};
use core::fmt;
use core::fmt::Debug;

use std::ffi::{OsStr, OsString};

use answer::Answers;
use question::Question;

mod answer;
mod question;

#[derive(Debug)]
pub struct PamConversation {
    username: String,
    password: String,
}

impl PamConversation {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    fn prompt(&self, _: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn error_msg(&self, _: impl AsRef<OsStr>) {}

    fn info_msg(&self, _: impl AsRef<OsStr>) {}

    fn communicate(&self, messages: &[Exchange]) {
        for msg in messages {
            match msg {
                Exchange::Prompt(prompt) => prompt.set_answer(self.prompt(prompt.question())),
                Exchange::MaskedPrompt(prompt) => {
                    prompt.set_answer(self.masked_prompt(prompt.question()))
                }
                Exchange::Info(prompt) => {
                    self.info_msg(prompt.question());
                    prompt.set_answer(Ok(()))
                }
                Exchange::Error(prompt) => {
                    self.error_msg(prompt.question());
                    prompt.set_answer(Ok(()))
                }
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PamOwnedConversation {
    callback: pam::aliases::ConversationCallback,
    conv: Box<PamConversation>,
}

impl PamOwnedConversation {
    pub fn new(conv: PamConversation) -> Self {
        Self {
            callback: Self::wrapper_callback,
            conv: Box::new(conv),
        }
    }

    unsafe extern "C" fn wrapper_callback(
        count: c_int,
        questions: *const *const pam::pam_message,
        answers: *mut *mut pam::pam_response,
        me: *mut c_void,
    ) -> c_int {
        unsafe {
            let internal = || {
                let conv = me
                    .cast::<PamConversation>()
                    .as_ref()
                    .ok_or(ErrorCode::ConversationError)?;
                let q_iter =
                    crate::pam::helper::iter_over::<Question, _>(questions, count as usize);
                let answers_ptr = answers.as_mut().ok_or(ErrorCode::ConversationError)?;

                let messages: Vec<OwnedExchange> = q_iter
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()
                    .map_err(|_| ErrorCode::ConversationError)?;

                let borrowed: Result<Vec<_>> = messages.iter().map(Exchange::try_from).collect();

                conv.communicate(&borrowed?);

                let owned = Answers::build(messages)?;
                *answers_ptr = owned.into_ptr();
                Ok(())
            };
            ReturnCode::from(internal()).into()
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Exchange<'a> {
    Prompt(&'a QAndA<'a>),
    MaskedPrompt(&'a MaskedQAndA<'a>),
    Error(&'a ErrorMsg<'a>),
    Info(&'a InfoMsg<'a>),
}

macro_rules! q_and_a {
    ($(#[$m:meta])* $name:ident<'a, Q=$qt:ty, A=$at:ty>, $val:path) => {
        $(#[$m])*
        pub struct $name<'a> {
            q: $qt,
            a: Cell<Result<$at>>,
        }

        $(#[$m])*
        impl<'a> $name<'a> {
            pub(crate) fn new(question: $qt) -> Self {
                Self {
                    q: question,
                    a: Cell::new(Err(ErrorCode::ConversationError)),
                }
            }

            pub(crate) fn question(&self) -> $qt {
                self.q
            }

            pub(crate) fn set_answer(&self, answer: Result<$at>) {
                self.a.set(answer)
            }

            pub(crate) fn answer(self) -> Result<$at> {
                self.a.into_inner()
            }
        }

        $(#[$m])*
        impl fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::result::Result<(), fmt::Error> {
                f.debug_struct(stringify!($name)).field("q", &self.q).finish_non_exhaustive()
            }
        }
    };
}

q_and_a!(
    MaskedQAndA<'a, Q = &'a OsStr, A = OsString>,
    Exchange::MaskedPrompt
);

q_and_a!(QAndA<'a, Q = &'a OsStr, A = OsString>, Exchange::Prompt);

#[derive(Debug, Default, PartialEq)]
pub struct BinaryData {
    pub(crate) data: Vec<u8>,

    pub(crate) data_type: u8,
}

impl<IV: Into<Vec<u8>>> From<(IV, u8)> for BinaryData {
    fn from((data, data_type): (IV, u8)) -> Self {
        Self {
            data: data.into(),
            data_type,
        }
    }
}

impl From<BinaryData> for (Vec<u8>, u8) {
    fn from(value: BinaryData) -> Self {
        (value.data, value.data_type)
    }
}

impl<'a> From<&'a BinaryData> for (&'a [u8], u8) {
    fn from(value: &'a BinaryData) -> Self {
        (&value.data, value.data_type)
    }
}

q_and_a!(InfoMsg<'a, Q = &'a OsStr, A = ()>, Exchange::Info);

q_and_a!(ErrorMsg<'a, Q = &'a OsStr, A = ()>, Exchange::Error);

#[derive(Debug)]
pub enum OwnedExchange<'a> {
    MaskedPrompt(MaskedQAndA<'a>),
    Prompt(QAndA<'a>),
    Info(InfoMsg<'a>),
    Error(ErrorMsg<'a>),
}

impl<'a> TryFrom<&'a OwnedExchange<'a>> for Exchange<'a> {
    type Error = ErrorCode;
    fn try_from(src: &'a OwnedExchange) -> core::result::Result<Self, ErrorCode> {
        match src {
            OwnedExchange::MaskedPrompt(m) => Ok(Exchange::MaskedPrompt(m)),
            OwnedExchange::Prompt(m) => Ok(Exchange::Prompt(m)),
            OwnedExchange::Info(m) => Ok(Exchange::Info(m)),
            OwnedExchange::Error(m) => Ok(Exchange::Error(m)),
        }
    }
}
