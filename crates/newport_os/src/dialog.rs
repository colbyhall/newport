use crate::win32::*;
use crate::window::Window;

use std::{
    ptr::{ null, null_mut },
    mem::size_of,
};

pub struct DialogBuilder<'a> {
    window: &'a Window,

    _title: Option<String>,

    extensions:        Vec<(String, String)>,
    default_extension: usize,
}

impl<'a> DialogBuilder<'a> {
    pub fn new(window: &'a Window) -> Self {
        let result = Self{
            window,

            _title: None,
            extensions: Vec::new(),
            default_extension: 0,
        };

        result.extension("All", "*", false)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self._title = Some(title.into());
        self
    }

    pub fn extension(mut self, name: impl Into<String>, extension: impl Into<String>, as_default: bool) -> Self {
        self.extensions.push((name.into(), extension.into()));
        if as_default {
            self.default_extension = self.extensions.len() - 1;
        }
        self
    }
}

pub struct DialogResult {
    _entry: String,
}

impl<'a> DialogBuilder<'a> {
    pub fn show(self) -> Option<DialogResult> {
        let mut filter = String::new();
        for (name, ext) in self.extensions {
            filter.push_str(&name);
            filter.push(0 as char);
            filter.push_str(&format!("*.{}", &ext));
            filter.push(0 as char);
        }
        filter.push(0 as char);

        let mut file: [i8; 260] = [0; 260];

        let mut ofa = OPENFILENAMEA{
            lStructSize: size_of::<OPENFILENAMEA>() as DWORD,
            hwndOwner: self.window.handle(),
            hInstance: unsafe{ GetModuleHandleA(null()) },
            lpstrFilter: filter.as_ptr() as *const i8,
            lpstrCustomFilter: null_mut(),
            nMaxCustFilter: 0,
            nFilterIndex: self.default_extension as DWORD,
            lpstrFile: file.as_mut_ptr(),
            nMaxFile: 260,
            lpstrFileTitle: null_mut(),
            nMaxFileTitle: 0,
            lpstrInitialDir: null_mut(),
            lpstrTitle: null(),
            Flags: OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST,
            nFileOffset: 0,
            nFileExtension: 0,
            lpstrDefExt: null_mut(),
            lCustData: 0,
            lpfnHook: null_mut(),
            lpTemplateName: null_mut(),
            lpEditInfo: null_mut(),
            lpstrPrompt: null_mut(),
            pvReserved: null_mut(),
            dwReserved: 0,
            FlagsEx: 0,
        };

        unsafe{ GetOpenFileNameA(&mut ofa) };

        let result = unsafe{ GetLastError() };
        println!("{}", result);

        // NOT FINISHED
        todo!();
    }
}