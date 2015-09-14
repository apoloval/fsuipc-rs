# FSUIPC Rust library

This library provides the code needed to implement a [FSUIPC][1] client in
[Rust language][2].

## Usage

FSUIPC library is based on the following two traits:

* `fsuipc::Handle`, which represents a handle to FSUIPC. It cannot be used to
read of write FSUIPC offsets but to instantiate `fsuipc::Session` objects.
* `fsuipc::Session`, which represents a session comprised of a sequence of
read and write requests that are executed when `process()` method is invoked.

Let's see some examples:

```Rust
// First we create both handle & session using a local IPC.
let fsuipc = try!(fsuipc::local::LocalHandle::new());
let mut session = fsuipc.session();

// This variable will be use to store the result of reading the altitude
let mut altitude: u32 = 0;

// This is the value of QNH that we are gonna write to FSUIPC
let qnh: u16 = 1020 * 16;

// We request to read from offset 0x3324 and save the result in `altitude`.
// The length of the offset to read is inferred from the variable type
// (`altitude` is u32, so 4 bytes are requested).
session.read(0x3324, &mut altitude);

// We request to write to offset 0x0330 from the contents of `qnh`.
// The length of the offset to write is inferred from the variable type
// (`qnh` is u16, so 2 bytes are requested).
session.write(0x0330, &qnh);

// Now all the requests are processed. After that, `altitude` will receive
// the value read from FSUIPC, and offset 0x0330 will be updated with the
// value of `qnh`. This call consumes the `session` object. For further
// reads and writes, another session must be created.
try!(session.process());
```

## Current limitations

Only local IPC mode is supported. Thus the code using this library must be
run in the same process as FSUIPC (FSX/P3D module or gauge). 

## License

This code is published under [Mozilla Public License v2] terms.

[1]: http://www.schiratti.com/dowson.html
[2]: http://rust-lang.org/
[3]: https://www.mozilla.org/en-US/MPL/2.0/
