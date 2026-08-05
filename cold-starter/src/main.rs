pub mod catchup;

fn main() {
    println!("Cold Starter initialized");
    let routines = catchup::CatchupRoutines;
    routines.fetch_200_ema();
    routines.replay_buffer_in_paper_mode();
    routines.transition_to_live();
}
