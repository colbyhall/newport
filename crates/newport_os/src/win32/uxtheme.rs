use super::*;

#[link(name = "uxtheme")]
extern "stdcall" {
	pub fn SetWindowTheme(hwnd: HWND, pszSubAppName: LPCWSTR, pszSubIdList: LPCWSTR) -> HRESULT;
}
