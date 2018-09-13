use std::ptr;
use std::mem;
use std::sync::atomic::{Ordering, AtomicIsize};

use ::widestring::{U16String, U16CString};
use ::winapi::ctypes::c_void;
use ::winapi::shared::minwindef::{BOOL, FALSE, TRUE};
use ::winapi::um::errhandlingapi::GetLastError;
use ::winapi::um::winbase::{self, LocalFree, FormatMessageW};
use ::winapi::um::winnt::{self, LPWSTR};
use ::winapi::um::consoleapi::SetConsoleCtrlHandler;
use ::winapi::um::winuser::{self, MessageBoxW, RegisterHotKey, UnregisterHotKey, PostQuitMessage, PostMessageW, PostThreadMessageW, GetMessageW};
use ::winapi::um::processthreadsapi::GetCurrentThreadId;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Win32 {
            display("win32 error: {}", last_error())
        }
    }
}

pub fn last_error() -> String {
    unsafe {
        let code = GetLastError();
        trace!("last win32 error code: {:#x}", code);

        let mut msg_buf = ptr::null_mut();
        let ret = FormatMessageW(winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER | winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
                                ptr::null(),
                                code,
                                winnt::MAKELANGID(winnt::LANG_NEUTRAL, winnt::SUBLANG_DEFAULT) as u32,
                                (&mut msg_buf as *mut LPWSTR) as LPWSTR,
                                0,
                                ptr::null_mut());
        if ret == 0 {
            return format!("failed to get error message, error code: {:#x}", GetLastError());
        }

        let message = U16String::from_ptr(msg_buf, ret as usize).to_string_lossy();
        LocalFree(msg_buf as *mut c_void);

        message
    }
}

pub fn error_mbox(msg: &str) {
    unsafe {
        MessageBoxW(ptr::null_mut(), U16CString::from_str(msg).unwrap().as_ptr(), U16CString::from_str("Error").unwrap().as_ptr(), winuser::MB_OK);
    }
}

pub fn get_current_thread_id() -> u32 {
    unsafe {
        GetCurrentThreadId()
    }
}

pub type ConsoleCtrlHandler = fn (ctrl_type: u32) -> bool;
static mut CTRL_HANDLER: Option<ConsoleCtrlHandler> = None;

unsafe extern "system" fn _ctrl_handler(ctrl_type: u32) -> BOOL {
    if let None = CTRL_HANDLER {
        return FALSE;
    }

    match CTRL_HANDLER {
        Some(handler) => (handler)(ctrl_type) as BOOL,
        None => FALSE
    }
}
pub fn set_ctrl_handler(handler: ConsoleCtrlHandler) -> Result<(), Error> {
    unsafe {
        if let None = CTRL_HANDLER {
            if SetConsoleCtrlHandler(Some(_ctrl_handler), TRUE) == 0 {
                return Err(Error::Win32);
            }
        }

        CTRL_HANDLER = Some(handler);
    }

    Ok(())
}

pub fn post_quit(code: i32) {
    unsafe {
        PostQuitMessage(code);
    }
}
pub fn post_message(msg: u32, w_param: usize, l_param: isize) -> Result<(), Error> {
    unsafe {
        if PostMessageW(ptr::null_mut(), msg, w_param, l_param) == 0 {
            return Err(Error::Win32);
        }

        Ok(())
    }
}
pub fn post_thread_message(thread_id: u32, msg: u32, w_param: usize, l_param: isize) -> Result<(), Error> {
    unsafe {
        if PostThreadMessageW(thread_id, msg, w_param, l_param) == 0 {
            return Err(Error::Win32);
        }

        Ok(())
    }
}
pub fn get_message(filter_min: u32, filter_max: u32) -> Result<winuser::MSG, Error> {
    unsafe {
        let mut msg = mem::zeroed();
        if GetMessageW(&mut msg, ptr::null_mut(), filter_min, filter_max) == -1 {
            return Err(Error::Win32);
        }

        Ok(msg)
    }
}

static HOTKEY_ID: AtomicIsize = AtomicIsize::new(0);
pub struct Hotkey(i32);
impl Hotkey {
    pub fn new(modifiers: isize, key: i32) -> Result<Hotkey, Error> {
        let id = HOTKEY_ID.fetch_add(1, Ordering::SeqCst) as i32;
        unsafe {
            if RegisterHotKey(ptr::null_mut(), id, modifiers as u32, key as u32) == 0 {
                return Err(Error::Win32);
            }

            trace!("registered hotkey {}", id);
        }

        Ok(Hotkey(id))
    }
    pub fn matches(&self, msg: &winuser::MSG) -> bool {
        if msg.message == winuser::WM_HOTKEY && msg.wParam as i32 == self.0 {
            true
        } else {
            false
        }
    }
}
impl Drop for Hotkey {
    fn drop(&mut self) {
        unsafe {
            if UnregisterHotKey(ptr::null_mut(), self.0) == 0 {
                error!("failed to unregister hotkey {}: {}", self.0, last_error());
            }

            trace!("unregistered hotkey {}", self.0);
        }
    }
}
