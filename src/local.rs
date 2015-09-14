
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use super::{Handler, Session};

pub struct LocalHandler;

impl Handler for LocalHandler {
    type Sess = LocalSession;
    fn connect() -> io::Result<Self> { unimplemented!() }
    fn session(&self) -> LocalSession { unimplemented!() }
    fn disconnect(self) { unimplemented!() }
}

pub struct LocalSession;

impl Session for LocalSession {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize> {
        unimplemented!()
    }

    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize> {
        unimplemented!()
    }
}
