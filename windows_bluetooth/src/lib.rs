mod discover;
mod bluetooth_device;
mod connect_os;
mod remove;
pub(crate) mod with_err;

use windows::core::HRESULT;
use windows::Win32::Foundation::WIN32_ERROR;
pub use discover::*;
pub use bluetooth_device::*;
pub use connect_os::*;
pub use remove::*;

#[inline]
fn err_eq(h_res: HRESULT, win32_err: WIN32_ERROR) -> bool {
    h_res == HRESULT::from_win32(win32_err.0)
}