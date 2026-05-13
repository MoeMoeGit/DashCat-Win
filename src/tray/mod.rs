//! System tray module

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

const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1;

static RUNNING: AtomicBool = AtomicBool::new(false);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

pub struct TrayApp;

impl TrayApp {
    pub fn run(settings: Settings) -> Result<()> {
        *SETTINGS.lock().unwrap() = Some(settings.clone());

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

        unsafe { let _ = RegisterClassW(&wnd_class); }

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

        let mut msg = MSG::default();
        while RUNNING.load(Ordering::SeqCst) {
            unsafe {
                let ret = GetMessageW(&mut msg, HWND::default(), 0, 0);
                if ret == BOOL(0) { break; }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        static mut CPU_MONITOR: Option<CpuMonitor> = None;
        static mut MEMORY_MONITOR: Option<MemoryMonitor> = None;
        static mut CAFFEINE: Option<Caffeine> = None;
        static mut ICON: Option<TrayIcon> = None;

        match msg {
            WM_CREATE => {
                CPU_MONITOR = Some(CpuMonitor::new());
                MEMORY_MONITOR = Some(MemoryMonitor::new());
                CAFFEINE = Some(Caffeine::new());
                ICON = Some(TrayIcon::new());

                if let Some(ref mut icon) = ICON { let _ = icon.create(hwnd, 1); }
                if let Some(settings) = SETTINGS.lock().unwrap().as_ref() {
                    if let Some(ref mut caffeine) = CAFFEINE {
                        caffeine.set_mode(settings.caffeine_mode);
                    }
                }
                SetTimer(hwnd, TIMER_ID, 200, None);
                LRESULT::default()
            }
            WM_DESTROY => {
                RUNNING.store(false, Ordering::SeqCst);
                if let Some(ref mut icon) = ICON { let _ = icon.remove(hwnd, 1); }
                PostQuitMessage(0);
                LRESULT::default()
            }
            WM_TRAYICON => {
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
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
                if let Some(ref mut cpu) = CPU_MONITOR {
                    if let Some(ref mem) = MEMORY_MONITOR {
                        let (c, m) = (cpu.usage(), mem.usage());
                        if let Some(ref mut icon) = ICON { icon.update(hwnd, 1, c, m); }
                    }
                }
                LRESULT::default()
            }
            WM_COMMAND => {
                Self::handle_command(wparam.0 as usize, &raw mut CAFFEINE);
                LRESULT::default()
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn handle_command(cmd: usize, caffeine: *mut Option<Caffeine>) {
        match cmd {
            100..=102 => {
                let mode = match cmd { 100 => MonitorMode::Combined, 101 => MonitorMode::Cpu, 102 => MonitorMode::Memory, _ => MonitorMode::Combined };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { s.monitor_mode = mode; let _ = s.save(); }
            }
            200..=203 => {
                let mode = match cmd { 200 => DisplayMode::Both, 201 => DisplayMode::AnimOnly, 202 => DisplayMode::PctOnly, 203 => DisplayMode::DualValues, _ => DisplayMode::Both };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { s.display_mode = mode; let _ = s.save(); }
            }
            300..=302 => {
                let mode = match cmd { 300 => CaffeineMode::Off, 301 => CaffeineMode::NoSleep, 302 => CaffeineMode::NoDisplaySleep, _ => CaffeineMode::Off };
                if let Some(s) = SETTINGS.lock().unwrap().as_mut() { s.caffeine_mode = mode; let _ = s.save(); }
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
            999 => { RUNNING.store(false, Ordering::SeqCst); unsafe { PostQuitMessage(0); } }
            _ => {}
        }
    }
}