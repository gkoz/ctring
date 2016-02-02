#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate libc;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

use libc::c_char;
use std::ffi::CString;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::str;

pub use std::ffi::NulError;
pub use std::str::Utf8Error;

pub struct Ctring {
    inner: Box<[u8]>,
}

impl Ctring {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Ctring, NulError> {
        CString::new(t).map(|s| {
            Ctring { inner: s.into_bytes_with_nul().into_boxed_slice() }
        })
    }
}

impl Deref for Ctring {
    type Target = Ctr;
    fn deref(&self) -> &Ctr {
        unsafe { mem::transmute(&*self.inner) }
    }
}

impl DerefMut for Ctring {
    fn deref_mut(&mut self) -> &mut Ctr {
        unsafe { mem::transmute(&mut *self.inner) }
    }
}

pub struct Ctr {
    inner: [c_char],
}

impl Ctr {
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> Result<&'a Ctr, Utf8Error> {
        let len = libc::strlen(ptr);
        let slice = slice::from_raw_parts(ptr, len as usize + 1);
        str::from_utf8(mem::transmute(slice)).map(|_| mem::transmute(slice))
    }

    pub fn from_nul_terminated_str(s: &str) -> &Ctr {
        unsafe {
            assert!(s.as_bytes().iter().position(|&b| b == 0) == Some(s.len() - 1));
            mem::transmute(s)
        }
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }
}

impl Deref for Ctr {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { mem::transmute(&self.inner[..self.inner.len() - 1]) }
    }
}

#[macro_export]
macro_rules! ctring_const {
    ($($name:ident => $value:tt,)+) => {
        lazy_static! {
            $(
                static ref $name: &'static $crate::Ctr =
                    $crate::Ctr::from_nul_terminated_str(concat!($value, "\0"));
            )+
        }
    }
}

#[cfg(test)]
mod test {
    use libc;

    ctring_const! {
        BAR => "AB",
        FOO => "CD",
    }

    #[test]
    fn it_works() {
        unsafe {
            libc::puts(BAR.as_ptr());
            libc::puts(FOO.as_ptr());
        }
    }
}
