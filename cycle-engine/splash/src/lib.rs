//! Cycle Finance açılış ekranı — FIGlet ASCII sanatı + yükleme çubuğu.
//!
//! "CYCLE FINANCE" yazısı matrix yeşili ile harf harf çizilir; altında bir
//! yükleme çubuğu tam 3 saniyede dolar. Yazı ve çubuk senkron ilerler:
//! çubuk %100 olduğunda yazı da tam haline ulaşır. Çubuk bitince kullanıcı
//! Enter'a basar ve sistem açılır (binary çıkar).

use figlet_rs::FIGfont;
use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use terminal_size::{terminal_size, Height, Width};

/// Toplam yükleme süresi (ms)
const LOAD_MS: u64 = 3000;

/// Varsayılan metin
const SPLASH_TEXT: &str = "CYCLE FINANCE";

/// Yükleme çubuğu genişliği (karakter)
const BAR_WIDTH: usize = 40;

/// Matrix yeşili (true color): #00FF41
const MATRIX_GREEN: &str = "\x1B[38;2;0;255;65m";
/// Siyah arkaplan
const BG_BLACK: &str = "\x1B[48;2;0;0;0m";
/// Renk sıfırla
const RESET: &str = "\x1B[0m";
/// Terminali tamamen temizle + imleci başa al + imleci gizle
const CLEAR: &str = "\x1B[2J\x1B[1;1H\x1B[?25l";

/// Açılış ekranını gösterir; Enter'a basılınca döner.
pub fn show_splash() {
    show_splash_with(SPLASH_TEXT, LOAD_MS);
}

/// Özel metin ve toplam yükleme süresi (ms) ile açılış ekranı gösterir.
/// Animasyon ve yükleme çubuğu senkron: süre bitince yazı tam halde olur.
pub fn show_splash_with(metin: &str, total_ms: u64) {
    let total = if total_ms == 0 { LOAD_MS } else { total_ms };
    let chars: Vec<char> = metin.chars().collect();
    let toplam_harf = chars.len();
    let step_ms = (total / toplam_harf as u64).max(1);

    // Arka planda futuristik ses çal
    let audio_child = std::process::Command::new("ffplay")
        .args(&[
            "-nodisp",
            "-autoexit",
            "/home/smhvz/Downloads/UI Sounds_ Futuristic sound effects example.mp3",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok();

    let font = FIGfont::standard().expect("FIGlet standart font yüklenemedi!");

    let (term_width, term_height) = if let Some((Width(w), Height(h))) = terminal_size() {
        (w as usize, h as usize)
    } else {
        (80, 24)
    };

    // Çubuğun satırı ekranın TAM dikey ortasında olsun.
    // Çubuk satırı = dikey_bosluk + fig_yukseklik + 1 (boş satır) → term_height / 2
    let tam_figure = font.convert(metin).expect("FIGlet dönüşüm başarısız!");
    let fig_yukseklik = tam_figure.to_string().lines().count();
    let orta = term_height / 2;
    let dikey_bosluk = orta.saturating_sub(fig_yukseklik + 1);

    let mut out = stdout();
    for i in 1..=toplam_harf {
        if write!(out, "{CLEAR}{BG_BLACK}").is_err() || out.flush().is_err() {
            return;
        }

        // Şu ana kadar biriken harfler
        let kismi_metin: String = chars[0..i].iter().collect();
        let figure = font.convert(&kismi_metin).expect("FIGlet dönüşüm başarısız!");
        let cikti = figure.to_string();

        for _ in 0..dikey_bosluk {
            if writeln!(out).is_err() {
                return;
            }
        }

        // Yazı (matrix yeşili, yatay ortalı)
        for satir in cikti.lines() {
            let yatay_bosluk = term_width.saturating_sub(satir.len()) / 2;
            if writeln!(out, "{}{MATRIX_GREEN}{}{RESET}", " ".repeat(yatay_bosluk), satir).is_err() {
                return;
            }
        }

        // Yükleme çubuğu (tam metnin ilerlemesiyle senkron)
        writeln!(out).ok();
        let percent = i * 100 / toplam_harf;
        let filled = percent * BAR_WIDTH / 100;
        let bar: String = format!("{}{} {}", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled), percent);
        let bar_yatay = term_width.saturating_sub(bar.len() + 2) / 2;
        if writeln!(out, "{}{MATRIX_GREEN}[{}]%{RESET}", " ".repeat(bar_yatay), bar).is_err() {
            return;
        }

        if out.flush().is_err() {
            return;
        }
        sleep(Duration::from_millis(step_ms));
    }

    // Çubuk tamamlandı — Enter bekle
    writeln!(out).ok();
    let msg = "▶ SİSTEMİ BAŞLATMAK İÇİN ENTER TUŞUNA BASINIZ";
    let msg_x = term_width.saturating_sub(msg.len()) / 2;
    let _ = writeln!(out, "{}{MATRIX_GREEN}{}{RESET}", " ".repeat(msg_x), msg);

    let _ = write!(out, "\x1B[?25h{RESET}");
    let _ = out.flush();

    // Enter bekle
    let mut buf = String::new();
    let _ = stdin().read_line(&mut buf);

    // Eğer ses hala çalıyorsa kapat
    if let Some(mut child) = audio_child {
        let _ = child.kill();
    }

    let _ = write!(out, "{CLEAR}{RESET}");
    let _ = out.flush();
    exit(0);
}
