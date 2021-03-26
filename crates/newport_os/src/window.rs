#[cfg(target_os = "windows")]
use crate::win32::*;

#[cfg(target_os = "windows")]
use crate::{MAKEINTRESOURCEA, GET_WHEEL_DELTA_WPARAM, HIWORD};

use crate::input::*;

use std::collections::VecDeque;
use std::mem::size_of;
use std::ptr::{null_mut, null, NonNull};
use std::ffi::CString;
use std::num::Wrapping;

/// Builder used to create [`Window`]s with set parameters
pub struct WindowBuilder {
    size:  (u32, u32),
    title: String,
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

            let mut adjusted_rect = RECT {
                left:   0,
                top:    0,
                right:  self.size.0,
                bottom: self.size.1,
            };
            AdjustWindowRect(&mut adjusted_rect, WP_OVERLAPPEDWINDOW, 0);

            let width  = (Wrapping(adjusted_rect.right) - Wrapping(adjusted_rect.left)).0;
            let height = (Wrapping(adjusted_rect.bottom) - Wrapping(adjusted_rect.top)).0;

            let window_title = CString::new(self.title.as_bytes()).unwrap();
            let handle = CreateWindowExA(
                0, 
                class.lpszClassName, 
                window_title.as_ptr(), 
                WP_OVERLAPPEDWINDOW, 
                0, 0, 
                width, height, 
                null_mut(), 
                null_mut(),
                class.hInstance, 
                null_mut()
            );

            if handle == INVALID_HANDLE_VALUE {
                return Err(WindowSpawnError::WindowCreateFailed);
            }

            let mut window = Window{
                handle: handle,
                size:   (width, height),
                title:  self.title,
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
        unsafe { ShowWindow(self.handle, 3) };
        let mut viewport_size = RECT::default();
        unsafe { GetClientRect(self.handle, &mut viewport_size); }
        self.size.0 = viewport_size.right - viewport_size.left;
        self.size.1 = viewport_size.bottom - viewport_size.top;
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
                window: NonNull::new(self as *mut Window).unwrap(),
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
}

#[cfg(target_os = "windows")]
impl Drop for Window {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.handle) };
        self.handle = null_mut();
    }
}

/// Different type of events that can occur to [`Window`]s
pub enum WindowEvent {
    FocusGained,
    FocusLost,
    Closed,
    Key { key: Input, pressed: bool },
    Resized(u32, u32),
    Resizing(u32, u32),
    Char(char),
    MouseWheel(i16),
    MouseButton { mouse_button: Input, pressed: bool },
    MouseMove(u32, u32)
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
    let mut result : Option<WindowEvent> = None;

    let window : &mut Window;
    unsafe { window = iterator.window.as_mut(); }

    match uMsg {
        WM_CLOSE => {
            result = Some(WindowEvent::Closed);
        },
        WM_SETFOCUS => {
            result = Some(WindowEvent::FocusGained);
        },
        WM_KILLFOCUS => {
            result = Some(WindowEvent::FocusLost);
        },
        WM_SYSKEYDOWN | WM_KEYDOWN => {
            let key = Input::key_from_code(wParam as u8).unwrap();
            result = Some(WindowEvent::Key{ key: key, pressed: true });
        },
        WM_SYSKEYUP | WM_KEYUP => {
            let key = Input::key_from_code(wParam as u8).unwrap();
            result = Some(WindowEvent::Key{ key: key, pressed: false });
        },
        WM_SIZING | WM_SIZE => {
            let mut viewport_size = RECT::default();
            unsafe { GetClientRect(hWnd, &mut viewport_size); }
            let old_width = window.size.0;
            let old_height = window.size.1;
            window.size.0 = viewport_size.right - viewport_size.left;
            window.size.1 = viewport_size.bottom - viewport_size.top;

            if uMsg == WM_SIZING {
                result = Some(WindowEvent::Resizing(old_width, old_height));
            } else {
                result = Some(WindowEvent::Resized(old_width, old_height));
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

            result = Some(WindowEvent::Char(std::char::from_u32(c).unwrap()));
        },
        WM_MOUSEHWHEEL => {
            let delta = GET_WHEEL_DELTA_WPARAM!(wParam);
            result = Some(WindowEvent::MouseWheel(delta));
        },
        WM_LBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_LEFT, pressed: true });
        },
        WM_LBUTTONUP => {
            unsafe { ReleaseCapture(); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_LEFT, pressed: false });
        },
        WM_MBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_MIDDLE, pressed: true });
        },
        WM_MBUTTONUP => {
            unsafe { ReleaseCapture(); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_MIDDLE, pressed: false });
        },
        WM_RBUTTONDOWN => {
            unsafe { SetCapture(window.handle); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_RIGHT, pressed: true });
        },
        WM_RBUTTONUP => {
            unsafe { ReleaseCapture(); }
            result = Some(WindowEvent::MouseButton{ mouse_button: MOUSE_BUTTON_RIGHT, pressed: false });
        },
        WM_MOUSEMOVE => {
            let x = (lParam & 0xFFFF) as u32;
            let y = ((lParam >> 16) & 0xFFFF) as u32;
            result = Some(WindowEvent::MouseMove(x, y));
        },
        _ => { }
    }

    if result.is_some() {
        iterator.queue.push_back(result.unwrap());
    }

    unsafe { DefWindowProcA(hWnd, uMsg, wParam, lParam) }
}