//! Cycle Finance açılış ekranı — bağımsız binary.
//!
//! `cargo run -p cycle-splash` veya `target/release/cycle-splash` ile
//! tek terminalde çalışır; FIGlet ASCII animasyonu + 3sn'lik yükleme çubuğu
//! bittikten sonra Enter bekler, basılınca çıkar.
//! tmux başlatıcısı bunu 4'lü ekran açılmadan önce çağırır.

fn main() {
    cycle_splash::show_splash();
}
