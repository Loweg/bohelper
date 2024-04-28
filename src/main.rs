#![feature(str_from_utf16_endian)]

use std::{
	env::args,
	path::PathBuf,
	sync::{Arc, Mutex},
};

use axum::{routing::get, Router};

mod app;
mod data;
mod logic;
mod save;

use app::*;
use data::init_items;
use save::{default_save_path, SaveData};

#[tokio::main]
async fn main() {
	let mut data_path = PathBuf::from(args().nth(1).expect("Data path required"));
	data_path.extend(["StreamingAssets", "bhcontent", "core"]);
	println!("Using game path: {}", data_path.to_string_lossy());
	let data = init_items(&data_path);

	let path = default_save_path();
	println!("Using save path {}", path.to_string_lossy());
	let mut save = SaveData::from_path(path);
	save.items = save.items.into_iter().filter(|i|
		data.items.contains_key(&i.id) ||
		data.books.contains_key(&i.id) ||
		data.skills.contains_key(&i.id)).collect();

	let state = AppState {
		data: Arc::new(data),
		save: Arc::new(Mutex::new(save)),
	};

	let app = Router::new()
		.route("/", get(root))
		.route("/find_mems", get(p_form).post(find_mems))
		.route("/solve", get(s_form).post(solve))
		.route("/crafting", get(c_form).post(crafting))
		.route("/items", get(i_form).post(items))
		.with_state(state);
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
		.await
		.unwrap();
	axum::serve(listener, app).await.unwrap();


	/*let matching_skills: Vec<_> = data.skills.iter()
		.filter(|(_, s)| s.matches_exact(&aspects))
		.map(|(_, s)| s).collect();

	if matching_skills.is_empty() {
		println!("Warning: no matching skills")
	} else {
		println!("Matching skills:");
		print_skill_stations(&matching_skills, &data.workstations);
	}
	println!();

	for (mem, (source, _)) in find_memories(&[&aspects[0], &aspects[1]], 8, &world_items, &data.items, &data.books) {
		println!("{}:\t {}", source, mem)
	}*/


	//print_aspected(&data.items, &args.aspects.unwrap())


	/*
	let known_recipes: HashSet<_> = data.recipes.0.iter().chain(data.recipes.1.iter()).chain(data.recipes.2.iter())
		.filter(|r| skills.contains(&r.skill)).map(|r| r.label.clone()).collect();

	if data::principles().contains(&craft.as_str()) {

	} else {
		let skill = match data.skills.iter()
			.find(|(_, s)| s.label.to_lowercase().starts_with(&craft.to_lowercase()))
		{
			Some(s) => s,
			None => { println!("Skill not found: {}", craft); return; },
		};
		println!("Using skill {}\n", skill.1.label);

		println!("Prentice recipes:");
		for r in data.recipes.0.into_iter().filter(|r| skill.0 == &r.skill) {
			match known_recipes.get(&r.label) {
				Some(_) => println!("{} ({})", r.label, r.principle),
				None    => println!("{} ({}) [New Recipe!]", r.label, r.principle),
			}
		}

		println!();
		println!("Scholar recipes:");
		for r in data.recipes.1.into_iter().filter(|r| skill.0 == &r.skill) {
			let item = r.ingredient.clone().expect("Scholar recipe uses no ingredient");
			match known_recipes.get(&r.label) {
				Some(_) => println!("{} using {} ({})", r.label, item, r.principle),
				None    => println!("{} using {} ({}) [New Recipe!]", r.label, item, r.principle),
			}
		}

		println!();
		println!("Keeper recipes:");
		for r in data.recipes.2.into_iter().filter(|r| skill.0 == &r.skill) {
			let item = data.items.get(&r.ingredient.clone().expect("Keeper recipe uses no ingredient")).expect("Couldn't find ingredient");
			match known_recipes.get(&r.label) {
				Some(_) => println!("{} using {} ({})", r.label, item.label, r.principle),
				None    => println!("{} using {} ({}) [New Recipe!]", r.label, item.label, r.principle),
			}
		}
	}*/
}
