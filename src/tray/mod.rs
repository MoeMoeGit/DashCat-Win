//! System tray module

mod icon;
mod menu;

use crate::config::{CaffeineMode, DisplayMode, MonitorMode, Settings};
use crate::monitor::{CpuMonitor, MemoryMonitor};
use crate::power::Caffeine;
use icon::TrayIcon;
use menu::TrayMenu;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

// Message constants
const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1;

/// Global application state
static RUNNING: AtomicBool = AtomicBool::new(false);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
static STATS: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

/// Main tray application
pub struct TrayApp;

impl TrayApp {
    /// Run the application
    pub fn run(settings: Settings) -> Result<()> {
        // Store settings globally
        *SETTINGS.lock().unwrap() = Some(settings.clone());

        // Register window class
        let class_name = w!("DashCatWndClass");
        let h_instance = unsafe { GetModuleHandleW(None)? };

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance.into(),
            hIcon: HICON::default(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
        };

        unsafe {
            if RegisterClassW(&wnd_class) == 0 {
                // Class might already be registered, continue anyway
            }
        }

        // Create hidden window
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("DashCat"),
                WINDOW_STYLE::default(),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND::default(),
                HMENU::default(),
                h_instance,
                None,
            )
        };

        // Initialize caffeine with saved mode
        let mut caffeine = Caffeine::new();
        caffeine.set_mode(settings.caffeine_mode);

        // Start animation timer (200ms interval)
        unsafe {
            SetTimer(hwnd, TIMER_ID, 200, None);
        }

        // Set running flag
        RUNNING.store(true, Ordering::SeqCst);

        // Message loop
        let mut msg = MSG::default();
        while RUNNING.load(Ordering::SeqCst) {
            unsafe {
                let ret = GetMessageW(&mut msg, HWND::default(), 0, 0);
                if ret == BOOL(0) {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    /// Window procedure
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // Static state (persist across calls)
        static mut CPU_MONITOR: Option<CpuMonitor> = None;
        static mut MEMORY_MONITOR: Option<MemoryMonitor> = None;
        static mut CAFFEINE: Option<Caffeine> = None;
        static mut ICON: Option<TrayIcon> = None;

        match msg {
            WM_CREATE => {
                // Initialize components
                CPU_MONITOR = Some(CpuMonitor::new());
                MEMORY_MONITOR = Some(MemoryMonitor::new());
                CAFFEINE = Some(Caffeine::new());
                ICON = Some(TrayIcon::new());

                // Create tray icon
                if let Some(ref mut icon) = ICON {
                    let _ = icon.create(hwnd, 1);
                }

                // Apply saved caffeine mode
                if let Some(settings) = SETTINGS.lock().unwrap().as_ref() {
                    if let Some(ref mut caffeine) = CAFFEINE {
                        caffeine.set_mode(settings.caffeine_mode);
                    }
                }

                LRESULT::default()
            }
            WM_DESTROY => {
                RUNNING.store(false, Ordering::SeqCst);
                // Remove tray icon
                if let Some(ref mut icon) = ICON {
                    let _ = icon.remove(hwnd, 1);
                }
                PostQuitMessage(0);
                LRESULT::default()
            }
            WM_TRAYICON => {
                let mouse_msg = lparam.0 as u32;
                match mouse_msg {
                    WM_RBUTTONUP => {
                        // Show context menu
                        let mut menu = TrayMenu::new();
                        menu.show(hwnd);
                    }
                    WM_LBUTTONUP => {
                        // Left click - show clipboard panel (TODO)
                    }
                    _ => {}
                }
                LRESULT::default()
            }
            WM_TIMER => {
                // Update system stats and animation
                if let Some(ref mut cpu) = CPU_MONITOR {
                    if let Some(ref mem) = MEMORY_MONITOR {
                        let cpu_usage = cpu.usage();
                        let mem_usage = mem.usage();

                        // Store stats
                        if let Ok(mut stats) = STATS.lock() {
                            *stats = (cpu_usage, mem_usage);
                        }

                        // Update icon animation
                        if let Some(ref mut icon) = ICON {
                            icon.update(hwnd, 1, cpu_usage, mem_usage);
                        }
                    }
                }
                LRESULT::default()
            }
            WM_COMMAND => {
                // Handle menu commands
                let cmd = wparam.0 as usize;
                Self::handle_command(cmd, &mut CAFFEINE);
                LRESULT::default()
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// Handle menu command
    fn handle_command(cmd: usize, caffeine: &mut Option<Caffeine>) {
        match cmd {
            100..=102 => {
                // Monitor mode
                let mode = match cmd {
                    100 => MonitorMode::Combined,
                    101 => MonitorMode::Cpu,
                    102 => MonitorMode::Memory,
                    _ => MonitorMode::Combined,
                };
                if let Some(settings) = SETTINGS.lock().unwrap().as_mut() {
                    settings.monitor_mode = mode;
                    let _ = settings.save();
                }
            }
            200..=203 => {
                // Display mode
                let mode = match cmd {
                    200 => DisplayMode::Both,
                    201 => DisplayMode::AnimOnly,
                    202 => DisplayMode::PctOnly,
                    203 => DisplayMode::DualValues,
                    _ => DisplayMode::Both,
                };
                if let Some(settings) = SETTINGS.lock().unwrap().as_mut() {
                    settings.display_mode = mode;
                    let _ = settings.save();
                }
            }
            300..=302 => {
                // Caffeine mode
                let mode = match cmd {
                    300 => CaffeineMode::Off,
                    301 => CaffeineMode::NoSleep,
                    302 => CaffeineMode::NoDisplaySleep,
                    _ => CaffeineMode::Off,
                };
                if let Some(settings) = SETTINGS.lock().unwrap().as_mut() {
                    settings.caffeine_mode = mode;
                    let _ = settings.save();
                }
                // Apply caffeine mode
                if let Some(caffeine) = caffeine {
                    caffeine.set_mode(mode);
                }
            }
            999 => {
                // Quit
                RUNNING.store(false, Ordering::SeqCst);
                unsafe {
                    PostQuitMessage(0);
                }
            }
            _ => {}
        }
    }
}