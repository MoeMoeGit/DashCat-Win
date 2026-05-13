//! Tray icon management

use windows::core::*;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Cat animation frames (embedded)
const CAT_FRAMES: [&[u8]; 5] = [
    include_bytes!("../assets/cat_0.png"),
    include_bytes!("../assets/cat_1.png"),
    include_bytes!("../assets/cat_2.png"),
    include_bytes!("../assets/cat_3.png"),
    include_bytes!("../assets/cat_4.png"),
];

/// Tray icon handler
pub struct TrayIcon {
    current_frame: usize,
}

impl TrayIcon {
    /// Create a new tray icon handler
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }

    /// Create and show the tray icon
    pub unsafe fn create(&self, hwnd: HWND, id: u32) -> Result<()> {
        let hicon = self.load_icon(0)?;

        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: id,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_USER + 1,
            hIcon: hicon,
            szTip: {
                let mut arr = [0u16; 128];
                let tip = "DashCat\0";
                for (i, c) in tip.encode_utf16().take(127).enumerate() {
                    arr[i] = c;
                }
                arr
            },
            ..std::mem::zeroed()
        };

        Shell_NotifyIconW(NIM_ADD, &nid);

        Ok(())
    }

    /// Load a cat frame as icon
    unsafe fn load_icon(&self, frame: usize) -> Result<HICON> {
        // Decode PNG
        let png_data = CAT_FRAMES[frame.min(4)];
        let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
        let mut reader = decoder.read_info().map_err(|e| Error::from(HRESULT(-1)))?;
        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf).map_err(|e| Error::from(HRESULT(-1)))?;

        let info = reader.info();
        let width = info.width;
        let height = info.height;

        // Create DIB section
        let hdc = GetDC(HWND::default());

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..std::mem::zeroed()
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;

        // Copy RGBA to BGRA
        if !bits.is_null() {
            let bits_ptr = bits as *mut u8;
            for (i, pixel) in buf.chunks(4).enumerate() {
                let offset = i * 4;
                *bits_ptr.add(offset) = pixel[2];     // B
                *bits_ptr.add(offset + 1) = pixel[1]; // G
                *bits_ptr.add(offset + 2) = pixel[0]; // R
                *bits_ptr.add(offset + 3) = pixel[3]; // A
            }
        }

        ReleaseDC(HWND::default(), hdc);

        // Create mask
        let hmask = CreateBitmap(width as i32, height as i32, 1, 1, None);

        // Create icon
        let ii = ICONINFO {
            fIcon: BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hmask,
            hbmColor: hbitmap,
        };

        let hicon = CreateIconIndirect(&ii)?;

        // Cleanup
        let _ = DeleteObject(hbitmap);
        let _ = DeleteObject(hmask);

        Ok(hicon)
    }

    /// Update the icon with a new frame
    pub unsafe fn update(&mut self, hwnd: HWND, id: u32, _cpu: f32, _memory: f32) {
        // Advance frame
        self.current_frame = (self.current_frame + 1) % 5;

        // Load new icon
        if let Ok(hicon) = self.load_icon(self.current_frame) {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: id,
                uFlags: NIF_ICON,
                hIcon: hicon,
                ..std::mem::zeroed()
            };

            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    /// Remove the tray icon
    pub unsafe fn remove(&self, hwnd: HWND, id: u32) -> Result<()> {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: id,
            ..std::mem::zeroed()
        };

        Shell_NotifyIconW(NIM_DELETE, &nid);

        Ok(())
    }
}

impl Default for TrayIcon {
    fn default() -> Self {
        Self::new()
    }
}