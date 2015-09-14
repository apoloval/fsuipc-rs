
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
extern crate user32;
extern crate winapi;

mod ipc;
mod local;
mod raw;

use std::io;
use std::mem::size_of;

pub trait Handle {
    type Sess: Session;
    fn connect() -> io::Result<Self>;
    fn session(&self) -> Self::Sess;
    fn disconnect(self);
}

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
