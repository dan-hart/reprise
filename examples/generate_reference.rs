use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/cli-reference.md");
    let generated = reprise::cli::reference::markdown();
    match std::env::args().nth(1).as_deref() {
        None => std::fs::write(&path, generated)?,
        Some("--check") => {
            if std::fs::read_to_string(&path)? != generated {
                return Err(
                    "Command reference is stale; run cargo run --example generate_reference".into(),
                );
            }
        }
        Some(_) => return Err("Usage: cargo run --example generate_reference -- [--check]".into()),
    }
    Ok(())
}
