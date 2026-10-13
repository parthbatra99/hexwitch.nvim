use serde_json::{json, Value};

const OMARCHY: &str = include_str!("../../../../playground/themes/omarchy.json");
const EVAL_INK: &str = include_str!("../../../eval/states/dark.json");
const EVAL_PAPER: &str = include_str!("../../../eval/states/light.json");

pub struct Catalog {
    omarchy: Value,
}

fn built_in() -> Value {
    json!([
        {"id":"dark","name":"Eval ink","mode":"dark","dots":["#151c25","#bca2ec","#73c796","#e4bd76"]},
        {"id":"light","name":"Eval paper","mode":"light","dots":["#f3eee5","#7054a0","#35734e","#916125"]},
    ])
}

impl Catalog {
    pub fn load() -> Result<Self, String> {
        let omarchy: Value = serde_json::from_str(OMARCHY)
            .map_err(|e| format!("bundled Omarchy themes are invalid: {e}"))?;
        if omarchy["version"] != 1 || !omarchy["themes"].is_array() {
            return Err("bundled Omarchy themes have an unsupported shape".into());
        }
        Ok(Self { omarchy })
    }

    fn omarchy_themes(&self) -> &[Value] {
        self.omarchy["themes"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn public(&self) -> Value {
        let source = &self.omarchy["source"];
        let omarchy = self
            .omarchy_themes()
            .iter()
            .map(|theme| json!({"id":theme["id"],"name":theme["name"],"mode":theme["mode"],"dots":theme["dots"]}))
            .collect::<Vec<_>>();
        json!({"groups":[
            {"name":"Built in","themes":built_in()},
            {"name":"Omarchy","origin":source["origin"],"revision":source["revision"],"license":source["license"],"themes":omarchy},
        ]})
    }

    pub fn source(&self, id: &str) -> Result<String, String> {
        match id {
            "dark" => Ok(EVAL_INK.into()),
            "light" => Ok(EVAL_PAPER.into()),
            _ => self
                .omarchy_themes()
                .iter()
                .find(|theme| theme["id"] == id)
                .map(|theme| theme["theme"].to_string())
                .ok_or_else(|| "unknown bundled reference".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_theme_imports() {
        let catalog = Catalog::load().unwrap();
        let groups = catalog.public()["groups"].clone();
        let ids = groups
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["themes"].as_array().unwrap().clone())
            .map(|theme| theme["id"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert!(ids.len() >= 25);
        for id in ids {
            let theme = hexwitch_core::ingest::load_theme(&catalog.source(&id).unwrap())
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            theme.validate().unwrap_or_else(|e| panic!("{id}: {e}"));
        }
        assert!(catalog.source("nope").is_err());
    }
}
