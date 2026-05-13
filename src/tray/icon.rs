//! Tray icon with PNG animation
use std::io::Cursor;

use windows::core::*;
use windows::Win32::Foundation::{HWND, BOOL};
use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject};
use windows::Win32::UI::Shell::{Shell_NotifyIconW, NOTIFYICONDATAW, NOTIFY_ICON_DATA_FLAGS, NOTIFY_ICON_MESSAGE};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, DestroyIcon, HICON, ICONINFO, WM_USER};

const WM_TRAYICON: u32 = WM_USER + 1;
const NIM_ADD: u32 = 0x00000000;
const NIM_DELETE: u32 = 0x00000002;
const NIM_MODIFY: u32 = 0x00000001;
const NIF_MESSAGE: u32 = 0x00000001;
const NIF_ICON: u32 = 0x00000002;
const NIF_TIP: u32 = 0x00000004;

static CAT_FRAMES: [&[u8]; 5] = [
    include_bytes!("../assets/cat_0.png"),
    include_bytes!("../assets/cat_1.png"),
    include_bytes!("../assets/cat_2.png"),
    include_bytes!("../assets/cat_3.png"),
    include_bytes!("../assets/cat_4.png"),
];

pub struct TrayIcon { current_frame: usize, hicon: Option<HICON> }

impl TrayIcon {
    pub fn new() -> Self { Self { current_frame: 0, hicon: None } }

    pub fn create(&mut self, hwnd: HWND, id: usize) -> Result<()> {
        let icon = self.create_icon_from_frame(0)?;

        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd, uID: id as u32,
            uFlags: NOTIFY_ICON_DATA_FLAGS(NIF_ICON | NIF_MESSAGE | NIF_TIP),
            uCallbackMessage: WM_TRAYICON, hIcon: icon,
            szTip: [0; 128], ..Default::default()
        };

        // Dynamic tooltip with current stats
        let tip = "DashCat - System Monitor";
        let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
        for (i, &c) in tip_wide.iter().take(127).enumerate() { nid.szTip[i] = c; }

        unsafe { Shell_NotifyIconW(NOTIFY_ICON_MESSAGE(NIM_ADD), &nid); }
        self.hicon = Some(icon);
        Ok(())
    }

    pub fn remove(&mut self, hwnd: HWND, id: usize) -> Result<()> {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd, uID: id as u32, ..Default::default()
        };
        unsafe { Shell_NotifyIconW(NOTIFY_ICON_MESSAGE(NIM_DELETE), &nid); }
        if let Some(icon) = self.hicon.take() { unsafe { let _ = DestroyIcon(icon); } }
        Ok(())
    }

    pub fn update(&mut self, hwnd: HWND, id: usize, cpu: f32, mem: f32) {
        self.current_frame = (self.current_frame + 1) % 5;
        if let Some(old) = self.hicon.take() { unsafe { let _ = DestroyIcon(old); } }

        if let Ok(icon) = self.create_icon_from_frame(self.current_frame) {
            self.hicon = Some(icon);

            // Update tooltip with current values
            let tip = format!("DashCat\nCPU: {:.0}% | Mem: {:.0}%", cpu, mem);
            let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd, uID: id as u32,
                uFlags: NOTIFY_ICON_DATA_FLAGS(NIF_ICON | NIF_TIP),
                hIcon: icon,
                szTip: [0; 128],
                ..Default::default()
            };

            for (i, &c) in tip_wide.iter().take(127).enumerate() { nid.szTip[i] = c; }

            unsafe { Shell_NotifyIconW(NOTIFY_ICON_MESSAGE(NIM_MODIFY), &nid); }
        }
    }

    fn create_icon_from_frame(&self, frame: usize) -> Result<HICON> {
        let data = CAT_FRAMES[frame.min(4)];
        let decoder = png::Decoder::new(Cursor::new(data));
        let mut reader = decoder.read_info().map_err(|_| Error::from(HRESULT(-1)))?;

        let info = reader.info();
        let (w, h) = (info.width as i32, info.height as i32);

        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).map_err(|_| Error::from(HRESULT(-1)))?;

        // RGBA to BGRA
        for i in 0..(w * h) as usize {
            let (r, g, b) = (buf[i*4], buf[i*4+1], buf[i*4+2]);
            buf[i*4] = b; buf[i*4+1] = g; buf[i*4+2] = r;
        }

        unsafe { create_icon_from_bitmap(&buf, w, h) }
    }
}

unsafe fn create_icon_from_bitmap(data: &[u8], w: i32, h: i32) -> Result<HICON> {
    let hbm_color = CreateBitmap(w, h, 1, 32, Some(data.as_ptr() as *const _));
    if hbm_color.is_invalid() { return Err(Error::from(HRESULT(-1))); }

    let mask = vec![0u8; (w * h) as usize];
    let hbm_mask = CreateBitmap(w, h, 1, 1, Some(mask.as_ptr() as *const _));
    if hbm_mask.is_invalid() { DeleteObject(hbm_color); return Err(Error::from(HRESULT(-2))); }

    let info = ICONINFO { fIcon: BOOL(1), xHotspot: 0, yHotspot: 0, hbmMask: hbm_mask, hbmColor: hbm_color };
    let hicon = CreateIconIndirect(&info)?;

    DeleteObject(hbm_color);
    DeleteObject(hbm_mask);

    Ok(hicon)
}

impl Default for TrayIcon { fn default() -> Self { Self::new() } }