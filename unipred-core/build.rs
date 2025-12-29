use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["../proto/market_quote.proto"], &["../proto"])?;
    Ok(())
}