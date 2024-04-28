use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::Form;
use serde::Deserialize;
use std::sync::Mutex;

use crate::data::Data;
use crate::logic::{find_aspected, find_memories, get_skill_stations};
use crate::save::SaveData;

#[derive(Clone)]
pub struct AppState {
	pub data: Arc<Data>,
	pub save: Arc<Mutex<SaveData>>,
}

pub async fn root() -> Html<&'static str> {
	Html(
		r#"
		<!doctype html>
		<html>
		<head></head>
		<body>
			<a href=/find_mems>Find Memories</a>
			<a href=/solve>Solver</a>
			<a href=/crafting>Crafting</a>
			<a href=/items>Items Browser</a>
		</body>
		</html>
		"#,
	)
}

pub async fn p_form() ->  Html<&'static str> {
	Html(
		r#"
		<!doctype html>
		<html>
		<head></head>
		<body>
			<form action="/find_mems" method="post">
				<label for="principle">
					<input type="text" name="principle">
				</label>
			<input type="submit" value="Go">
			</form>
		</body>
		</html>
		"#,
	)
}

#[derive(Deserialize, Debug)]
pub struct PInput {
	principle: String,
}

pub async fn find_mems(
	State(state): State<AppState>,
	Form(input): Form<PInput>,
) -> impl IntoResponse {
	let mems = find_memories(&[&input.principle], 8, &state.save.lock().unwrap().items, &state.data.items, &state.data.books);
	drop(state.save);
	let mut res = String::new();
	for mem in mems {
		let val = mem.1.1.get(&input.principle).unwrap();
		res.push_str(&format!("{} has memory {} with {}: {}\n", mem.0, mem.1.0, &input.principle, val));
	}
	res
}

pub async fn s_form() ->  Html<&'static str> {
	Html(
		r#"
		<!doctype html>
		<html>
		<head></head>
		<body>
			<form action="/solve" method="post">
				<label for="p1">
					<input type="text" name="p1">
				</label>
				<label for="p2">
					<input type="text" name="p2">
				</label>
			<input type="submit" value="Go">
			</form>
		</body>
		</html>
		"#,
	)
}

#[derive(Deserialize, Debug)]
pub struct SInput {
	p1: String,
	p2: String,
}

pub async fn solve(
	State(state): State<AppState>,
	Form(input): Form<SInput>,
) -> String {
	let mut res = String::new();
	let matching_skills: Vec<_> = state.data.skills.iter()
		.filter(|(_, s)| s.matches_exact(&[input.p1.clone(), input.p2.clone()]))
		.map(|(_, s)| s).collect();

	if matching_skills.is_empty() {
		res.push_str("Warning: no matching skills\n");
	} else {
		res.push_str("Matching skills:\n");
		res.push_str(&get_skill_stations(&matching_skills, &state.data.workstations));
	}

	let mems = find_memories(
		&[&input.p1, &input.p2],
		8,
		&state.save.lock().unwrap().items,
		&state.data.items,
		&state.data.books
	);
	drop(state.save);

	for (mem, (source, _)) in mems {
		res.push_str(&format!("{}:\t {}\n", source, mem));
	}

	res
}

pub async fn c_form() ->  Html<&'static str> {
	Html(
		r#"
		<!doctype html>
		<html>
		<head></head>
		<body>
			<form action="/crafting" method="post">
				<label for="skill">
					<input type="text" name="skill">
				</label>
			<input type="submit" value="Go">
			</form>
		</body>
		</html>
		"#,
	)
}

#[derive(Deserialize, Debug)]
pub struct CInput {
	skill: String,
}

pub async fn crafting(
	State(state): State<AppState>,
	Form(input): Form<CInput>,
) -> String {
	let save = state.save.lock().unwrap();
	let known_recipes: HashSet<_> = state.data.recipes.0.iter()
		.chain(state.data.recipes.1.iter()).chain(state.data.recipes.2.iter())
		.filter(|r| save.skills.contains(&r.skill)).map(|r| r.label.clone()).collect();
	drop(save);

	let skill = match state.data.skills.iter().find(|(_, s)| s.label.to_lowercase().starts_with(&input.skill.to_lowercase())) {
		Some(s) => s,
		None => { return format!("Skill not found: {}", input.skill); },
	};

	let mut res = format!("Using skill {}\n", skill.1.label).to_owned();
	res.push_str("Prentice recipes:\n");

	for r in state.data.recipes.0.iter().filter(|r| skill.0 == &r.skill) {
		let s = match known_recipes.get(&r.label) {
			Some(_) => format!("{} ({})\n", r.label, r.principle),
			None    => format!("{} ({}) [New Recipe!]\n", r.label, r.principle),
		};
		res.push_str(&s);
	}

	res.push_str("\nScholar recipes:\n");
	for r in state.data.recipes.1.iter().filter(|r| skill.0 == &r.skill) {
		let item = r.ingredient.clone().expect("Scholar recipe uses no ingredient");
		let s = match known_recipes.get(&r.label) {
			Some(_) => format!("{} using {} ({})\n", r.label, item, r.principle),
			None    => format!("{} using {} ({}) [New Recipe!]\n", r.label, item, r.principle),
		};
		res.push_str(&s);
	}

	res.push_str("\nKeeper recipes:\n");
	for r in state.data.recipes.2.iter().filter(|r| skill.0 == &r.skill) {
		let item = state.data.items.get(&r.ingredient.clone().expect("Keeper recipe uses no ingredient")).expect("Couldn't find ingredient");
		let s = match known_recipes.get(&r.label) {
			Some(_) => format!("{} using {} ({})\n", r.label, item.label, r.principle),
			None    => format!("{} using {} ({}) [New Recipe!]\n", r.label, item.label, r.principle),
		};
		res.push_str(&s);
	}
	res
}

pub async fn i_form() ->  Html<&'static str> {
	Html(
		r#"
		<!doctype html>
		<html>
		<head></head>
		<body>
			<form action="/items" method="post">
				<label for="principles">
					<input type="text" name="principles">
				</label>
			<input type="submit" value="Go">
			</form>
		</body>
		</html>
		"#,
	)
}

#[derive(Deserialize, Debug)]
pub struct IInput {
	principles: String,
}

pub async fn items(
	State(state): State<AppState>,
	Form(input): Form<IInput>,
) -> impl IntoResponse {
	let aspects = input.principles.split(",").collect();
	let found = find_aspected(&state.data.items, &[aspects]);

	let mut res = String::new();
	for (label, aspects) in found {
		res.push_str(&label);
		res.push('\n');
		for (aspect, intensity) in aspects {
			if !aspect.starts_with("boost") {
				res.push_str(&format!("{aspect}: {intensity}  	\n"));
			}
		}
	}
	res
}
