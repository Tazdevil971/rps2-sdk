use core::ffi::{c_char, c_void};

pub const OPEN_CALL: i32 = 1;
pub const CLOSE_CALL: i32 = 2;
pub const REQ_SEND_CALL: i32 = 3;
pub const POLL_CALL: i32 = 4;
pub const EX_RECV_CALL: i32 = -5;
pub const EX_SEND_CALL: i32 = -6;
pub const EX_REQ_SEND_CALL: i32 = -7;
pub const EX_LOCK_CALL: i32 = -8;
pub const EX_UNLOCK_CALL: i32 = -9;
pub const KPUTS_CALL: i32 = 0x10;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Read = 1,
    ReadDone = 2,
    Write = 3,
    WriteDone = 4,
    ChStatus = 5,
    Error = 6,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// Invalid argument.
    Invalid = -1,
    /// Invalid socket descriptor.
    InvalidSock = -2,
    /// Protocol number already used.
    AlreadyUse = -3,
    /// Too many open protocols.
    MFile = -4,
    /// Invalid address for buffer.
    InvalidAddr = -5,
    /// Buffer is too small.
    PktSize = -6,
    /// Blocks in spite asynchronous.
    WouldBlock = -7,
    /// Already locked.
    AlreadyLock = -8,
    /// Not locked.
    NotLocked = -9,
    /// No route to host.
    NoRoute = -10,
    /// No room left on manager.
    NoSpace = -11,
    /// Invalid deci2 header.
    InvalHead = -12,
}

type Handler = fn(event: i32, param: i32, opt: *mut c_void);

macro_rules! deci2_invoke {
    ($id:expr, $($args:expr),*) => {{
        let mut args = [$($args as u32),*];
        let res = crate::os::deci2_call($id, args.as_mut_ptr());
        core::mem::transmute::<_, Error>(res)
    }};
}

pub unsafe fn open(proto: u16, opt: *mut c_void, handler: Handler) -> Error {
    deci2_invoke!(OPEN_CALL, proto, opt, handler, 0x20325fb0)
}

pub unsafe fn close(sock: i32) -> Error {
    deci2_invoke!(CLOSE_CALL, sock)
}

pub unsafe fn req_send(sock: i32, dest: i8) -> Error {
    deci2_invoke!(REQ_SEND_CALL, sock, dest)
}

pub unsafe fn poll(sock: i32) {
    deci2_invoke!(POLL_CALL, sock);
}

pub unsafe fn ex_recv(sock: i32, buf: *mut c_void, len: u16) -> Error {
    deci2_invoke!(EX_RECV_CALL, sock, buf, len)
}

pub unsafe fn ex_send(sock: i32, buf: *mut c_void, len: u16) -> Error {
    deci2_invoke!(EX_SEND_CALL, sock, buf, len)
}

pub unsafe fn ex_req_send(sock: i32, dest: i8) -> Error {
    deci2_invoke!(EX_REQ_SEND_CALL, sock, dest)
}

pub unsafe fn ex_lock(sock: i32) -> Error {
    deci2_invoke!(EX_LOCK_CALL, sock)
}

pub unsafe fn ex_unlock(sock: i32) -> Error {
    deci2_invoke!(EX_UNLOCK_CALL, sock)
}

pub unsafe fn kputs(ptr: *const c_char) {
    deci2_invoke!(KPUTS_CALL, ptr);
}
