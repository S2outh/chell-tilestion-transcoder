use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Parser)]
pub struct Cli {
    pub rustdoc_json_path: String,
    pub metadata_config: String,
    pub def_paths: Vec<String>,
    #[arg(short, long, default_value = "latest")]
    pub version: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let rustdoc_json = fs::read_to_string(cli.rustdoc_json_path)?;
    let metadata_config = fs::read_to_string(cli.metadata_config)?;

    let metadata = ron::from_str(&metadata_config)?;

    let out =
        chell_tilestion_transcoder::generate_tilestion_tm(cli.version, metadata, rustdoc_json, cli.def_paths)?;

    println!("{}", out);

    Ok(())
}
