use std::{
	env, error::Error, fs::File, io::Write, path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
	let dest_dir = Path::new(
			&env::var("OUT_DIR")?
		)
		.join("index.html");
	
	let index_html = include_str!("web/index.html")
		.replace("{comment_start}"	, "<!--")
		.replace("{comment_end}"	, "-->")
		.replace("{range}"			, include_str!("web/range.css"))
		.replace("{w3}"				, include_str!("web/w3.css"))
		.replace("    "				, "") // Remove four-space tabs
		.replace("  "				, "") // Remove two-space tabs
		.replace(|c: char| c.is_whitespace() && c != ' ', "") // Remove all non-space unneccesary whitespaces
		.replace("{js}"				, include_str!("web/script.js"));

	File::create(dest_dir)?
		.write_all( index_html.as_bytes() )?;

	Ok(())
}