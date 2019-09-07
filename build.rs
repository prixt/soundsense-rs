use std::{
	env, error::Error, fs::File, io::{BufWriter, Write}, path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
	let out_dir = env::var("OUT_DIR")?;
	let dest_dir = Path::new(&out_dir).join("index.html");

	write!(
		BufWriter::new(File::create(&dest_dir)?),
r#"
<!doctype html>
<html>
	<head>
		<style type="text/css">{w3}</style>
		<style type="text/css">{range}</style>
	</head>
	<body>
		<div class="w3-bar w3-border w3-light-grey w3-small">
			<button class='w3-bar-item w3-button'
				onclick="external.invoke('load_gamelog')">Load gamelog.txt</button>
			<button class='w3-bar-item w3-button'
				onclick="external.invoke('load_soundpack')">Load soundpack</button>
			<button class='w3-bar-item w3-button'
				onclick="external.invoke('load_ignore_list')">Load ignore.txt</button>
			<div class='w3-dropdown-hover w3-right'>
				<a ref ='#' class='w3-button'>Options</a>
				<div class='w3-dropdown-content w3-bar-block' style='right:0'>
					<button class='w3-bar-item w3-button w3-disabled'><s>Download Original's Soundpack</s></button>
					<button class='w3-bar-item w3-button w3-disabled'><s>Set current paths as default</s></button>
					<button class='w3-bar-item w3-button w3-disabled'><s>Set current volumes as default</s></button>
					<button class="w3-bar-item w3-button"
						onclick="external.invoke('show_about')">About</button>
				</div>
			</div>
		</div>
		<div class="w3-container">
			<table class="w3-table w3-bordered" id="channels"></table>
		</div>
	</body>
</html>
"#,
		w3=include_str!("src/ui/w3.css"),
		range=include_str!("src/ui/range.css"),
	)?;

	Ok(())
}