pub mod catchup;

#[tokio::main]
async fn main() {
    println!("Cold Starter initialized");
    let routines = catchup::CatchupRoutines;
    match routines.fetch_200_ema().await {
        Ok(ema) => println!("200 EMA: {ema:.4}"),
        Err(e) => {
            eprintln!("ColdStarter: 200 EMA alınamadı: {e}");
            std::process::exit(1);
        }
    }
    routines.transition_to_live();
}
