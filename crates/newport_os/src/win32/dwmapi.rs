use super::*;

#[repr(C)]
#[derive(Default)]
pub struct MARGINS {
	pub cxLeftWidth: i32,
	pub cxRightWidth: i32,
	pub cyTopHeight: i32,
	pub cyBottomHeight: i32,
}
pub type PMARGINS = *mut MARGINS;

pub const HTNOWHERE: i64 = 0;
pub const HTCLIENT: i64 = 1;
pub const HTCAPTION: i64 = 2;
pub const HTLEFT: i64 = 10;
pub const HTRIGHT: i64 = 11;
pub const HTTOP: i64 = 12;
pub const HTTOPLEFT: i64 = 13;
pub const HTTOPRIGHT: i64 = 14;
pub const HTBOTTOM: i64 = 15;
pub const HTBOTTOMLEFT: i64 = 16;
pub const HTBOTTOMRIGHT: i64 = 17;

#[link(name = "dwmapi")]
extern "stdcall" {
	pub fn DwmExtendFrameIntoClientArea(hwnd: HWND, pMarInset: *const MARGINS) -> HRESULT;
}
