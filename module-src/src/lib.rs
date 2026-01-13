mod foundry;

use foundry::{application, cprintln, *};
use futures::channel::mpsc;
use futures::FutureExt;
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const ID: &str = "johnys-module";

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
                    if item.is_two_handed() {
                        if context.left_hand.is_none() && context.right_hand.is_none() {
                            let item_data: EquipmentItemData = item.into();
                            context.left_hand = Some(item_data.clone());
                            context.right_hand = Some(item_data);
                            context.right_hand_secondary = true;
                        } else {
                            context.extra_held_items.push(item.into());
                        }
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

async fn handle_damage_roll(message: Message) -> Result<(), String> {
    if !is_enabled("popupEnabled") || !is_enabled("globalPopupEnabled") {
        return Ok(());
    }

    let Some(context) = message.pf2e_context() else {
        return Ok(());
    };
    let actor = context.target_actor().await.ctx("target actor")?;
    let gm_strategy = GMStrategy::from_settings(ID);
    if actor.is_owned_by_current_user(gm_strategy) {
        let _ = wait_for_dice_animation(&message).await;
        message.popup().await.ctx("popout")?;
    }

    Ok(())
}

async fn wait_for_dice_animation(message: &Message) -> Result<(), JsValue> {
    if !Game::instance()?.is_module_active("dice-so-nice") {
        return Ok(());
    }
    let message_id = message.id();
    let (tx, mut rx) = mpsc::unbounded();
    let hook_id = hook!("diceSoNiceRollComplete", |dice_message_id: JsValue| {
        if dice_message_id.as_string() == message_id {
            let _ = tx.unbounded_send(());
        }
    });

    let result = futures::select! {
        _ = rx.next() => Ok(()),
        _ = TimeoutFuture::new(20_000).fuse() => Err(JsValue::from_str("gave up")),
    };

    hooks_off("diceSoNiceRollComplete", hook_id);
    result
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
        if let Err(err) = handle_damage_roll(message.into()).await {
            cprintln!("Error in chat message handler: {err}");
        }
    });

    hook!("ready", || {
        if let Ok(game) = Game::instance() {
            if game.is_module_active("pf2e-bestiary-tracking") {
                cprintln!("PF2E Bestiary Tracking detected, registering equipment injection");
                hook!("renderPF2EBestiary", async |app: JsValue, html: JsValue| {
                    if let Err(err) = inject_equipment_ui_async(app.into(), html.into()).await {
                        cprintln!("Error injecting equipment UI: {err:?}");
                    }
                });
            }
        }
    });
}
