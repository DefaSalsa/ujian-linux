// Fitur khusus Windows: tombol Windows, Ctrl+Shift+Alt+K, Alt+Tab, dan volume.
// Di Linux, bagian Windows dinonaktifkan. Volume dinaikkan lewat pactl, dan
// shortcut Ctrl+Shift+Alt+K ditangani dari halaman (NAV_JS).

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use tauri::Manager;
    use crate::{log, APP};

    const WH_KEYBOARD_LL: i32 = 13;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    const VK_LWIN: u32 = 0x5B;
    const VK_RWIN: u32 = 0x5C;
    const VK_K: u32 = 0x4B;
    const VK_TAB: u32 = 0x09;
    const VK_ESCAPE: u32 = 0x1B;
    const VK_VOLUME_UP: u8 = 0xAF;
    const KEYEVENTF_KEYUP: u32 = 0x0002;

    #[repr(C)]
    struct KbdLlHookStruct {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }

    #[repr(C)]
    struct Msg {
        hwnd: *mut c_void,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    type HookProc = unsafe extern "system" fn(i32, usize, isize) -> isize;

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowsHookExW(id: i32, f: HookProc, hmod: *mut c_void, tid: u32) -> *mut c_void;
        fn CallNextHookEx(h: *mut c_void, code: i32, w: usize, l: isize) -> isize;
        fn GetMessageW(msg: *mut Msg, h: *mut c_void, min: u32, max: u32) -> i32;
        fn TranslateMessage(msg: *const Msg) -> i32;
        fn DispatchMessageW(msg: *const Msg) -> isize;
        fn GetAsyncKeyState(vk: i32) -> i16;
        fn keybd_event(bvk: u8, bscan: u8, dwflags: u32, dwextrainfo: usize);
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    fn key_down(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: usize, lparam: isize) -> isize {
        if code == 0 {
            let kb = &*(lparam as *const KbdLlHookStruct);
            let vk = kb.vk_code;

            // Tombol Windows diblokir, Start menu tidak terbuka
            if vk == VK_LWIN || vk == VK_RWIN {
                log("tombol windows diblokir");
                return 1;
            }

            let is_down = wparam == WM_KEYDOWN || wparam == WM_SYSKEYDOWN;

            // Alt+Tab, Alt+Esc, dan Ctrl+Esc diblokir agar tidak pindah aplikasi atau membuka Start
            if is_down && ((vk == VK_TAB && key_down(VK_MENU)) || (vk == VK_ESCAPE && (key_down(VK_MENU) || key_down(VK_CONTROL)))) {
                log("pindah aplikasi diblokir");
                return 1;
            }
            if is_down && vk == VK_K && key_down(VK_CONTROL) && key_down(VK_SHIFT) && key_down(VK_MENU) {
                log("kombinasi keluar ditekan");
                if let Some(app) = APP.get() {
                    if let Some(w) = app.get_webview_window("main") {
                        let r = w.eval("if (window.__openExit) window.__openExit();");
                        log(&format!("eval: {:?}", r.is_ok()));
                    }
                }
                return 1;
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }

    pub fn install_keyboard_hook() {
        std::thread::spawn(|| unsafe {
            let hmod = GetModuleHandleW(std::ptr::null());
            let h = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_proc, hmod, 0);
            if h.is_null() {
                log(&format!("hook GAGAL, error={}", GetLastError()));
            } else {
                log("hook terpasang");
            }
            let mut msg = Msg {
                hwnd: std::ptr::null_mut(),
                message: 0,
                wparam: 0,
                lparam: 0,
                time: 0,
                pt_x: 0,
                pt_y: 0,
            };
            loop {
                let r = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if r <= 0 {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
    }


    // Menekan tombol volume naik banyak kali (Windows naik 2% per tekan) sampai volume maksimum
    pub fn force_max_volume() {
        unsafe {
            for _ in 0..50 {
                keybd_event(VK_VOLUME_UP, 0, 0, 0);
                keybd_event(VK_VOLUME_UP, 0, KEYEVENTF_KEYUP, 0);
            }
        }
    }


}

#[cfg(not(windows))]
mod imp {
    use std::process::Command;

    pub fn install_keyboard_hook() {}

    pub fn force_max_volume() {
        let _ = Command::new("pactl").args(["set-sink-mute", "@DEFAULT_SINK@", "0"]).status();
        let _ = Command::new("pactl").args(["set-sink-volume", "@DEFAULT_SINK@", "100%"]).status();
    }
}

pub use imp::{force_max_volume, install_keyboard_hook};
