mod calculator;
mod conversions;
mod system;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Clone, Debug)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub score: i64,
    pub requires_confirmation: bool,
}

#[derive(Clone, Debug)]
enum Action {
    Copy(String),
    System(system::SystemAction),
}

#[derive(Default)]
pub struct ActionCatalog {
    actions: Mutex<HashMap<String, Action>>,
}

impl ActionCatalog {
    pub fn evaluate(&self, query: &str) -> Vec<ActionItem> {
        let mut candidates = Vec::new();
        if let Ok(value) = calculator::evaluate(query) {
            let formatted = format_number(value);
            candidates.push(self.register(
                format!("{query} = {formatted}"),
                "Calculation · Enter to copy".into(),
                9_700,
                false,
                Action::Copy(formatted),
            ));
        }
        if let Some(conversion) = conversions::convert(query) {
            candidates.push(self.register(
                conversion.display,
                "Offline conversion · Enter to copy".into(),
                9_600,
                false,
                Action::Copy(conversion.value),
            ));
        }
        if let Some(action) = system::SystemAction::parse(query) {
            candidates.push(self.register(
                action.title().into(),
                action.subtitle().into(),
                9_500,
                action.requires_confirmation(),
                Action::System(action),
            ));
        }
        candidates
    }

    pub fn execute(&self, app: &tauri::AppHandle, id: &str, confirmed: bool) -> Result<(), String> {
        let action = self
            .actions
            .lock()
            .map_err(|_| "action catalog is unavailable")?
            .get(id)
            .cloned()
            .ok_or("action expired; search again")?;
        match action {
            Action::Copy(value) => app
                .clipboard()
                .write_text(value)
                .map_err(|error| error.to_string()),
            Action::System(action) => {
                if action.requires_confirmation() && !confirmed {
                    return Err("confirmation required".into());
                }
                action.execute()
            }
        }
    }

    fn register(
        &self,
        title: String,
        subtitle: String,
        score: i64,
        requires_confirmation: bool,
        action: Action,
    ) -> ActionItem {
        let mut hasher = DefaultHasher::new();
        format!("{action:?}").hash(&mut hasher);
        let id = format!("action:{:016x}", hasher.finish());
        if let Ok(mut actions) = self.actions.lock() {
            if actions.len() > 128 {
                actions.clear();
            }
            actions.insert(id.clone(), action);
        }
        ActionItem {
            id,
            title,
            subtitle,
            score,
            requires_confirmation,
        }
    }
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.10}").trim_end_matches('0').to_owned()
    }
}
