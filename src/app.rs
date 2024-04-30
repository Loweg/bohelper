use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Form;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::sync::Mutex;

use crate::data::{Data, Recipe};
use crate::logic::{find_aspected, find_memories, get_skill_stations};
use crate::save::SaveData;
use crate::ui::base_layout;

#[derive(Clone)]
pub struct AppState {
	pub data: Arc<Data>,
	pub save: Arc<Mutex<SaveData>>,
}

pub async fn root() -> Markup {
	base_layout("BoHelper", html! {})
}

pub async fn p_form() -> Markup {
	base_layout("BoH Memories", html! {
		form method="post" action="/find_mems" {
			input .textbox type="text" name="principle" id="principle" placeholder="Principle";
			input type="submit" value="Go";
		}
	})
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
		res.push_str(&format!("{} has memory {} with {}: {}\n", mem.1.0, mem.0, &input.principle, val));
	}
	base_layout("BoH Memories", response_to_html(res))
}

pub async fn s_form() -> Markup {
	base_layout("BoH Solver", html! {
		form method="post" action="/solve" {
			input .textbox type="text" name="p1" id="p1" placeholder="Principle";
			input .textbox type="text" name="p2" id="p2" placeholder="Principle";
			input type="submit" value="Go";
		}
	})
}

#[derive(Deserialize, Debug)]
pub struct SInput {
	p1: String,
	p2: String,
}

pub async fn solve(
	State(state): State<AppState>,
	Form(input): Form<SInput>,
) -> Markup {
	let mut res = String::new();
	let matching_skills: Vec<_> = state.data.skills.iter()
		.filter(|(_, s)| s.matches(&[input.p1.clone(), input.p2.clone()]))
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
	base_layout("BoH Solver", response_to_html(res))
}

pub async fn c_form() -> Markup {
	base_layout("BoH Crafting", html! {
		form method="post" action="/crafting" {
			input .textbox type="text" name="skill" id="skill" placeholder="Skill";
			input type="submit" value="Go";
		}
	})
}

#[derive(Deserialize, Debug)]
pub struct CInput {
	skill: String,
}

pub async fn crafting(
	State(state): State<AppState>,
	Form(input): Form<CInput>,
) -> Markup {
	/*for recipe in state.data.recipes.0.iter()
		.chain(state.data.recipes.1.iter())
		.chain(state.data.recipes.2.iter())
	{
		if !state.data.workstations.iter().any(|w| w.can_craft(recipe, &state.data)) {
			println!("Uncraftable: {:?}", recipe)
		}
		}*/

	let save = state.save.lock().expect("Lock poison error");
	let known_recipes: HashSet<_> = state.data.recipes.0.iter()
		.chain(state.data.recipes.1.iter())
		.chain(state.data.recipes.2.iter())
		.filter(|r| state.data.workstations.iter().any(|w| w.can_craft(r, &state.data)))
		.filter(|r| save.skills.contains(&r.skill))
		.map(|r| r.label.clone()).collect();
	drop(save);

	let skill = match state.data.skills.iter()
		.find(|(_, s)| s.label.to_lowercase().starts_with(&input.skill.to_lowercase()))
	{
		Some(s) => s,
		None => { return base_layout("BoH Crafting", html!{ (format!("Skill not found: {}", input.skill))}); },
	};


	let mut res = format!("Using skill {}\n", skill.1.label).to_owned();
	let mut collate_recipes = |recipes: &Vec<Recipe>, level: RecipeLevel| {
		res.push_str(&format!("\n{} recipes:\n", level));
		let recipes: Vec<_> = recipes.iter().filter(|r|
			skill.0 == &r.skill &&
			state.data.workstations.iter().any(|w| w.can_craft(r, &state.data))).collect();
		let len = recipes.len();
		for r in recipes {
			let item = match level {
				RecipeLevel::Prentice => String::new(),
				RecipeLevel::Scholar => String::from(" using ") + &r.ingredient.as_ref().unwrap(),
				RecipeLevel::Keeper => {
					if let Some(item) = state.data.items.get(r.ingredient.as_ref().unwrap()) {
						String::from(" using ") + &item.label
					} else {
						String::from(" using ") + &r.ingredient.as_ref().unwrap()
					}
				},
			};
			res.push_str(&match known_recipes.get(&r.label) {
				Some(_) => format!("{}{} ({})\n", r.label, item, r.principle),
				None    => format!("{}{} ({}) [New Recipe!]\n", r.label, item, r.principle),
			});
			match len {
				0 => res.push_str("[2 uncraftable recipes not shown]\n"),
				1 => res.push_str("[1 uncraftable recipe not shown]\n"),
				_ => {},
			}
		}
	};

	collate_recipes(&state.data.recipes.0, RecipeLevel::Prentice);
	collate_recipes(&state.data.recipes.1, RecipeLevel::Scholar);
	collate_recipes(&state.data.recipes.2, RecipeLevel::Keeper);

	base_layout("BoH Recipes", response_to_html(res))
}

#[derive(Debug, PartialEq, Eq)]
enum RecipeLevel {
	Prentice,
	Keeper,
	Scholar,
}
impl fmt::Display for RecipeLevel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			RecipeLevel::Prentice => f.write_str("Prentice"),
			RecipeLevel::Keeper => f.write_str("Keeper"),
			RecipeLevel::Scholar => f.write_str("Scholar"),
		}
	}
}

pub async fn i_form() -> Markup {
	base_layout("BoH Item Browser", html! {
		form method="post" action="/items" {
			input .textbox type="text" name="principles" id="principles" placeholder="lantern,tool";
			input type="submit" value="Go";
		}
	})
}

#[derive(Deserialize, Debug)]
pub struct IInput {
	principles: String,
}

pub async fn items(
	State(state): State<AppState>,
	Form(input): Form<IInput>,
) -> Markup {
	let aspects: Vec<_> = input.principles.split(",").collect();
	let found = find_aspected(&state.data.items, aspects.as_slice());

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
	base_layout("BoH Item Browser", response_to_html(res))
}

fn response_to_html(res: String) -> Markup {
	let mut html = String::new();
	for l in res.lines() {
		html.push_str(&format!("<p>{l}</p>"))
	}
	html! {(PreEscaped(html))}
}
