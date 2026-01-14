mod foundry;

use foundry::{application, cprintln, *};
use futures::channel::mpsc;
use futures::FutureExt;
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

const ID: &str = "johnys-module";

#[derive(Clone, Debug, Default)]
struct MessageTargetState {
    targets: Vec<String>,
    timestamp: f64,
    waiting_for_dice: bool,
    animation_complete: bool,
}

impl MessageTargetState {
    fn find_new_targets(&self, new_targets: &[String]) -> Vec<String> {
        new_targets
            .iter()
            .filter(|target| !self.targets.contains(target))
            .cloned()
            .collect()
    }

    fn store_targets(&mut self, targets: Vec<String>) {
        self.targets = targets;
        if self.timestamp == 0.0 {
            self.timestamp = now();
        }
    }
}

/// Wrapper around the global message state HashMap with internal mutex
struct MessageStateMap {
    states: Mutex<HashMap<String, MessageTargetState>>,
}

impl MessageStateMap {
    fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Get targets to check, handling dice animation waiting logic
    fn get_targets_to_check(
        &self,
        message_id: &str,
        current_targets: Vec<String>,
        msg_type: &str,
    ) -> Option<Vec<String>> {
        let mut map = self.states.lock().unwrap();
        let state = map
            .entry(message_id.to_string())
            .or_insert_with(MessageTargetState::default);

        // If we were waiting for dice, dice are done now - clear waiting and use current targets
        if state.waiting_for_dice {
            cprintln!("Dice animation complete, using current targets: {current_targets:?}");
            state.waiting_for_dice = false;
            state.store_targets(current_targets.clone());
            return Some(current_targets);
        }

        // Find new targets
        let new_targets = state.find_new_targets(&current_targets);
        cprintln!(
            "new {current_targets:?} old {:?} newly_added {new_targets:?}",
            state.targets
        );

        if new_targets.is_empty() {
            return None;
        }

        // Check if we need to wait for dice animation (only if Dice So Nice is active)
        let dice_so_nice_active = Game::is_module_active("dice-so-nice");
        let should_wait_for_dice = msg_type == "damage-roll" && dice_so_nice_active && !state.animation_complete;
        cprintln!("should wait for dice {should_wait_for_dice} (dsn active: {dice_so_nice_active}) {state:?} {message_id}");

        if should_wait_for_dice {
            // Mark as waiting - diceSoNiceRollComplete will trigger re-processing
            state.waiting_for_dice = true;
            // Store current targets so updateChatMessage doesn't re-trigger
            state.store_targets(current_targets);
            return None;
        }

        // Update stored targets
        state.store_targets(current_targets);
        Some(new_targets)
    }

    /// Clear waiting flag and return whether we were waiting
    fn clear_waiting(&self, message_id: &str) -> bool {
        let mut map = self.states.lock().unwrap();
        if let Some(state) = map.get_mut(message_id) {
            let was_waiting = state.waiting_for_dice;
            state.animation_complete = true;
            was_waiting
        } else {
            false
        }
    }
}

// Global state to track message targets for detecting updates
static MESSAGE_TARGETS: Lazy<MessageStateMap> = Lazy::new(|| MessageStateMap::new());

#[derive(Serialize, Clone)]
struct EquipmentItemData {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    img: Option<String>,
}

impl From<&Item> for EquipmentItemData {
    fn from(item: &Item) -> Self {
        Self {
            name: item.name(),
            img: item.img(),
        }
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct EquipmentContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    left_hand: Option<EquipmentItemData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    right_hand: Option<EquipmentItemData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    armor: Option<EquipmentItemData>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    left_hand_secondary: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    right_hand_secondary: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    extra_held_items: Vec<EquipmentItemData>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    worn_items: Vec<EquipmentItemData>,
}

impl From<&[Item]> for EquipmentContext {
    fn from(items: &[Item]) -> Self {
        let mut context = EquipmentContext::default();
        for item in items.iter().filter(|item| {
            item.is_physical_item()
                && item
                    .carry_type()
                    .as_ref()
                    .map_or(false, |ct| ct == "worn" || ct == "held")
        }) {
            let item_type = item.item_type().unwrap_or_default();
            let carry_type = item.carry_type().unwrap_or_default();

            match (item_type.as_str(), carry_type.as_str()) {
                ("armor", "worn") => {
                    context.armor = Some(item.into());
                }
                ("weapon" | "shield", "held") => {
                    if item.traits().iter().any(|t| t == "free-hand") {
                        context.extra_held_items.push(item.into());
                    } else if item.is_two_handed() {
                        if let Some(item) = context.left_hand.take() {
                            context.extra_held_items.push(item)
                        }
                        if let Some(item) = context.right_hand.take() {
                            context.extra_held_items.push(item)
                        }
                        let item_data: EquipmentItemData = item.into();
                        context.left_hand = Some(item_data.clone());
                        context.right_hand = Some(item_data);
                        context.right_hand_secondary = true;
                    } else {
                        if context.left_hand.is_none() {
                            context.left_hand = Some(item.into());
                        } else if context.right_hand.is_none() {
                            context.right_hand = Some(item.into());
                        } else {
                            context.extra_held_items.push(item.into());
                        }
                    }
                }
                (_, "worn") => {
                    context.worn_items.push(item.into());
                }
                _ => {}
            }
        }

        context
    }
}

pub trait LogResultExt<T> {
    fn ctx(self, msg: &str) -> Result<T, String>;
}

impl<T> LogResultExt<T> for Result<T, JsValue> {
    fn ctx(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{msg}: {e:?}"))
    }
}

fn is_enabled(key: &str) -> bool {
    let value = get_setting(ID, key);
    value.as_bool().unwrap_or(true)
}

/// Get current timestamp in milliseconds
fn now() -> f64 {
    js_sys::Date::now()
}

async fn handle_damage_message(message: Message) -> Result<(), String> {
    if !is_enabled("popupEnabled") || !is_enabled("globalPopupEnabled") {
        return Ok(());
    }
    let msg_type = message.pf2e_type().unwrap_or_default();
    if !matches!(msg_type.as_str(), "damage-roll" | "spell-cast") {
        return Ok(());
    }

    let Some(context) = message.pf2e_context() else {
        return Ok(());
    };
    if msg_type == "spell-cast" && context.options().iter().any(|i| i == "damaging-effect") {
        return Ok(());
    }
    let msg_id = message.id();

    // Get current targets
    let current_targets = message.target_uuids().await;

    // Get targets to check from the state map
    let Some(targets_to_check) =
        MESSAGE_TARGETS.get_targets_to_check(&msg_id, current_targets, &msg_type)
    else {
        return Ok(());
    };

    // Check if any targets belong to current user and show popup
    let gm_strategy = GMStrategy::from_settings(ID);
    for uuid in targets_to_check {
        if let Ok(actor) = Game::from_uuid(&uuid).await {
            if actor.is_owned_by_current_user(gm_strategy) {
                message.popup().await.ctx("popout")?;
                break;
            }
        }
    }

    Ok(())
}

/// Open the equipment screen for the selected actor
/// Can be called from macros with: game.modules.get("johnys-module").api.openEquipmentScreen()
#[wasm_bindgen]
pub async fn open_equipment_screen() {
    if let Err(error_msg) = try_open_equipment_screen()
        .await
        .ctx("Unable to view equipment")
    {
        cprintln!("Error opening equipment screen: {}", error_msg);
        UI::notify_error(&error_msg);
    }
}

async fn try_open_equipment_screen() -> Result<(), JsValue> {
    if !is_enabled("visibleEquipmentEnabled") {
        return Err(JsValue::from_str("Visible equipment must be enabled by GM"));
    }
    let game = Game::instance()?;
    let hovered = game.hovered_token();
    let targeted_tokens = game.user_targets();

    let selected_token = hovered.as_ref().or_else(|| targeted_tokens.first());

    let all_items: EquipmentContext = selected_token
        .ok_or_else(|| JsValue::from_str("Please select or target a token"))?
        .actor_items()
        .as_slice()
        .into();

    let html = application::render_template(
        "modules/johnys-module/templates/equipment-screen.hbs",
        &all_items,
    )
    .await?;

    application::show_dialog("Equipment", html, vec![("close", "Close", None)]).await?;

    Ok(())
}

fn register_settings() {
    SettingConfig::new()
        .name("Enable equipment preview (Global)")
        .hint("Enable or disable equipment icon preview macro and bestiary integration for your players.")
        .scope("world")
        .config(true)
        .type_boolean()
        .default_bool(true)
        .register(ID, "visibleEquipmentEnabled");

    SettingConfig::new()
        .name("Enable Damage Popups (Global)")
        .hint("Enable or disable automatic damage popup windows when your players tokens receive damage.")
        .scope("world")
        .config(true)
        .type_boolean()
        .default_bool(true)
        .register(ID, "globalPopupEnabled");

    SettingConfig::new()
        .name("Enable Damage Popups")
        .hint("Enable or disable automatic damage popup windows when an actor you own receives damage.")
        .scope("client")
        .config(true)
        .type_boolean()
        .default_bool(true)
        .register(ID, "popupEnabled");

    GMStrategy::register_setting(ID);
}

/// render equipment data and inject it into the bestiary window
async fn inject_equipment_ui_async(app: BestiaryApp, html: HtmlElement) -> Result<(), JsValue> {
    let Some(uuid) = app.selected_monster_uuid() else {
        return Ok(());
    };

    if !is_enabled("visibleEquipmentEnabled") {
        return Err(JsValue::from_str("Visible equipment must be enabled by GM"));
    }

    let game = Game::instance()?;
    let all_items: EquipmentContext = game
        .find_token_by_actor_uuid(&uuid)
        .map(|token| token.actor_items().as_slice().into())
        .unwrap_or_default();

    let equipment_html = application::render_template(
        "modules/johnys-module/templates/equipment-screen.hbs",
        &all_items,
    )
    .await?;

    let container_data = html
        .query_selector(".right-monster-container-data")?
        .ok_or_else(|| JsValue::from_str("Could not find right-monster-container-data"))?;

    // Check if equipment section already exists
    if html.query_selector(".equipment-data-section")?.is_none() {
        // Insert as the last section after passives
        container_data.insert_adjacent_html(
            "beforeend",
            &format!(
                r#"
            <div class="data-section primary-container active equipment-data-section">
                <div class="data-header primary-container">
                    <div class="data-header-label">
                        <div class="data-icon primary-icon">
                            <i class="fa-solid fa-shield"></i>
                        </div>
                        <div class="flex-value">Equipment</div>
                    </div>
                </div>
                <div class="data-body primary-border-container">
                    {}
                </div>
            </div>
        "#,
                equipment_html
            ),
        )?;
    }

    Ok(())
}

#[wasm_bindgen]
pub fn main() {
    hook!("init", || {
        register_settings();
        // Register API for macro access
        if let Ok(game) = Game::instance() {
            if let Ok(modules) = game.modules() {
                if let Some(module) = modules.get(ID) {
                    let equipment_fn = Closure::wrap(Box::new(move || {
                        wasm_bindgen_futures::spawn_local(async move {
                            open_equipment_screen().await;
                        });
                    }) as Box<dyn Fn()>);

                    module
                        .set_api_property("openEquipmentScreen", equipment_fn.as_ref())
                        .ok();
                    equipment_fn.forget();
                }
            }
        }
    });

    hook!("createChatMessage", async |message: JsValue| {
        if let Err(err) = handle_damage_message(message.into()).await {
            cprintln!("Error in chat message handler: {err}");
        }
    });

    hook!(
        "updateChatMessage",
        async |message: JsValue, _changes: JsValue, _options: JsValue| {
            if let Err(err) = handle_damage_message(message.into()).await {
                cprintln!("Error in message update handler: {err}");
            }
        }
    );

    hook!(
        "diceSoNiceRollComplete",
        async |dice_message_id: JsValue| {
            if let Some(msg_id) = dice_message_id.as_string() {
                cprintln!("Dice animation complete for message {}", msg_id);

                // Clear waiting flag and check if we should reprocess
                let should_reprocess = MESSAGE_TARGETS.clear_waiting(&msg_id);

                // If we were waiting for dice animation, re-process the message
                cprintln!("Re-processing message {} after dice animation", msg_id);

                if should_reprocess {
                    // Get the message and re-process it
                    if let Ok(game) = Game::instance() {
                        if let Ok(Some(message)) = game.get_message(&msg_id) {
                            if let Err(err) = handle_damage_message(message).await {
                                cprintln!("Error re-processing message after dice: {err}");
                            }
                        }
                    }
                }
            }
        }
    );

    hook!("ready", || {
        if Game::is_module_active("pf2e-bestiary-tracking") {
            cprintln!("PF2E Bestiary Tracking detected, registering equipment injection");
            hook!("renderPF2EBestiary", async |app: JsValue, html: JsValue| {
                if let Err(err) = inject_equipment_ui_async(app.into(), html.into()).await {
                    cprintln!("Error injecting equipment UI: {err:?}");
                }
            });
        }
    });
}
