fn main() {
    // Daftarkan perintah aplikasi agar bisa diberi izin di capabilities.
    // Tanpa ini, halaman https (mankeerom.yzz.me) tidak boleh memanggil request_exit.
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&["login", "open_exam", "request_exit"]),
        ),
    )
    .expect("gagal membuat izin perintah");
}
