use maud::{html, Markup};

pub fn base_layout(title: &str, content: Markup) -> Markup {
	html! {
		(maud::DOCTYPE)
		html {
			head {
				meta charset="utf-8";
				meta name="application-name" content="BoHelper";
				meta name="color-scheme" content="dark";
				meta name="viewport" content="width=device-width, initial-scale=1.0";
				link rel="stylesheet" href="/assets/main.css";
				title { (title) }
			}
			body {
				header {
					a .hbutton href = "/" {"Main"}
					a .hbutton href = "/find_mems" {"Find Memories"}
					a .hbutton href = "/solve" {"Solver"}
					a .hbutton href = "/crafting" {"Crafting"}
					a .hbutton href = "/items" {"Items Browser"}
				}
				(content)
			}
		}
	}
}
