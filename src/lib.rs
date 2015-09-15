
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
extern crate kernel32;
extern crate user32;
extern crate winapi;

mod ipc;
mod raw;

#[cfg(all(windows, target_pointer_width = "32"))]
pub mod local;

#[cfg(windows)]
pub mod user;

use std::io;
use std::mem::size_of;

/// A handle to FSUIPC
/// This type represents a handle to FSUIPC. It cannot be used directly to read of write from or
/// to FSUIPC offsets. A `Session` object is created from the handle instead.
pub trait Handle {
    /// The type of the session objects created by this handle.
    type Sess: Session;

    /// Create a new session from this handle
    fn session(&self) -> Self::Sess;

    /// Disconnect the handle
    fn disconnect(self);
}

/// A session of read & write operations from/to FSUIPC
/// Objects of this trait represents a session comprised of a sequence of read and write
/// operations. The operations are requested by using `read()` and `write()` methods.
/// They are not executed immediately but after calling `process()` method, which consumes
/// the session.
pub trait Session {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize>;
    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize>;
    fn process(self) -> io::Result<usize>;

    fn read<'a, T>(&'a mut self, offset: u16, result: &'a mut T) -> io::Result<usize> {
        self.read_bytes(offset, result as *mut T as *mut u8, size_of::<T>())
    }

    fn write<T>(&mut self, offset: u16, value: &T) -> io::Result<usize> {
        self.write_bytes(offset, value as *const T as *const u8, size_of::<T>())
    }
}
