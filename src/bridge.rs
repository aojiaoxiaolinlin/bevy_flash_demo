//! The browser sends small commands; Rust owns asset and playback state.
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::VecDeque};

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Page {
    #[default]
    Animation,
    Skins,
    Ui,
    Buttons,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Navigate { page: Page, index: usize },
    Retry,
    Play { name: String, mode: String },
    Fallback { name: String },
    Pause,
    Speed { value: f64 },
    Seek { frame: usize },
    Skin { slot: String, variant: String },
    Reset,
    Disabled { value: bool },
    Diagnostics { value: bool },
}

#[derive(Default, Serialize)]
pub struct Status {
    pub page: Page,
    pub index: usize,
    pub ready: bool,
    pub error: Option<String>,
    pub notice: String,
    pub assets: Vec<String>,
    pub clips: Vec<String>,
    pub skins: Vec<(String, Vec<String>, String)>,
    pub fallback: String,
    pub clip: String,
    pub frame: usize,
    pub frames: usize,
    pub playing: bool,
    pub speed: f64,
    pub terminal: bool,
    pub events: Vec<String>,
    pub clicks: u32,
    pub fps: Option<f32>,
}

thread_local! {
    static COMMANDS: RefCell<VecDeque<Action>> = const { RefCell::new(VecDeque::new()) };
    static STATUS: RefCell<String> = RefCell::new("{}".into());
}

pub fn drain() -> Vec<Action> {
    COMMANDS.with(|queue| queue.borrow_mut().drain(..).collect())
}
pub fn push(action: Action) {
    COMMANDS.with(|queue| queue.borrow_mut().push_back(action));
}
pub fn publish(status: &Status) {
    if let Ok(json) = serde_json::to_string(status) {
        STATUS.with(|value| *value.borrow_mut() = json);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn send_command(json: &str) -> Result<(), wasm_bindgen::JsValue> {
    let action = serde_json::from_str(json)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    push(action);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn demo_status() -> String {
    STATUS.with(|value| value.borrow().clone())
}
