//! Tray icon rendering and animation

use std::time::{Duration, Instant};

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;

/// Cat animation frames (embedded PNGs)
const CAT_FRAMES: [&[u8]; 5] = [
    include_bytes!("../assets/cat_0.png"),
    include_bytes!("../assets/cat_1.png"),
    include_bytes!("../assets/cat_2.png"),
    include_bytes!("../assets/cat_3.png"),
    include_bytes!("../assets/cat_4.png"),
];

/// Icon color based on caffeine mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconColor {
    Default,
    Blue,    // Preventing system sleep
    Orange,  // Preventing display sleep
}

/// Tray icon handler
pub struct TrayIcon {
    hwnd: windows::Win32::Foundation::HWND,
    icon_id: u32,
    current_frame: usize,
    color: IconColor,
    last_update: Instant,
    animation_interval: Duration,
}

impl TrayIcon {
    /// Create a new tray icon handler
    pub fn new() -> Self {
        Self {
            hwnd: windows::Win32::Foundation::HWND::default(),
            icon_id: 1,
            current_frame: 0,
            color: IconColor::Default,
            last_update: Instant::now(),
            animation_interval: Duration::from_millis(200), // 5 fps base
        }
    }

    /// Create and show the tray icon
    pub fn create(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load the first frame as icon
        let hicon = self.load_frame_icon(0)?;
        
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: self.icon_id,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_USER + 1,
                hIcon: hicon,
                ..Default::default()
            };

            // Set tooltip text
            let tip = windows::core::w!("DashCat");
            nid.szTip[..tip.len()].copy_from_slice(tip);

            Shell_NotifyIconW(NIM_ADD, &mut nid);
        }

        Ok(())
    }

    /// Load a cat frame PNG and convert to HICON
    fn load_frame_icon(&self, frame: usize) -> Result<HICON, Box<dyn std::error::Error>> {
        let png_data = CAT_FRAMES[frame.min(4)];
        
        // Decode PNG
        let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
        let mut reader = decoder.read_info()?;
        
        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf)?;
        
        let info = reader.info();
        let width = info.width;
        let height = info.height;
        
        // Create a DIB section for the icon
        unsafe {
            let hdc = GetDC(HWND::default());
            
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32), // Top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits: *mut u32 = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits as *mut _ as _, None, 0);

            // Copy pixel data (convert RGBA to BGRA for Windows)
            for (i, pixel) in buf.chunks(4).enumerate() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = pixel[3];
                
                // Apply color tint if needed
                let (r, g, b) = match self.color {
                    IconColor::Blue => {
                        // Tint towards blue
                        (r / 3, g / 3, b.saturating_add(100))
                    }
                    IconColor::Orange => {
                        // Tint towards orange
                        (r.saturating_add(100), g / 2, b / 4)
                    }
                    IconColor::Default => (r, g, b),
                };

                if !bits.is_null() {
                    *bits.add(i) = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                }
            }

            ReleaseDC(HWND::default(), hdc);

            // Create mask bitmap
            let hmask = CreateBitmap(width as i32, height as i32, 1, 1, None);

            // Create icon
            let ii = ICONINFO {
                fIcon: true,
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hmask,
                hbmColor: hbitmap,
            };

            let hicon = CreateIconIndirect(&ii);

            // Clean up bitmaps
            DeleteObject(hbitmap);
            DeleteObject(hmask);

            Ok(hicon)
        }
    }

    /// Update the icon based on current system load
    pub fn update(&mut self, cpu_usage: f32, memory_usage: f32) {
        // Calculate frame interval based on load
        // Higher load = faster animation (lower interval)
        let max_load = cpu_usage.max(memory_usage);
        let fps = 2.0 + (max_load / 100.0) * 10.0; // 2-12 fps
        self.animation_interval = Duration::from_millis((1000.0 / fps) as u64);

        // Check if it's time to advance frame
        if self.last_update.elapsed() >= self.animation_interval {
            self.current_frame = (self.current_frame + 1) % 5;
            self.last_update = Instant::now();

            // Update the tray icon
            if let Ok(hicon) = self.load_frame_icon(self.current_frame) {
                self.set_icon(hicon);
            }
        }
    }

    /// Set the tray icon to a specific HICON
    fn set_icon(&self, hicon: HICON) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: self.icon_id,
                uFlags: NIF_ICON,
                hIcon: hicon,
                ..Default::default()
            };

            Shell_NotifyIconW(NIM_MODIFY, &mut nid);
        }
    }

    /// Set the icon color (caffeine mode)
    pub fn set_color(&mut self, color: IconColor) {
        if self.color != color {
            self.color = color;
            // Reload current frame with new color
            if let Ok(hicon) = self.load_frame_icon(self.current_frame) {
                self.set_icon(hicon);
            }
        }
    }

    /// Remove the tray icon
    pub fn remove(&self) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: self.icon_id,
                ..Default::default()
            };

            Shell_NotifyIconW(NIM_DELETE, &mut nid);
        }
    }
}

impl Default for TrayIcon {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove();
    }
}
