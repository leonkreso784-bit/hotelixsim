//! `cargo run -p ai --bin mjerilo -- S1 [S2]` — mjerilo slaganja AI presuda s Leonovim ocjenama.

#[tokio::main]
async fn main() {
    let codes: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ai::benchmark::run(&codes).await);
}
