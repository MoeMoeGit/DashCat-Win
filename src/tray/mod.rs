//! System tray module

mod icon;
mod menu;

use crate::config::Settings;
use icon::TrayIcon;
use menu::TrayMenu;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

/// Main tray application
pub struct TrayApp {
    settings: Settings,
    icon: TrayIcon,
    menu: TrayMenu,
    running: AtomicBool,
}

impl TrayApp {
    /// Create a new tray application
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            icon: TrayIcon::new(),
            menu: TrayMenu::new(),
            running: AtomicBool::new(false),
        }
    }

    /// Run the application main loop
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the window class and create hidden window
        self.init_window()?;

        // Create system tray icon
        self.icon.create()?;

        // Build the context menu
        self.menu.build(&self.settings);

        // Set running flag
        self.running.store(true, Ordering::SeqCst);

        // Main message loop
        let mut msg = MSG::default();
        while self.running.load(Ordering::SeqCst) {
            // Process messages with a timeout to allow for animation updates
            if unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0) } > 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }

        Ok(())
    }

    /// Initialize the hidden window for receiving messages
    fn init_window(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let h_instance = GetModuleHandleW(None)?;
            
            let class_name = windows::core::w!("DashCatWndClass");
            
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: HICON::default(),
                hCursor: HCURSOR::default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: windows::core::PCWSTR::null(),
                lpszClassName: class_name,
            };

            RegisterClassW(&wnd_class);

            // Create a hidden window
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                windows::core::w!("DashCat"),
                WINDOW_STYLE::default(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND::default(),
                HMENU::default(),
                h_instance,
                None as *const _,
            );

            if hwnd.is_invalid() {
                return Err("Failed to create window".into());
            }

            // Store hwnd for later use (in a thread-local or similar)
        }

        Ok(())
    }

    /// Window procedure for handling messages
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT::default()
            }
            WM_USER + 1 => {
                // Tray icon callback
                let mouse_msg = lparam.0 as u32;
                match mouse_msg {
                    WM_LBUTTONUP => {
                        // Left click - show clipboard panel
                        // TODO: Show clipboard panel
                    }
                    WM_RBUTTONUP => {
                        // Right click - show context menu
                        // TODO: Show menu
                    }
                    _ => {}
                }
                LRESULT::default()
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// Stop the application
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
