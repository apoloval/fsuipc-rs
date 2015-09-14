
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::cmp::min;

pub struct RawBytes {
    data: *const u8,
    len: usize,
}

impl RawBytes {
    pub fn new(data: *const u8, len: usize) -> Self {
        RawBytes { data: data, len: len }
    }
}

impl io::Read for RawBytes {
    fn read(&mut self, buff: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let nbytes = min(self.len, buff.len());
            for i in 0..nbytes {
                buff[i] = *self.data;
                self.data = self.data.offset(1);
                self.len -= 1;
            }
            Ok(nbytes)
        }
    }
}

#[cfg(test)]
mod test {

    use std::io::Read;

    use super::*;

    #[test]
    fn should_read_from_rawbytes() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0, 0, 0, 0];
        let mut raw = RawBytes::new(&src as *const u8, 4);
        assert_eq!(raw.read(&mut dest).unwrap(), 4);
        assert_eq!(dest[0], 1);
        assert_eq!(dest[1], 2);
        assert_eq!(dest[2], 3);
        assert_eq!(dest[3], 4);
    }

    #[test]
    fn should_read_from_rawbytes_with_underflow() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0, 0];
        let mut raw = RawBytes::new(&src as *const u8, 4);
        assert_eq!(raw.read(&mut dest).unwrap(), 2);
        assert_eq!(dest[0], 1);
        assert_eq!(dest[1], 2);
    }

    #[test]
    fn should_read_from_rawbytes_with_overflow() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0, 0, 0, 0, 0, 0];
        let mut raw = RawBytes::new(&src as *const u8, 4);
        assert_eq!(raw.read(&mut dest).unwrap(), 4);
        assert_eq!(dest[0], 1);
        assert_eq!(dest[1], 2);
        assert_eq!(dest[2], 3);
        assert_eq!(dest[3], 4);
        assert_eq!(dest[4], 0);
        assert_eq!(dest[5], 0);
    }
}
