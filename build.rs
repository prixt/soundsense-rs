use std::{
	env, error::Error, fs::File, io::{BufWriter, Write}, path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
	let dest_dir = Path::new(
			&env::var("OUT_DIR")?
		)
		.join("index.html");

	write!(
		BufWriter::new(File::create(&dest_dir)?),
		include_str!("web/index.html"),
		w3=include_str!("web/w3.css"),
		range=include_str!("web/range.css"),
		js=include_str!("web/script.js"),
		comment_start="<!--", comment_end="-->",
	)?;

	Ok(())
}