#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Logika kiosk dan keluar

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

mod winkeys;
use winkeys::{force_max_volume, install_keyboard_hook};
use rodio::Source;
const HOME_URL: &str = "https://mankeerom.yzz.me/ujian";
const PASSWORD: &str = "MAN123";

static APP: OnceLock<AppHandle> = OnceLock::new();
static EXAM_STARTED: AtomicBool = AtomicBool::new(false);
static ALLOW_CLOSE: AtomicBool = AtomicBool::new(false);
static FOCUS_ALARM_ON: AtomicBool = AtomicBool::new(false);

// Disuntikkan ke setiap halaman https: navbar, kotak password keluar, dan info jaringan lemah.
const NAV_JS: &str = r#"
(function () {
  if (window.top !== window) return;
  if (location.protocol !== 'https:') return;

  // Tema mengikuti Windows: gelap -> hitam pekat, terang -> putih
  function isDark() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  // Kotak password dibuat sendiri, karena window.prompt tidak didukung di Tauri
  window.__openExit = function () {
    if (document.getElementById('__exit_box') || !document.body) return;
    var d = isDark();
    var cardBg = d ? '#111827' : '#FFFFFF';
    var tx = d ? '#F9FAFB' : '#111827';
    var fieldBg = d ? '#1F2937' : '#FFFFFF';
    var line = d ? '#6B7280' : '#9CA3AF';
    var cancelBg = d ? '#1F2937' : '#FFFFFF';
    var cancelLine = d ? '#374151' : '#D1D5DB';
    var o = document.createElement('div');
    o.id = '__exit_box';
    o.style.cssText = 'position:fixed;inset:0;z-index:2147483647;background:rgba(0,0,0,.6);' +
      'display:flex;align-items:center;justify-content:center;font-family:sans-serif';
    o.innerHTML = '<div style="background:' + cardBg + ';border:1px solid ' + cancelLine + ';border-radius:16px;padding:24px;width:300px;box-sizing:border-box">' +
      '<div style="font-size:16px;font-weight:600;color:' + tx + ';margin-bottom:12px">Masukkan password untuk keluar</div>' +
      '<input id="__exit_pwd" type="password" style="width:100%;box-sizing:border-box;padding:10px 12px;border:1px solid ' + line + ';border-radius:10px;font-size:15px;background:' + fieldBg + ';color:' + tx + '">' +
      '<div id="__exit_err" style="color:#DC2626;font-size:13px;min-height:18px;margin-top:6px"></div>' +
      '<div style="display:flex;gap:8px;margin-top:8px">' +
      '<button id="__exit_cancel" style="flex:1;padding:10px;border:1px solid ' + cancelLine + ';background:' + cancelBg + ';color:' + tx + ';border-radius:10px;cursor:pointer">Batal</button>' +
      '<button id="__exit_ok" style="flex:1;padding:10px;border:none;background:#DC2626;color:#fff;border-radius:10px;cursor:pointer">Keluar</button>' +
      '</div></div>';
    document.body.appendChild(o);
    var input = document.getElementById('__exit_pwd');
    input.focus();

    function close() { o.remove(); }
    function submit() {
      window.__TAURI__.core.invoke('request_exit', { pwd: input.value }).then(function (ok) {
        if (!ok) {
          document.getElementById('__exit_err').textContent = 'Password salah';
          input.value = '';
          input.focus();
        }
      }).catch(function (e) {
        document.getElementById('__exit_err').textContent = 'Gagal keluar: ' + e;
      });
    }
    document.getElementById('__exit_cancel').onclick = close;
    document.getElementById('__exit_ok').onclick = submit;
    input.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') submit();
      if (e.key === 'Escape') close();
    });
  };

  function pickBg() {
    var m = document.querySelector('meta[name="theme-color"]');
    if (m && m.content) return m.content;
    var sels = ['header', 'nav', '.navbar', '.topbar', '.header'];
    for (var i = 0; i < sels.length; i++) {
      var el = document.querySelector(sels[i]);
      if (el) {
        var c = getComputedStyle(el).backgroundColor;
        if (c && c !== 'rgba(0, 0, 0, 0)' && c !== 'transparent') return c;
      }
    }
    return null;
  }

  // Warna navbar mengikuti tema Windows: gelap -> hitam pekat, terang -> putih
  function themeBg() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? '#000000' : '#FFFFFF';
  }

  function textColorFor(bg) {
    var probe = document.createElement('div');
    probe.style.color = bg;
    document.body.appendChild(probe);
    var rgb = getComputedStyle(probe).color;
    probe.remove();
    var n = (rgb.match(/[\d.]+/g) || [0, 0, 0]).map(Number);
    var lum = (0.299 * n[0] + 0.587 * n[1] + 0.114 * n[2]) / 255;
    return lum > 0.6 ? '#111111' : '#ffffff';
  }

  function build() {
    if (document.getElementById('__exam_nav') || !document.body) return;
    var bg = themeBg();
    var dark = bg === '#000000';
    var fg = dark ? '#FFFFFF' : '#111111';
    var btnBg = dark ? '#1F2937' : '#E5E7EB';
    var line = dark ? '#374151' : '#9CA3AF';
    var bar = document.createElement('div');
    bar.id = '__exam_nav';
    bar.style.cssText = 'position:fixed;top:0;left:0;right:0;height:36px;z-index:2147483647;' +
      'display:flex;align-items:center;gap:4px;padding:0 6px;box-sizing:border-box;' +
      'background:' + bg + ';border-bottom:1px solid ' + line + ';font-family:sans-serif';

    function btn(label, fn) {
      var b = document.createElement('button');
      b.textContent = label;
      b.style.cssText = 'height:26px;min-width:34px;padding:0 8px;border:none;border-radius:8px;cursor:pointer;' +
        'font-size:13px;font-weight:600;color:' + fg + ';background:' + btnBg + ';border:1px solid ' + line;
      b.onclick = fn;
      bar.appendChild(b);
    }

    btn('\u25C0', function () { history.back(); });
    btn('\u25B6', function () { history.forward(); });
    btn('\u27F3', function () { location.reload(); });
    btn('\u2302', function () { location.href = 'https://mankeerom.yzz.me/ujian'; });
    btn('\u2715', function () { window.__openExit(); });

    document.body.appendChild(bar);
    document.body.style.paddingTop = '36px';
  }

  function showOffline() {
    if (document.getElementById('__offline') || !document.body) return;
    var o = document.createElement('div');
    o.id = '__offline';
    var d = isDark();
    o.style.cssText = 'position:fixed;inset:0;z-index:2147483646;background:' + (d ? '#000000' : '#ffffff') + ';' +
      'display:flex;flex-direction:column;align-items:center;justify-content:center;' +
      'font-family:sans-serif;color:' + (d ? '#E5E7EB' : '#374151') + ';text-align:center';
    o.innerHTML = '<div style="font-size:48px">&#128246;</div>' +
      '<div style="font-size:18px;font-weight:600;margin-top:12px">Jaringan lemah</div>' +
      '<div style="font-size:14px;color:' + (d ? '#9CA3AF' : '#6B7280') + ';margin-top:6px">Menghubungkan kembali...</div>';
    document.body.appendChild(o);
  }

  document.addEventListener('keydown', function (e) {
    if (e.ctrlKey && e.shiftKey && e.altKey && (e.key === 'K' || e.key === 'k')) {
      e.preventDefault();
      window.__openExit();
    }
  }, true);

  window.addEventListener('offline', showOffline);
  window.addEventListener('online', function () { location.reload(); });
  document.addEventListener('DOMContentLoaded', function () {
    build();
    if (!navigator.onLine) showOffline();
  });
  setInterval(function () {
    if (!document.getElementById('__exam_nav')) build();
  }, 2000);
})();

  // Dropdown dibuat sendiri, supaya popup native tidak mengambil fokus
  var __selPanel = null;
  function __closeSel() { if (__selPanel) { __selPanel.remove(); __selPanel = null; } }
  document.addEventListener('mousedown', function (e) {
    var sel = e.target.closest ? e.target.closest('select') : null;
    if (!sel) {
      if (__selPanel && !__selPanel.contains(e.target)) __closeSel();
      return;
    }
    if (sel.disabled) return;
    e.preventDefault();
    sel.focus();
    __closeSel();
    var dark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    var bg = dark ? '#1e1e1e' : '#ffffff';
    var tx = dark ? '#f2f2f2' : '#111111';
    var line = dark ? '#444444' : '#cccccc';
    var hover = dark ? '#2d5fa8' : '#dbeafe';
    var p = document.createElement('div');
    p.style.cssText = 'position:fixed;z-index:2147483647;background:' + bg + ';color:' + tx +
      ';border:1px solid ' + line + ';border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.3);' +
      'overflow:auto;max-height:50vh;font:14px system-ui,sans-serif;box-sizing:border-box;';
    var r = sel.getBoundingClientRect();
    p.style.left = r.left + 'px';
    p.style.width = Math.max(r.width, 160) + 'px';
    if (r.bottom + 260 > window.innerHeight && r.top > 260) {
      p.style.bottom = (window.innerHeight - r.top + 4) + 'px';
    } else {
      p.style.top = (r.bottom + 4) + 'px';
    }
    Array.prototype.forEach.call(sel.options, function (opt, i) {
      var item = document.createElement('div');
      item.textContent = opt.text;
      item.style.cssText = 'padding:10px 12px;cursor:pointer;' +
        (opt.selected ? 'font-weight:600;' : '') + (opt.disabled ? 'opacity:.5;' : '');
      item.addEventListener('mouseenter', function () { item.style.background = hover; });
      item.addEventListener('mouseleave', function () { item.style.background = ''; });
      item.addEventListener('mousedown', function (ev) {
        ev.preventDefault();
        if (opt.disabled) return;
        sel.selectedIndex = i;
        sel.dispatchEvent(new Event('input', { bubbles: true }));
        sel.dispatchEvent(new Event('change', { bubbles: true }));
        __closeSel();
      });
      p.appendChild(item);
    });
    document.body.appendChild(p);
    __selPanel = p;
  }, true);
  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape') __closeSel();
  }, true);
  window.addEventListener('scroll', __closeSel, true);
  window.addEventListener('resize', __closeSel);

"#;

// ---------- Perintah yang dipanggil dari halaman ----------

#[tauri::command]
fn login(pwd: String) -> bool {
    pwd == PASSWORD
}

#[tauri::command]
fn open_exam(app: AppHandle) {
    EXAM_STARTED.store(true, Ordering::SeqCst);
    if let Some(w) = app.get_webview_window("main") {
        // Setelah login berhasil, jendela berubah jadi fullscreen dan selalu di atas
        let _ = w.set_resizable(false);
        let _ = w.set_fullscreen(true);
        let _ = w.set_always_on_top(true);
        if let Ok(url) = HOME_URL.parse::<tauri::Url>() {
            let _ = w.navigate(url);
        }
    }
}

#[tauri::command]
fn request_exit(app: AppHandle, pwd: String) -> bool {
    if pwd != PASSWORD {
        return false;
    }
    ALLOW_CLOSE.store(true, Ordering::SeqCst);
    FOCUS_ALARM_ON.store(false, Ordering::SeqCst);
    log("password keluar benar");
    // Pengaman: aplikasi pasti tertutup paling lambat 25 detik, walau alarm atau exit bermasalah
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(25));
        log("watchdog: paksa keluar");
        std::process::exit(0);
    });
    play_alarm_then_exit(app);
    true
}

// ---------- Alarm ----------

// Suara alarm ditanam di dalam .exe saat build, jadi tidak ada file terpisah yang bisa diubah orang.
// Untuk mengganti suara: ganti file alarm.wav di folder utama repository, lalu build ulang.
static ALARM_BYTES: &[u8] = include_bytes!("../../alarm.wav");

// Alarm berulang saat jendela ujian kehilangan fokus (Linux). Berhenti saat keluar dengan password.
fn start_focus_alarm() {
    if FOCUS_ALARM_ON.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                if let Ok(src) = rodio::Decoder::new(std::io::Cursor::new(ALARM_BYTES)) {
                    sink.append(src.repeat_infinite());
                }
                while FOCUS_ALARM_ON.load(Ordering::SeqCst) {
                    force_max_volume();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                sink.stop();
            }
        }
    });
}

fn play_alarm_then_exit(app: AppHandle) {
    std::thread::spawn(move || {
        // Volume dipaksa maksimum terus-menerus selama alarm berbunyi
        let playing = std::sync::Arc::new(AtomicBool::new(true));
        let flag = playing.clone();
        std::thread::spawn(move || {
            while flag.load(Ordering::SeqCst) {
                force_max_volume();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
        if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                // Alarm berbunyi 1 kali penuh
                for _ in 0..1 {
                    if let Ok(src) = rodio::Decoder::new(std::io::Cursor::new(ALARM_BYTES)) {
                        sink.append(src);
                    }
                }
                sink.sleep_until_end();
            }
        }
        playing.store(false, Ordering::SeqCst);
        app.exit(0);
    });
}

// ---------- Tombol Windows dan Ctrl+Shift+Alt+K ----------
// Dipanggil langsung dari user32/kernel32 (tanpa crate windows) agar tidak ada konflik versi.


fn log(msg: &str) {
    use std::io::Write;
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("log.txt")) {
            let _ = writeln!(f, "{} {}", chrono_free_time(), msg);
        }
    }
}

fn chrono_free_time() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

// ---------- Main ----------

fn main() {
    // Driver NVIDIA: render DMABUF dan compositing WebKit dimatikan (penyebab umum halaman tersendat)
    if std::path::Path::new("/proc/driver/nvidia").exists() {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![login, open_exam, request_exit])
        .on_window_event(|window, event| {
            // Setelah ujian dimulai, tutup lewat X atau Alt+F4 dibatalkan kecuali sudah lewat password
            if let WindowEvent::CloseRequested { api, .. } = event {
                if EXAM_STARTED.load(Ordering::SeqCst) && !ALLOW_CLOSE.load(Ordering::SeqCst) {
                    api.prevent_close();
                }
            }
            // Jendela kehilangan fokus saat ujian: alarm berbunyi dan jendela diminta kembali ke depan
            if let WindowEvent::Focused(false) = event {
                if EXAM_STARTED.load(Ordering::SeqCst) && !ALLOW_CLOSE.load(Ordering::SeqCst) {
                    start_focus_alarm();
                    let _ = window.set_focus();
                }
            }
        })
        .setup(|app| {
            let _ = APP.set(app.handle().clone());
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Ujian MAN")
                // Halaman login: jendela Windows biasa (bisa dipindah, diubah ukuran, dimaksimalkan)
                .inner_size(440.0, 580.0)
                .min_inner_size(400.0, 540.0)
                .center()
                .initialization_script(NAV_JS)
                .build()?;
            install_keyboard_hook();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
