use clap::Parser;

/// Book of Hours helper.
/// --principle, --solve, --aspects, and --craft are mutually exclusive.
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

	/// The aspects of the skill you want to upgrade, and its level.
	#[arg(long, short = 'S', num_args = 2)]
	pub solve: Option<Vec<String>>,

	/// Aspects of the item you are looking for.
	#[arg(short, long, num_args = 1..)]
	pub aspects: Option<Vec<String>>,

	/// Either a principle or a skill.
	#[arg(short, long, num_args = 1..)]
	pub craft: Option<String>,
}
