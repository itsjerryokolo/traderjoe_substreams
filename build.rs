use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new("LBFactoryV2", "abi/dexcandlesv2_factory.json")?
        .generate()?
        .write_to_file("src/abi/dexcandlesv2_factory.rs")?;

    Abigen::new("LBPairV21", "abi/dexcandlesv2_pair.json")?
        .generate()?
        .write_to_file("src/abi/dexcandlesv2_pair.rs")?;

    Abigen::new("ERC20", "abi/ERC20.json")?
        .generate()?
        .write_to_file("src/abi/erc20.rs")?;

    Ok(())
}
