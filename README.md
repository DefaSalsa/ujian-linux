<!-- Panduan versi Linux -->
# Ujian MAN untuk Linux

Versi Linux dari aplikasi ujian. Dibuat dengan Tauri (Rust + WebKitGTK). Build dilakukan langsung di komputer Linux Anda, tanpa GitHub.

## Sistem yang didukung

- **Debian 12 (bookworm) atau yang lebih baru**, termasuk Debian 13 dan 14.
- **Ubuntu 22.04 atau yang lebih baru.**
- Distro lain yang punya `libwebkit2gtk-4.1` dan glibc 2.35 atau lebih baru.

Debian 11 dan yang lebih lama tidak didukung, karena tidak punya `libwebkit2gtk-4.1`.

## Cara membuat aplikasi Linux (lewat GitHub)

Tidak perlu install apa pun di komputer. Build dijalankan di GitHub.

1. Buat repository baru di GitHub (sebaiknya Private).
2. Upload **seluruh isi** folder ini, termasuk folder `.github` (aktifkan "tampilkan file tersembunyi" jika tidak terlihat).
3. Buka tab **Actions**, pilih **Build Linux**, tunggu tanda centang hijau (sekitar 10 sampai 20 menit pertama kali).
4. Klik run tersebut, lalu unduh artifact **UjianMAN**. Hasilnya berupa `UjianMAN.zip`.
5. Di dalam zip ada dua file:
   - File `.deb` untuk Ubuntu dan Debian.
   - File `.AppImage` untuk hampir semua distro Linux.

## Cara install

**Dari file .deb (Ubuntu dan Debian):**
```bash
sudo apt install ./NAMA_FILE.deb
```
Ganti `NAMA_FILE.deb` dengan nama file `.deb` yang didownload. Kalau nama mengandung spasi, tulis di dalam tanda kutip.
Setelah itu aplikasi muncul di menu aplikasi dengan nama Ujian MAN.

**Dari AppImage (semua distro):**
```bash
chmod +x NAMA_FILE.AppImage
./NAMA_FILE.AppImage
```
Ganti `NAMA_FILE.AppImage` dengan nama file `.AppImage` yang didownload.

## Konfigurasi (build.env)
Semua pengaturan ada di file `build.env`. Ubah nilainya, lalu upload ulang dan jalankan build lagi.

| Kunci | Fungsi | Contoh |
|---|---|---|
| `APP_NAME` | Nama aplikasi (judul jendela, Start menu, nama installer) | `Ujian MAN` |
| `APP_ID` | ID unik aplikasi | `id.dstech.ujianmankeerom` |
| `PASSWORD` | Password untuk masuk dan keluar | `MAN123` |
| `HOME_URL` | Alamat halaman ujian | `https://mankeerom.yzz.me/ujian` |
| `LOGIN_TITLE` | Judul di halaman login | `Ujian Online MAN Keerom` |
| `LOGO` | File logo (PNG, sebaiknya persegi) untuk ikon dan halaman login | `logo.png` |
| `ARTIFACT_NAME` | Nama zip hasil download | `UjianMAN` |

Catatan:
- Spasi di nama file hasil build otomatis diganti `_`, misalnya `Ujian MAN` menjadi `Ujian_MAN_1.0.0_x64-setup.exe`.
- Mengubah `APP_ID` membuat data lama (sesi login) tidak ikut terbawa.
- Format setiap baris: `KUNCI=NILAI`, tanpa tanda kutip.

## Suara alarm

Suara alarm ditanam di dalam aplikasi saat build. Untuk mengganti suara:
1. Siapkan file suara dengan nama `alarm.wav` (format WAV).
2. Ganti file `alarm.wav` di folder utama, sejajar dengan folder `src` dan `src-tauri`.
3. Upload ulang ke GitHub dan jalankan **Build Linux** lagi.

Volume alarm dinaikkan lewat PipeWire atau PulseAudio (`pactl`). Biasanya sudah ada di Ubuntu dan Debian.

## Cara kerja

- Halaman login tampil sebagai jendela biasa di tengah layar. Setelah password benar, aplikasi berubah jadi layar penuh dan selalu di atas, lalu halaman ujian dibuka.
- Navbar: Mundur, Maju, Refresh, Home, dan ✕ (✕ minta password).
- Keluar dengan password: tekan ✕ atau Ctrl+Shift+Alt+K, lalu masukkan password MAN123. Alarm berbunyi 1 kali penuh, lalu aplikasi tertutup.
- Saat ujian berjalan, tombol X jendela dan Alt+F4 tidak menutup aplikasi.
- Saat koneksi putus di halaman ujian, muncul "Jaringan lemah" dan aplikasi menyambung lagi saat koneksi kembali.

## Batasan di Linux

- **Saat jendela ujian kehilangan fokus** (misalnya peserta pindah aplikasi), alarm berbunyi terus dan aplikasi mencoba kembali ke depan. Alarm berhenti setelah keluar dengan password. Di Wayland, aplikasi tidak selalu bisa kembali ke depan.
- **Tombol Super dan Alt+Tab belum diblokir.** Di Linux, aplikasi biasa tidak bisa memblokir tombol-tombol itu, terutama di Wayland. Versi Windows memblokirnya lewat fitur khusus Windows, yang tidak tersedia di Linux.
- **Ctrl+Shift+Alt+K hanya jalan saat jendela aplikasi sedang aktif.**
- **Tidak ada pin layar,** sama seperti versi Windows.
- **Tidak ada alarm saat keluar paksa** dan layar penutup seperti di Android. Hanya keluar dengan password yang membunyikan alarm.
- **Kode Linux belum pernah diuji di desktop Linux.** Jika build gagal, kirim pesan error dari tab Actions.
