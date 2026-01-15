use crate::features::is_enabled;
use crate::foundry::error::{ContextExt as _, Error};
use crate::foundry::{cprintln, *};
use crate::{hook, ID};
use futures::lock::Mutex;
use js_sys::Date;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
struct MessageState {
    popped_out: bool,
    timestamp: f64,
    animation_complete: bool,
}

impl Default for MessageState {
    fn default() -> Self {
        Self {
            popped_out: false,
            timestamp: Date::now(),
            animation_complete: false,
        }
    }
}

static MESSAGE_STATE: Lazy<Mutex<HashMap<String, MessageState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl MessageState {
    async fn get(msg_id: &str) -> MessageState {
        let mut map = MESSAGE_STATE.lock().await;
        map.retain(|_, v| v.timestamp > Date::now() - (1000.0 * 60.0 * 5.0));
        map.get(msg_id).cloned().unwrap_or_default()
    }

    async fn update(msg_id: String, updater: impl Fn(&mut MessageState)) {
        let mut map = MESSAGE_STATE.lock().await;
        let state = map.entry(msg_id).or_default();
        updater(state);
    }
}

async fn handle_message(message: Message) -> Result<(), Error> {
    if !is_enabled("popupEnabled") || !is_enabled("globalPopupEnabled") {
        return Ok(());
    }
    let msg_type = message.pf2e_type().unwrap_or_default();
    if !matches!(msg_type.as_str(), "damage-roll" | "spell-cast") {
        return Ok(());
    }

    let damaging_effect = message
        .pf2e_context()
        .ctx("pf2e context")?
        .options()
        .iter()
        .any(|i| i == "damaging-effect");

    if msg_type == "spell-cast" && damaging_effect {
        return Ok(());
    }
    let msg_id = message.id();

    let state = MessageState::get(&msg_id).await;
    let dice_so_nice_active = Game::is_module_active("dice-so-nice");
    let wait_for_animation =
        msg_type == "damage-roll" && dice_so_nice_active && !state.animation_complete;
    if wait_for_animation || state.popped_out {
        return Ok(());
    }
    let gm_strategy = GMStrategy::from_settings(ID);
    let current_targets = message.target_uuids().await;
    for uuid in current_targets {
        if let Ok(actor) = Game::from_uuid(&uuid).await {
            if actor.is_owned_by_current_user(gm_strategy) {
                message.popout().await.ctx("popout")?;
                MessageState::update(msg_id, |state| state.popped_out = true).await;
                break;
            }
        }
    }

    Ok(())
}

pub fn init() {
    hook!("init", || {
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
    });

    hook!("createChatMessage", async |message: JsValue| {
        if let Err(err) = handle_message(message.into()).await {
            cprintln!("Error in chat message handler: {err}");
        }
    });

    hook!(
        "updateChatMessage",
        async |message: JsValue, _changes: JsValue, _options: JsValue| {
            if let Err(err) = handle_message(message.into()).await {
                cprintln!("Error in message update handler: {err}");
            }
        }
    );

    hook!(
        "diceSoNiceRollComplete",
        async |dice_message_id: JsValue| {
            if let Some(msg_id) = dice_message_id.as_string() {
                MessageState::update(msg_id.clone(), |state| state.animation_complete = true).await;
                if let Ok(game) = Game::instance() {
                    if let Ok(Some(message)) = game.get_message(&msg_id) {
                        if let Err(err) = handle_message(message).await {
                            cprintln!("Error re-processing message after dice: {err}");
                        }
                    }
                }
            }
        }
    );
}
