
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::CString;
use std::io;
use std::ptr::null_mut;

use user32::FindWindowExA;
use winapi::windef::HWND;

use super::{Handle, Session};
use super::ipc::*;

pub struct LocalHandle {
    handle: HWND,
}

impl Handle for LocalHandle {
    type Sess = LocalSession;

    fn connect() -> io::Result<Self> {
        unsafe {
            let win_name = CString::new("UIPCMAIN").unwrap();
            let handle = FindWindowExA(
                null_mut(), null_mut(), win_name.as_ptr(), null_mut());
            if handle != null_mut() {
                Ok(LocalHandle { handle: handle })
            } else {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to local FSUIPC: cannot create window handle"))
            }
        }
    }

    fn session(&self) -> LocalSession {
        LocalSession {
            handle: self.handle,
            buffer: io::Cursor::new(Vec::with_capacity(4096))
        }
    }

    fn disconnect(self) {}
}

pub struct LocalSession {
    handle: HWND,
    buffer: io::Cursor<Vec<u8>>,
}

impl Session for LocalSession {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize> {
        self.buffer.write_rsd(offset, dest, len)
    }

    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize> {
        self.buffer.write_wsd(offset, src, len)
    }
}
