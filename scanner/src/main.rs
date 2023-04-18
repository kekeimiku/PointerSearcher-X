use scanner::cmd::Commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    argh::from_env::<Commands>().init().unwrap();
    Ok(())
}
