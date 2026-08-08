//! Cycle Finance açılış ekranı — FIGlet ASCII sanatı (harf harf animasyon).
//!
//! Terminal boyutunu algılar, "CYCLE FINANCE" metnini standart FIGlet fontuyla
//! tam ortalanmış şekilde çizer ve her döngüde bir harf ekleyerek animasyon
//! üretir.

use figlet_rs::FIGfont;
use std::io::{stdout, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use terminal_size::{terminal_size, Height, Width};

/// Varsayılan animasyon hızı (ms)
const ANIM_SPEED_MS: u64 = 180;

/// Varsayılan metin
const SPLASH_TEXT: &str = "CYCLE FINANCE";

/// Matrix yeşili (true color): #00FF41
const MATRIX_GREEN: &str = "\x1B[38;2;0;255;65m";
/// Siyah arkaplan
const BG_BLACK: &str = "\x1B[48;2;0;0;0m";
/// Renk sıfırla
const RESET: &str = "\x1B[0m";
/// Terminali tamamen temizle + imleci başa al + imleci gizle
const CLEAR: &str = "\x1B[2J\x1B[1;1H\x1B[?25l";

/// Açılış ekranını gösterir. `speed_ms` 0 verilirse varsayılan (180ms) kullanılır.
pub fn show_splash() {
    show_splash_with(SPLASH_TEXT, ANIM_SPEED_MS);
}

/// Özel metin ve hız ile açılış ekranı gösterir.
pub fn show_splash_with(metin: &str, speed_ms: u64) {
    let speed = if speed_ms == 0 { ANIM_SPEED_MS } else { speed_ms };
    let chars: Vec<char> = metin.chars().collect();
    let toplam_harf = chars.len();

    // FIGlet standart fontunu yükle
    let font = FIGfont::standard().expect("FIGlet standart font yüklenemedi!");

    // Terminal boyutlarını al
    let (term_width, term_height) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w as usize, h as usize)
    } else {
        (80, 24) // Varsayılan boyut
    };

    // Tam figure'ın yüksekliği (dikey ortalama için)
    let tam_figure = font.convert(metin).expect("FIGlet dönüşüm başarısız!");
    let tam_cikti = tam_figure.to_string();
    let fig_yukseklik = tam_cikti.lines().count();

    let dikey_bosluk = if term_height > fig_yukseklik {
        (term_height - fig_yukseklik) / 2
    } else {
        0
    };

    // Animasyon: her seferinde 1 harf ekle
    let mut out = stdout();
    for i in 1..=toplam_harf {
        // Terminali temizle (siyah arkaplan), imleci başa al
        if write!(out, "{CLEAR}{BG_BLACK}").is_err() || out.flush().is_err() {
            return; // pipe kapandı (ör. head), sessizce çık
        }

        let kismi_metin: String = chars[0..i].iter().collect();
        let figure = font.convert(&kismi_metin).expect("FIGlet dönüşüm başarısız!");
        let cikti = figure.to_string();

        // Dikey ortalama
        for _ in 0..dikey_bosluk {
            if writeln!(out).is_err() {
                return;
            }
        }

        // Yatay ortalama + matrix yeşili ile yazdır
        for satir in cikti.lines() {
            let satir_uzunluk = satir.len();
            let yatay_bosluk = if term_width > satir_uzunluk {
                (term_width - satir_uzunluk) / 2
            } else {
                0
            };
            if writeln!(out, "{}{MATRIX_GREEN}{}{RESET}", " ".repeat(yatay_bosluk), satir).is_err() {
                return;
            }
        }

        if out.flush().is_err() {
            return;
        }
        sleep(Duration::from_millis(speed));
    }

    // İmleci tekrar göster
    let _ = write!(out, "\x1B[?25h{RESET}\n");
    let _ = out.flush();
    exit(0);
}
