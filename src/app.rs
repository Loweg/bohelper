use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
	/// Path to save file
	#[arg(short, long, default_value_t = String::new())]
	pub save_path: String,

	/// Path to game data
	#[arg(short, long)]
	pub data_path: String,

	/// Principle of the memory you are looking for
	#[arg(short, long, default_value_t = String::new())]
	pub principle: String,

	/// The aspects of the skill you want to upgrade, and its level. Does nothing if using --principle.
	#[arg(long, num_args = 2)]
	pub solve: Option<Vec<String>>,

	/// The level of the skill you are upgrading. Does nothing if not using --solve
	#[arg(short, long, default_value_t = 1)]
	pub level: isize,

	/// Aspects of the item you are looking for. Does nothing if using --principle or --solve.
	#[arg(short, long, num_args = 1..)]
	pub aspects: Option<Vec<String>>,
}
