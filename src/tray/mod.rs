//! System tray module
//!
//! Manages the system tray icon, message loop, and user interactions.

mod icon;
mod menu;

use crate::config::{CaffeineMode, DisplayMode, MonitorMode, Settings, is_auto_start_enabled, enable_auto_start, disable_auto_start};
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

/// Custom message ID for tray icon events
const WM_TRAYICON: u32 = WM_USER + 1;
/// Timer ID for animation updates
const TIMER_ID: usize = 1;

/// Global running state
static RUNNING: AtomicBool = AtomicBool::new(false);
/// Global settings storage (protected by Mutex)
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

/// Main tray application
pub struct TrayApp;

impl TrayApp {
    /// Run the tray application
    /// 
    /// Creates a hidden window for receiving tray events and starts the message loop.
    pub fn run(settings: Settings) -> Result<()> {
        *SETTINGS.lock().unwrap() = Some(settings.clone());

        let class_name = w!("DashCatWndClass");
        
        // SAFETY: GetModuleHandleW returns the handle to the current module,
        // which is always valid for the running process.
        let h_instance = unsafe { GetModuleHandleW(None)? };

        // Define window class for the hidden message window
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance.into(),
            hIcon: HICON::default(),
            // SAFETY: LoadCursorW with IDC_ARROW is a standard system cursor
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
        };

        // SAFETY: RegisterClassW registers a window class. Safe with valid WNDCLASSW.
        unsafe { let _ = RegisterClassW(&wnd_class); }

        // SAFETY: CreateWindowExW creates a hidden window for message handling.
        // The window is never shown, so default parameters are appropriate.
        let _ = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("DashCat"),
                WINDOW_STYLE::default(),
                CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
                HWND::default(), HMENU::default(), h_instance, None,
            )
        };

        RUNNING.store(true, Ordering::SeqCst);

        // Main message loop
        let mut msg = MSG::default();
        while RUNNING.load(Ordering::SeqCst) {
            // SAFETY: Standard Windows message loop. GetMessageW, TranslateMessage,
            // and DispatchMessageW are safe with valid MSG structure.
            unsafe {
                let ret = GetMessageW(&mut msg, HWND::default(), 0, 0);
                if ret == BOOL(0) { break; }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    /// Window procedure for handling messages
    /// 
    /// # Safety
    /// This is an extern "system" function called by Windows. Static mutable
    /// variables are used for state that persists across message calls.
    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // Static state for monitors, caffeine, and icon
        // These are initialized in WM_CREATE and persist for the window lifetime
        static mut CPU_MONITOR: Option<CpuMonitor> = None;
        static mut MEMORY_MONITOR: Option<MemoryMonitor> = None;
        static mut CAFFEINE: Option<Caffeine> = None;
        static mut ICON: Option<TrayIcon> = None;

        match msg {
            WM_CREATE => {
                // Initialize monitors and icon
                CPU_MONITOR = Some(CpuMonitor::new());
                MEMORY_MONITOR = Some(MemoryMonitor::new());
                CAFFEINE = Some(Caffeine::new());
                ICON = Some(TrayIcon::new());

                // Create tray icon
                if let Some(ref mut icon) = ICON { let _ = icon.create(hwnd, 1); }
                
                // Apply saved caffeine mode
                if let Some(settings) = SETTINGS.lock().unwrap().as_ref() {
                    if let Some(ref mut caffeine) = CAFFEINE {
                        caffeine.set_mode(settings.caffeine_mode);
                    }
                }
                
                // Start animation timer (200ms interval)
                SetTimer(hwnd, TIMER_ID, 200, None);
                LRESULT::default()
            }
            
            WM_DESTROY => {
                // Cleanup on exit
                RUNNING.store(false, Ordering::SeqCst);
                if let Some(ref mut icon) = ICON { let _ = icon.remove(hwnd, 1); }
                PostQuitMessage(0);
                LRESULT::default()
            }
            
            WM_TRAYICON => {
                // Handle tray icon mouse events
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Show context menu on right-click
                        let settings = SETTINGS.lock().unwrap();
                        let (mm, dm, cm) = settings.as_ref()
                            .map(|s| (s.monitor_mode, s.display_mode, s.caffeine_mode))
                            .unwrap_or((MonitorMode::Combined, DisplayMode::Both, CaffeineMode::Off));
                        drop(settings);
                        
                        let mut menu = TrayMenu::new();
                        menu.show(hwnd, mm, dm, cm);
                    }
                    WM_LBUTTONUP => {}
                    _ => {}
                }
                LRESULT::default()
            }
            
            WM_TIMER => {
                // Update monitoring values and icon tooltip
                if let Some(ref mut cpu) = CPU_MONITOR {
                    if let Some(ref mem) = MEMORY_MONITOR {
                        let (c, m) = (cpu.usage(), mem.usage());
                        if let Some(ref mut icon) = ICON { icon.update(hwnd, 1, c, m); }
                    }
                }
                LRESULT::default()
            }
            
            WM_COMMAND => {
                // Handle menu commands
                Self::handle_command(wparam.0, &raw mut CAFFEINE);
                LRESULT::default()
            }
            
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// Handle menu command
    fn handle_command(cmd: usize, caffeine: *mut Option<Caffeine>) {
        match cmd {
            100..=102 => {
                // Monitor mode selection
                let mode = match cmd { 
                    100 => MonitorMode::Combined, 
                    101 => MonitorMode::Cpu, 
                    102 => MonitorMode::Memory, 
                    _ => MonitorMode::Combined 
                };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { 
                    s.monitor_mode = mode; 
                    let _ = s.save(); 
                }
            }
            200..=203 => {
                // Display mode selection
                let mode = match cmd { 
                    200 => DisplayMode::Both, 
                    201 => DisplayMode::AnimOnly, 
                    202 => DisplayMode::PctOnly, 
                    203 => DisplayMode::DualValues, 
                    _ => DisplayMode::Both 
                };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { 
                    s.display_mode = mode; 
                    let _ = s.save(); 
                }
            }
            300..=302 => {
                // Caffeine mode selection
                let mode = match cmd { 
                    300 => CaffeineMode::Off, 
                    301 => CaffeineMode::NoSleep, 
                    302 => CaffeineMode::NoDisplaySleep, 
                    _ => CaffeineMode::Off 
                };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { 
                    s.caffeine_mode = mode; 
                    let _ = s.save(); 
                }
                // SAFETY: caffeine pointer was obtained from valid static mut reference
                unsafe {
                    if let Some(ref mut c) = *caffeine { c.set_mode(mode); }
                }
            }
            400 => {
                // Toggle auto-start
                if is_auto_start_enabled() {
                    disable_auto_start();
                } else {
                    enable_auto_start();
                }
            }
            999 => { 
                // Quit
                RUNNING.store(false, Ordering::SeqCst); 
                // SAFETY: PostQuitMessage is safe to call at any time
                unsafe { PostQuitMessage(0); } 
            }
            _ => {}
        }
    }
}