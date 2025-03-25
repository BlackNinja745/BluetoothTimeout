use tracing_subscriber::fmt::format::FmtSpan;
use windows_bluetooth::discover_devices;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    println!("{:#?}", discover_devices()?);
    
    Ok(())
}