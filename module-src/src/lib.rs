mod foundry;

use foundry::{cprintln, *};
use wasm_bindgen::prelude::*;


const ID: &str = "johnys-module";

pub trait LogResultExt<T> {
    fn ctx(self, msg: &str) -> Result<T, String>;
}

impl<T> LogResultExt<T> for Result<T, JsValue> {
    fn ctx(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{msg}: {e:?}"))
    }
}

fn is_popup_enabled() -> bool {
    let value = get_setting(ID, "popupEnabled");
    value.as_bool().unwrap_or(true)
}

async fn handle_damage_roll(message: Message) -> Result<(), String> {
    if !is_popup_enabled() {
        return Ok(());
    }

    let Some(context) = message.pf2e_context() else {
        return Ok(());
    };
    let actor = context.target_actor().await.ctx("target actor")?;
    let damage = message.first_roll().map(|roll| roll.total()).unwrap_or(0.0);

    //cprintln!("Final damage value: {}", damage);
    if damage <= 0.0 {
        return Ok(());
    }

    let gm_strategy = GMStrategy::from_settings(ID);
    if actor.is_owned_by_current_user(gm_strategy) {
        //cprintln!("Current user owns the damaged actor!");
        message.popup().await.ctx("popout")?;
    }

    Ok(())
}


#[wasm_bindgen]
pub fn main() {
    hook!("init", || {
        SettingConfig::new()
            .name("Enable Damage Popups")
            .hint("Enable or disable automatic damage popup windows when you receive damage")
            .scope("client")
            .config(true)
            .type_boolean()
            .default_bool(true)
            .register(ID, "popupEnabled");

        GMStrategy::register_setting(ID);
    });

    hook!("createChatMessage", async |message: JsValue| {
        if let Err(err) = handle_damage_roll(message.into()).await {
            cprintln!("Error in chat message handler: {err}");
        }
    });
}
