//! Windows Registry utilities for auto-start

use windows::core::*;
use windows::Win32::System::Registry::*;

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("DashCat");

/// Enable auto-start
pub fn enable_auto_start() -> bool {
    unsafe {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_str = exe_path.to_string_lossy().to_string();
        let value_wide: Vec<u16> = exe_str.encode_utf16().chain(std::iter::once(0)).collect();

        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_WRITE, &mut hkey).0 != 0 {
            return false;
        }

        let value_bytes: &[u8] = std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2);
        let result = RegSetValueExW(hkey, APP_NAME, 0, REG_SZ, Some(value_bytes)).0 == 0;
        RegCloseKey(hkey);
        result
    }
}

/// Disable auto-start
pub fn disable_auto_start() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_WRITE, &mut hkey).0 != 0 {
            return false;
        }

        let result = RegDeleteValueW(hkey, APP_NAME).0 == 0;
        RegCloseKey(hkey);
        result
    }
}

/// Check if auto-start is enabled
pub fn is_auto_start_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_READ, &mut hkey).0 != 0 {
            return false;
        }

        let mut buffer = [0u16; 512];
        let mut size = buffer.len() as u32 * 2;
        let result = RegGetValueW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            APP_NAME,
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr() as *mut _),
            Some(&mut size),
        ).0 == 0;

        RegCloseKey(hkey);
        result
    }
}