use tracing_subscriber::fmt::format::FmtSpan;

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
}