#![allow(dead_code)]
#![allow(arithmetic_overflow)]

#[cfg(target_os = "windows")]
use crate::win32::*;

#[cfg(target_os = "windows")]
use crate::{ MAKEINTRESOURCEA, GET_WHEEL_DELTA_WPARAM, HIWORD, proc_address };

use crate::library::Library;
use crate::input::*;

use newport_math::Rect;

use lazy_static::lazy_static;

use std::collections::VecDeque;
use std::mem::size_of;
use std::ptr::{null_mut, null, NonNull};
use std::ffi::CString;
use std::num::Wrapping;

#[repr(C)]
#[allow(non_camel_case_types)]
enum PROCESS_DPI_AWARENESS {
    PROCESS_DPI_UNAWARE,
    PROCESS_SYSTEM_DPI_AWARE,
    PROCESS_PER_MONITOR_DPI_AWARE
}

type SetProcessDPIAwareness = extern fn(PROCESS_DPI_AWARENESS) -> HRESULT;

#[repr(C)]
#[allow(non_camel_case_types)]
enum MONITOR_DPI_TYPE {
    MDT_EFFECTIVE_DPI,
    MDT_ANGULAR_DPI,
    MDT_RAW_DPI,
    MDT_DEFAULT,
}

type GetDpiForMonitor = extern fn(HMONITOR, MONITOR_DPI_TYPE, *mut UINT, *mut UINT) -> HRESULT;

#[cfg(target_os = "windows")]
lazy_static! {
    static ref SHCORE: Option<Library> = {
        let library = Library::new("shcore.dll").ok()?;

        let func : Option<SetProcessDPIAwareness> = proc_address!(library, "SetProcessDpiAwareness");
        if func.is_some() {
            let func = func.unwrap();
            func(PROCESS_DPI_AWARENESS::PROCESS_SYSTEM_DPI_AWARE);
        }

        Some(library)
    };
}

#[derive(Copy, Clone)]
pub enum WindowStyle {
    Windowed,
    Borderless,
    Fullscreen,
    CustomTitleBar{
        border: f32,
        drag:   Rect,
    }
}

/// Builder used to create [`Window`]s with set parameters
pub struct WindowBuilder {
    size:  (u32, u32),
    title: String,
    style: WindowStyle,
}

impl WindowBuilder {
    /// Returns a [`WindowBuilder`] to start building off of
    /// 
    /// # Examples
    /// ```
    /// use newport_os::window::WindowBuilder;
    /// let builder = WindowBuilder::new();
    /// ```
    pub const fn new() -> Self {
        Self{
            size: (1280, 720),
            title: String::new(),
            style: WindowStyle::Windowed,
        }
    }

    /// Sets title in [`WindowBuilder`]. Consumes and returns a [`WindowBuilder`] to build off of. 
    /// 
    /// # Arguments
    /// 
    /// * `title` - A string that will be used as the title in the spawned window
    /// 
    /// # Examples
    /// ```
    /// use newport_os::window::WindowBuilder;
    /// let builder = WindowBuilder::new()
    ///     .title("Hello, world!".to_string());
    /// ```
    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    /// Sets size in [`WindowBuilder`]. Consumes and returns a [`WindowBuilder`] to build off of.
    /// 
    /// # Arguments
    /// 
    /// * `size` - A tuple of `(u32, u32)` that is the width and height of the viewport in the spawned window
    /// 
    /// # Examples
    /// ```
    /// use newport_os::window::WindowBuilder;
    /// 
    /// let builder = WindowBuilder::new()
    ///     .size((1920, 1080));
    /// ```
    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.size = size;
        self
    }

    pub fn style(mut self, style: WindowStyle) -> Self {
        self.style = style;
        self
    }
}

/// Error reported in [`WindowBuilder::spawn()`]
#[derive(Debug)]
pub enum WindowSpawnError {
    ClassRegisterFailed,
    WindowCreateFailed,
}

#[cfg(target_os = "windows")]
impl WindowBuilder {
    /// Consumes a [`WindowBuilder`] and tries to create a [`Window`]. Returns a 
    /// [`Window`] on success and a [`WindowSpawnError`] on fail.
    /// 
    /// # Examples
    /// ```
    /// use newport_os::window::WindowBuilder;
    /// let window = WindowBuilder::new()
    ///     .title("Hello, world!".to_string())
    ///     .size((1920, 1080))
    ///     .spawn()
    ///     .unwrap();
    /// ```
    pub fn spawn(self) -> Result<Window, WindowSpawnError> {
        #[allow(unused_unsafe)]
        unsafe {
            let class = WNDCLASSEXA{
                cbSize:         size_of::<WNDCLASSEXA>() as UINT, 
                style:          0, 
                lpfnWndProc:    Some(window_callback), 
                cbClsExtra:     0, 
                cbWndExtra:     0, 
                hInstance:      GetModuleHandleA(null()), 
                hIcon:          null_mut(), 
                hCursor:        LoadCursorA(null_mut(), MAKEINTRESOURCEA!(32512)),
                hbrBackground:  5 as HBRUSH, 
                lpszMenuName:   null_mut(),
                lpszClassName:  self.title.as_ptr() as LPCSTR,
                hIconSm:        null_mut(),
            };

            if RegisterClassExA(&class) == 0 {
                return Err(WindowSpawnError::ClassRegisterFailed);
            }

            // Apparently on Windows 10 "A" suffix functions can take utf 8
            assert!(self.title.is_ascii(), "We're only using ASCII windows functions. This should eventually change");

            let style = WS_OVERLAPPEDWINDOW;

            let mut adjusted_rect = RECT {
                left:   0,
                top:    0,
                right:  self.size.0,
                bottom: self.size.1,
            };
            AdjustWindowRect(&mut adjusted_rect, style, 0);

            let width  = (Wrapping(adjusted_rect.right) - Wrapping(adjusted_rect.left)).0;
            let height = (Wrapping(adjusted_rect.bottom) - Wrapping(adjusted_rect.top)).0;

            let window_title = CString::new(self.title.as_bytes()).unwrap();
            let handle = CreateWindowExA(
                0, 
                class.lpszClassName, 
                window_title.as_ptr(), 
                style, 
                0, 0, 
                width, height, 
                null_mut(), 
                null_mut(),
                class.hInstance, 
                null_mut()
            );

            let shcore = SHCORE.as_ref();
            let dpi = if shcore.is_none() {
                1.0
            } else {
                let shcore = shcore.unwrap();

                let mut result = 1.0;

                let func : Option<GetDpiForMonitor> = proc_address!(shcore, "GetDpiForMonitor");
                if func.is_some() {
                    let monitor = MonitorFromWindow(handle, MONITOR_DEFAULTTONEAREST);

                    let mut dpix : UINT = 0;
                    let mut dpiy : UINT = 0;
                    let func = func.unwrap();
                    func(monitor, MONITOR_DPI_TYPE::MDT_EFFECTIVE_DPI, &mut dpix, &mut dpiy);
                    result = dpix as f32 / 96.0;
                }
                result
            };

            if handle == INVALID_HANDLE_VALUE {
                return Err(WindowSpawnError::WindowCreateFailed);
            }

            // We have to do this to prevent weird bugs with top title bar showing
            match &self.style {
                WindowStyle::CustomTitleBar{ .. } => {
                    SetWindowTheme(handle, b"\0".as_ptr() as LPCWSTR, b"\0".as_ptr() as LPCWSTR);
                }
                _ => {} 
            }

            let mut track = TRACKMOUSEEVENT{
                cbSize:         size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags:        0x00000002, // Mouse Leave
                hwndTrack:      handle,
                dwHoverTime:    0,
            };
            TrackMouseEvent(&mut track);

            let mut window = Window{
                handle: handle,
                size:   (width, height),
                title:  self.title,
                dpi:    dpi,
                style:  self.style,
                mouse_left: false,
            };

            window.center_in_window(); // TODO: Have some position in the builder with a center function

            Ok(window)            
        }
    }
}

#[cfg(target_os = "windows")]
pub use crate::win32::HWND as WindowHandle;

/// An os's shell window that can be drawn into
/// 
/// This can be used by different libraries for drawing and input
pub struct Window {
    handle: WindowHandle,
    size: (u32, u32),
    title: String,
    dpi:   f32,
    style: WindowStyle,
    
    mouse_left: bool,
}

#[cfg(target_os = "windows")]
impl Window {
    /// Sets a window to be visible
    pub fn set_visible(&mut self, visible: bool) -> bool {
        let visibility = if visible { SW_SHOW } else { SW_HIDE };
        unsafe{ ShowWindow(self.handle, visibility) == 1 }
    }

    /// Centers a window in its current monitor
    pub fn center_in_window(&mut self) -> bool {
        unsafe {
            let monitor_width = GetSystemMetrics(SM_CXSCREEN);
            let monitor_height = GetSystemMetrics(SM_CYSCREEN);
    
            let x = monitor_width / 2 - (self.size.0 as i32) / 2;
            let y = monitor_height / 2 - (self.size.1 as i32) / 2;
    
            SetWindowPos(self.handle as HWND, null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER) == 1
        }
    }

    /// Maximizes a window and updates the size
    pub fn maximize(&mut self) {
        if self.is_maximized() {
            unsafe { ShowWindow(self.handle, 9) };
        } else {
            unsafe { ShowWindow(self.handle, 3) };
        }
        let mut viewport_size = RECT::default();
        unsafe { GetClientRect(self.handle, &mut viewport_size); }
        self.size.0 = viewport_size.right - viewport_size.left;
        self.size.1 = viewport_size.bottom - viewport_size.top;
    }

    pub fn minimize(&mut self) {
        unsafe { ShowWindow(self.handle, 6) };
    }

    /// Polls os shell for window events and returns a [`WindowEventIterator`]
    /// 
    /// # Examples
    /// 
    /// ```
    /// use newport_os::window::{ WindowEvent, WindowBuilder };
    /// 
    /// let window = WindowBuilder::new().spawn().unwrap();
    /// 
    /// `run: loop {
    ///     for event in window.poll_events() {
    ///         match event {
    ///             WindowEvent::Closed => break `run;
    ///             _ => { }
    ///         }
    ///     }
    /// } 
    /// ```
    pub fn poll_events(&mut self) -> WindowEventIterator {
        unsafe {
            let mut result = WindowEventIterator{
                queue:  VecDeque::new(),

                window:      NonNull::new(self as *mut Window).unwrap(),
            };

            // We upload the self ptr here because of rust move semantics.
            let result_ptr = &mut result as *mut WindowEventIterator;
            SetWindowLongPtrA(self.handle, GWLP_USERDATA, result_ptr as LONG_PTR);

            let mut msg = MSG {
                hWnd: null_mut(),
                message: 0,
                wParam: 0,
                lParam: 0,
                time: 0,
                pt: POINT {
                    x: 0,
                    y: 0
                },
            };

            // Poll window events and dispatch events through the window_callback
            while PeekMessageA(&mut msg, self.handle, 0, 0, PM_REMOVE) == 1 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            result
        }
    }

    /// Returns the title as a &str
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the os handle generally as a [`std::ffi::c_void`]
    pub fn handle(&self) -> WindowHandle {
        self.handle
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    pub fn is_maximized(&self) -> bool {
        unsafe{ IsZoomed(self.handle) != 0 }
    }

    pub fn is_minimized(&self) -> bool {
        unsafe{ IsIconic(self.handle) != 0 }
    }

    pub fn set_custom_drag(&mut self, new_drag: Rect) {
        match &mut self.style {
            WindowStyle::CustomTitleBar{ drag, .. } => {
                *drag = new_drag;
            },
            _ => { }, // Do nothing. This kind of sucks
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for Window {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.handle) };
        self.handle = null_mut();
    }
}

/// Different type of events that can occur to [`Window`]s
#[derive(Copy, Clone)]
pub enum WindowEvent {
    FocusGained,
    FocusLost,
    Closed,
    Key { key: Input, pressed: bool },
    Resized(u32, u32),
    Resizing(u32, u32),
    Char(char),
    MouseWheel(i16),
    MouseButton { mouse_button: Input, pressed: bool, position: (u32, u32) },
    MouseMove(u32, u32),
    MouseLeave
}

/// Iterator containing [`WindowEvent`]s after being polled
pub struct WindowEventIterator {
    queue:  VecDeque<WindowEvent>,
    
    window: NonNull<Window>,
}

impl Iterator for WindowEventIterator {
    type Item = WindowEvent;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
extern fn window_callback(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let iterator : &mut WindowEventIterator;
    unsafe {
        let iterator_ptr = GetWindowLongPtrA(hWnd, GWLP_USERDATA) as *mut WindowEventIterator;
        if iterator_ptr == null_mut() {
            return DefWindowProcA(hWnd, uMsg, wParam, lParam);
        }
        iterator = &mut *iterator_ptr;
    }
    let mut event : Option<WindowEvent> = None;

    let window = unsafe { iterator.window.as_mut() };

    let x = (lParam & 0xFFFF) as u32;
    let y = ((lParam >> 16) & 0xFFFF) as u32;

    let mut result: LRESULT = 0;
    match uMsg {
        WM_CLOSE => {
            event = Some(WindowEvent::Closed);
        },
        WM_SETFOCUS => {
            event = Some(WindowEvent::FocusGained);
        },
        WM_KILLFOCUS => {
            event = Some(WindowEvent::FocusLost);
        },
        WM_SYSKEYDOWN | WM_KEYDOWN => {
            let key = Input::key_from_code(wParam as u8);
            if key.is_some() {
                event = Some(WindowEvent::Key{ key: key.unwrap(), pressed: true });
            }
        },
        WM_SYSKEYUP | WM_KEYUP => {
            let key = Input::key_from_code(wParam as u8);
            if key.is_some() {
                event = Some(WindowEvent::Key{ key: key.unwrap(), pressed: false });
            }
        },
        WM_SIZING | WM_SIZE => {
            let mut viewport_size = RECT::default();
            unsafe { GetClientRect(hWnd, &mut viewport_size); }
            let old_width = window.size.0;
            let old_height = window.size.1;
            window.size.0 = viewport_size.right - viewport_size.left;
            window.size.1 = viewport_size.bottom - viewport_size.top;

            if uMsg == WM_SIZING {
                event = Some(WindowEvent::Resizing(old_width, old_height));
            } else {
                event = Some(WindowEvent::Resized(old_width, old_height));
            }
        },
        WM_CHAR => {
            static mut SURROGATE_PAIR_FIRST : u32 = 0;
            let mut c = wParam as u32;

            if c < 32 && c != '\t' as u32 { return 0; }
            if c == 127 { return 0; }

            if c >= 0xD800 && c <= 0xDBFF {
                unsafe { SURROGATE_PAIR_FIRST = c; }
                return 0;
            } else if c >= 0xDC00 && c <= 0xDFFF {
                let surrogate_pair_second = c;
                c = 0x10000;
                unsafe { c += (SURROGATE_PAIR_FIRST & 0x03FF) << 10; }
                c += surrogate_pair_second & 0x03FF;
            }

            event = Some(WindowEvent::Char(std::char::from_u32(c).unwrap()));
        },
        WM_MOUSEWHEEL => {
            let delta = GET_WHEEL_DELTA_WPARAM!(wParam) / 8;
            event = Some(WindowEvent::MouseWheel(delta));
        },
        WM_LBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_LEFT, pressed: true, position: (x, height - y) });
        },
        WM_LBUTTONUP => {
            unsafe { ReleaseCapture(); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_LEFT, pressed: false, position: (x, height - y) });
        },
        WM_MBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_MIDDLE, pressed: true, position: (x, height - y) });
        },
        WM_MBUTTONUP => {
            unsafe { ReleaseCapture(); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_MIDDLE, pressed: false, position: (x, height - y) });
        },
        WM_RBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_RIGHT, pressed: true, position: (x, height - y) });
        },
        WM_RBUTTONUP => {
            unsafe { ReleaseCapture(); }
            let (_, height) = window.size();
            event = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_RIGHT, pressed: false, position: (x, height - y) });
        },
        WM_MOUSEMOVE => {
            let (_, height) = window.size();

            event = Some(WindowEvent::MouseMove(x, height - y));

            if window.mouse_left {
                window.mouse_left = false;

                let mut track = TRACKMOUSEEVENT{
                    cbSize:         size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags:        0x00000002, // Mouse Leave
                    hwndTrack:      hWnd,
                    dwHoverTime:    0,
                };
                unsafe { TrackMouseEvent(&mut track) };
            }
        },
        WM_MOUSELEAVE => {
            event = Some(WindowEvent::MouseLeave);

            window.mouse_left = true;
        },

        WM_DWMCOMPOSITIONCHANGED => {
            match window.style {
                WindowStyle::CustomTitleBar{ .. } => {
                    // let mut margins = MARGINS::default();
        
                    // if window.is_maximized() {
                    //     let x_push = unsafe{ GetSystemMetrics(SM_CXFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) };
                    //     let y_push = unsafe{ GetSystemMetrics(SM_CYFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) };
        
                    //     margins.cxLeftWidth = x_push;
                    //     margins.cxRightWidth = x_push;
        
                    //     margins.cyTopHeight = y_push;
                    //     margins.cyBottomHeight = y_push;
                    // }
        
                    // unsafe{ DwmExtendFrameIntoClientArea(hWnd, &margins) };
                },
                _ => { }
            }
        },
        WM_NCACTIVATE => {
            match window.style {
                WindowStyle::CustomTitleBar{ .. } => {
                    result = 1;

                    if window.is_minimized() {
                        result = unsafe { DefWindowProcA(hWnd, uMsg, wParam, lParam) };
                    }
                },
                _ => { }
            }
        },
        WM_NCCALCSIZE => {
            match window.style {
                WindowStyle::CustomTitleBar{ .. } => {
                    let mut margins = MARGINS::default();
        
                    let rect = unsafe{ &mut *(lParam as *mut RECT) };
        
                    if window.is_maximized() {
                        let x_push = unsafe{ GetSystemMetrics(SM_CXFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) };
                        let y_push = unsafe{ GetSystemMetrics(SM_CYFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) };
        
                        rect.left += x_push as u32;
                        rect.top  += y_push as u32;
                        rect.bottom -= x_push as u32;
                        rect.right -= y_push as u32;
        
                        margins.cxLeftWidth = x_push;
                        margins.cxRightWidth = x_push;
        
                        margins.cyTopHeight = y_push;
                        margins.cyBottomHeight = y_push;
                    }
        
                    unsafe{ DwmExtendFrameIntoClientArea(hWnd, &margins) };
                },
                _ => { }
            }
        },
        WM_NCHITTEST => {
            match window.style {
                WindowStyle::CustomTitleBar{ border, drag } => {
                    let mut mouse = POINT{ x: x, y: y };

                    let mut frame = RECT::default();
                    unsafe{ GetWindowRect(hWnd, &mut frame) };
        
                    let mut client = RECT::default();
                    unsafe{ 
                        GetClientRect(hWnd, &mut client);
                        ScreenToClient(hWnd, &mut mouse);
                    }

                    if !client.point_in_rect(mouse) {
                        return HTNOWHERE;
                    }

                    // Convert newport rect into windows RECT
                    let height = client.bottom;
                    let drag = RECT {
                        top:    height - drag.max.y as u32,
                        bottom: height - drag.min.y as u32,

                        left:   drag.min.x as u32,
                        right:  drag.max.x as u32,
                    };
        
                    let mut left = false;
                    let mut right = false;
                    let mut bot = false;
                    let mut top = false;
                    if !window.is_minimized() {
                        left = client.left <= mouse.x && mouse.x < client.left + border as u32;
                        right = client.right - border as u32 <= mouse.x && mouse.x < client.right;
                        bot = client.bottom - border as u32 <= mouse.y && mouse.y < client.bottom;
                        top = client.top <= mouse.y && mouse.y < client.top + border as u32;
                    }
                    
                    if left {
                        if top {
                            result = HTTOPLEFT;
                        } else if bot {
                            result = HTBOTTOMLEFT;
                        } else {
                            result = HTLEFT;
                        }
                    } else if right {
                        if top {
                            result = HTTOPRIGHT;
                        } else if bot {
                            result = HTBOTTOMRIGHT;
                        } else {
                            result = HTRIGHT;
                        }
                    } else if top {
                        result = HTTOP;
                    } else if bot {
                        result = HTBOTTOM;
                    } else {
                        if drag.point_in_rect(mouse) {
                            result = HTCAPTION;
                        } else {
                            result = HTCLIENT;
                        }
                    }
                },
                _ => { }
            }

        },
        _ => result = unsafe { DefWindowProcA(hWnd, uMsg, wParam, lParam) }
    }

    if event.is_some() {
        iterator.queue.push_back(event.unwrap());
    }

    result
}