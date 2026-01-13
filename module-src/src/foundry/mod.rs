// Foundry VTT API bindings for Rust/WASM
//
// This module provides a Rust-native interface to Foundry VTT's JavaScript API.
// It wraps the JavaScript objects in strongly-typed Rust structs.
#![allow(dead_code)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[path = "macros.rs"]
#[macro_use]
mod macros;

pub(crate) use macros::cprintln;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = Hooks, js_name = on)]
    pub fn hooks_on(hook: &str, r#fn: &Closure<dyn Fn()>) -> i32;

    #[wasm_bindgen(js_namespace = Hooks, js_name = on)]
    pub fn hooks_on_1(hook: &str, r#fn: &Closure<dyn Fn(JsValue)>) -> i32;

    #[wasm_bindgen(js_namespace = Hooks, js_name = on)]
    pub fn hooks_on_2(hook: &str, r#fn: &Closure<dyn Fn(JsValue, JsValue)>) -> i32;

    #[wasm_bindgen(js_namespace = Hooks, js_name = once)]
    pub fn hooks_once_1(hook: &str, r#fn: &Closure<dyn Fn(JsValue)>) -> i32;

    #[wasm_bindgen(js_namespace = Hooks, js_name = off)]
    pub fn hooks_off(hook: &str, hook_id: i32);

    // fromUuid global function
    #[wasm_bindgen(catch, js_name = fromUuid)]
    pub async fn from_uuid_raw(uuid: &str) -> Result<JsValue, JsValue>;

    // Game settings API
    #[wasm_bindgen(js_namespace = ["game", "settings"], js_name = register)]
    fn register_setting_raw(module: &str, key: &str, data: &JsValue);

    #[wasm_bindgen(js_namespace = ["game", "settings"], js_name = get)]
    pub fn get_setting(module: &str, key: &str) -> JsValue;
}

pub fn get_property(obj: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    js_sys::Reflect::get(obj, jstr!(key))
}

pub fn get_string_property(obj: &JsValue, key: &str) -> Option<String> {
    get_property(obj, key).ok()?.as_string()
}

fn get_f64_property(obj: &JsValue, key: &str) -> Option<f64> {
    get_property(obj, key).ok()?.as_f64()
}

/// Builder for creating Foundry VTT settings
pub struct SettingConfig {
    config: js_sys::Object,
}

impl SettingConfig {
    pub fn new() -> Self {
        Self {
            config: js_sys::Object::new(),
        }
    }

    pub fn name(self, name: &str) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("name"), jstr!(name)).unwrap();
        self
    }

    pub fn hint(self, hint: &str) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("hint"), jstr!(hint)).unwrap();
        self
    }

    pub fn scope(self, scope: &str) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("scope"), jstr!(scope)).unwrap();
        self
    }

    pub fn config(self, config: bool) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("config"), &JsValue::from(config)).unwrap();
        self
    }

    pub fn type_string(self) -> Self {
        let global = js_sys::global();
        let string_constructor = js_sys::Reflect::get(&global, jstr!("String")).unwrap();
        js_sys::Reflect::set(&self.config, jstr!("type"), &string_constructor).unwrap();
        self
    }

    pub fn type_boolean(self) -> Self {
        let global = js_sys::global();
        let boolean_constructor = js_sys::Reflect::get(&global, jstr!("Boolean")).unwrap();
        js_sys::Reflect::set(&self.config, jstr!("type"), &boolean_constructor).unwrap();
        self
    }

    pub fn type_number(self) -> Self {
        let global = js_sys::global();
        let number_constructor = js_sys::Reflect::get(&global, jstr!("Number")).unwrap();
        js_sys::Reflect::set(&self.config, jstr!("type"), &number_constructor).unwrap();
        self
    }

    pub fn default_string(self, default: &str) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("default"), jstr!(default)).unwrap();
        self
    }

    pub fn default_bool(self, default: bool) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("default"), &JsValue::from(default)).unwrap();
        self
    }

    pub fn default_number(self, default: f64) -> Self {
        js_sys::Reflect::set(&self.config, jstr!("default"), &JsValue::from(default)).unwrap();
        self
    }

    pub fn choices(self, choices: &[(&str, &str)]) -> Self {
        let choices_obj = js_sys::Object::new();
        for (key, value) in choices {
            js_sys::Reflect::set(&choices_obj, jstr!(key), jstr!(value)).unwrap();
        }
        js_sys::Reflect::set(&self.config, jstr!("choices"), &choices_obj).unwrap();
        self
    }

    pub fn register(self, module_id: &str, key: &str) {
        register_setting_raw(module_id, key, &self.config);
    }
}

pub struct Game {
    inner: JsValue,
}

impl Game {
    pub fn instance() -> Result<Self, JsValue> {
        let inner = js_sys::Reflect::get(&js_sys::global(), jstr!("game"))?;
        Ok(Game { inner })
    }

    /// Get the underlying JsValue
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

pub struct UI {
    inner: JsValue,
}

impl UI {
    pub fn instance() -> Result<Self, JsValue> {
        let inner = js_sys::Reflect::get(&js_sys::global(), jstr!("ui"))?;
        Ok(Self { inner })
    }

    /// Convenience method to show an error notification
    pub fn notify_error(message: &str) {
        if let Ok(ui) = Self::instance() {
            ui.notifications_error(message);
        }
    }

    /// Convenience method to show a warning notification
    pub fn notify_warn(message: &str) {
        if let Ok(ui) = Self::instance() {
            ui.notifications_warn(message);
        }
    }

    /// Convenience method to show an info notification
    pub fn notify_info(message: &str) {
        if let Ok(ui) = Self::instance() {
            ui.notifications_info(message);
        }
    }

    pub fn notifications_error(&self, message: &str) {
        if let Ok(notifications) = get_property(&self.inner, "notifications") {
            if let Ok(error_fn) = get_property(&notifications, "error") {
                let args = js_sys::Array::new();
                args.push(jstr!(message));
                let _ = js_sys::Reflect::apply(error_fn.unchecked_ref(), &notifications, &args);
            }
        }
    }

    pub fn notifications_warn(&self, message: &str) {
        if let Ok(notifications) = get_property(&self.inner, "notifications") {
            if let Ok(warn_fn) = get_property(&notifications, "warn") {
                let args = js_sys::Array::new();
                args.push(jstr!(message));
                let _ = js_sys::Reflect::apply(warn_fn.unchecked_ref(), &notifications, &args);
            }
        }
    }

    pub fn notifications_info(&self, message: &str) {
        if let Ok(notifications) = get_property(&self.inner, "notifications") {
            if let Ok(info_fn) = get_property(&notifications, "info") {
                let args = js_sys::Array::new();
                args.push(jstr!(message));
                let _ = js_sys::Reflect::apply(info_fn.unchecked_ref(), &notifications, &args);
            }
        }
    }
}

impl Game {
    /// Get the current user
    pub fn user(&self) -> Result<User, JsValue> {
        let inner = get_property(&self.inner, "user")?;
        Ok(inner.into())
    }

    /// Get all users in the game
    pub fn users(&self) -> Result<UserCollection, JsValue> {
        let inner = get_property(&self.inner, "users")?;
        Ok(UserCollection { inner })
    }

    /// Get a message by ID
    pub fn get_message(&self, id: &str) -> Result<Option<Message>, JsValue> {
        let messages = get_property(&self.inner, "messages")?;
        let get_fn = get_property(&messages, "get")?;
        let args = js_sys::Array::new();
        args.push(jstr!(id));

        let result = js_sys::Reflect::apply(get_fn.unchecked_ref(), &messages, &args)?;

        if result.is_null() || result.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(result.into()))
        }
    }

    /// Resolve a UUID to an actor
    pub async fn from_uuid(uuid: &str) -> Result<Actor, JsValue> {
        let inner = from_uuid_raw(uuid).await?;
        Ok(inner.into())
    }

    /// Get the currently controlled tokens
    pub fn controlled_tokens(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        if let Ok(canvas) = get_property(&self.inner, "canvas") {
            if let Ok(tokens_layer) = get_property(&canvas, "tokens") {
                if let Ok(controlled) = get_property(&tokens_layer, "controlled") {
                    if let Ok(Some(iter)) = js_sys::try_iter(&controlled) {
                        for item in iter {
                            if let Ok(inner) = item {
                                tokens.push(inner.into());
                            }
                        }
                    }
                }
            }
        }

        tokens
    }

    /// Get the current user's targeted tokens
    pub fn user_targets(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        if let Ok(user) = self.user() {
            if let Ok(targets) = get_property(user.as_js_value(), "targets") {
                if let Ok(Some(iter)) = js_sys::try_iter(&targets) {
                    for item in iter {
                        if let Ok(inner) = item {
                            tokens.push(inner.into());
                        }
                    }
                }
            }
        }

        tokens
    }

    /// Get the token currently being hovered over
    pub fn hovered_token(&self) -> Option<Token> {
        if let Ok(canvas) = get_property(&self.inner, "canvas") {
            if let Ok(tokens_layer) = get_property(&canvas, "tokens") {
                if let Ok(hover) = get_property(&tokens_layer, "hover") {
                    if !hover.is_null() && !hover.is_undefined() {
                        return Some(hover.into());
                    }
                }
            }
        }
        None
    }

    /// Check if a module is installed and active
    pub fn is_module_active(&self, module_id: &str) -> bool {
        if let Ok(modules) = self.modules() {
            if let Some(module) = modules.get(module_id) {
                return module.is_active();
            }
        }
        false
    }

    /// Get the module collection
    pub fn modules(&self) -> Result<ModuleCollection, JsValue> {
        let inner = get_property(&self.inner, "modules")?;
        Ok(ModuleCollection { inner })
    }

    /// Find a token on the current scene by actor UUID
    /// Searches through all tokens on the canvas to find one whose actor matches the given UUID
    pub fn find_token_by_actor_uuid(&self, uuid: &str) -> Option<Token> {
        // Extract just the actor ID from the UUID (format is "Actor.ID")
        let actor_id = if let Some(idx) = uuid.rfind('.') {
            &uuid[idx + 1..]
        } else {
            uuid
        };

        let canvas = get_property(&self.inner, "canvas").ok()?;
        let tokens_layer = get_property(&canvas, "tokens").ok()?;
        let placeables = get_property(&tokens_layer, "placeables").ok()?;

        if let Ok(Some(iter)) = js_sys::try_iter(&placeables) {
            for token_result in iter {
                if let Ok(token_js) = token_result {
                    if let Ok(token_actor) = get_property(&token_js, "actor") {
                        let token_actor_id = get_string_property(&token_actor, "id");
                        let token_actor_uuid = get_string_property(&token_actor, "uuid");

                        // Try matching by ID first, then by full UUID
                        if let Some(id) = &token_actor_id {
                            if id == actor_id {
                                return Some(Token::from(token_js));
                            }
                        }

                        if let Some(token_uuid) = &token_actor_uuid {
                            if token_uuid == uuid {
                                return Some(Token::from(token_js));
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

/// Collection of Foundry VTT modules
pub struct ModuleCollection {
    inner: JsValue,
}

impl ModuleCollection {
    /// Get a module by ID
    pub fn get(&self, module_id: &str) -> Option<Module> {
        let get_fn = get_property(&self.inner, "get").ok()?;
        let args = js_sys::Array::new();
        args.push(jstr!(module_id));

        let inner = js_sys::Reflect::apply(get_fn.unchecked_ref(), &self.inner, &args).ok()?;

        if inner.is_null() || inner.is_undefined() {
            None
        } else {
            Some(Module { inner })
        }
    }
}

/// Represents a Foundry VTT module
pub struct Module {
    inner: JsValue,
}

impl Module {
    /// Check if the module is active
    pub fn is_active(&self) -> bool {
        get_property(&self.inner, "active")
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get the module's ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "id")
    }

    /// Get the module's title
    pub fn title(&self) -> Option<String> {
        get_string_property(&self.inner, "title")
    }

    /// Set a property on the module's API object
    /// This creates the `api` object if it doesn't exist
    pub fn set_api_property(&self, key: &str, value: &JsValue) -> Result<(), JsValue> {
        // Get or create the api object
        let api = if let Ok(existing_api) = get_property(&self.inner, "api") {
            if !existing_api.is_null() && !existing_api.is_undefined() {
                existing_api
            } else {
                let new_api = js_sys::Object::new();
                js_sys::Reflect::set(&self.inner, jstr!("api"), &new_api)?;
                new_api.into()
            }
        } else {
            let new_api = js_sys::Object::new();
            js_sys::Reflect::set(&self.inner, jstr!("api"), &new_api)?;
            new_api.into()
        };

        js_sys::Reflect::set(&api, jstr!(key), value)?;
        Ok(())
    }

    /// Get the underlying JsValue
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Represents a token on the canvas
pub struct Token {
    inner: JsValue,
}

impl From<JsValue> for Token {
    fn from(inner: JsValue) -> Self {
        Token { inner }
    }
}

impl Token {
    /// Get the token's name
    pub fn name(&self) -> Option<String> {
        get_string_property(&self.inner, "name")
    }

    /// Get the token's ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "id")
    }

    /// Get the actor associated with this token
    pub fn actor(&self) -> Option<Actor> {
        get_property(&self.inner, "actor")
            .ok()
            .map(|inner| inner.into())
    }

    /// Get items directly from token.actor.items (works even with limited permissions)
    pub fn actor_items(&self) -> Vec<Item> {
        let mut items = Vec::new();

        if let Ok(actor) = get_property(&self.inner, "actor") {
            if let Ok(items_collection) = get_property(&actor, "items") {
                if let Ok(Some(iter)) = js_sys::try_iter(&items_collection) {
                    for item_result in iter {
                        if let Ok(inner) = item_result {
                            items.push(inner.into());
                        }
                    }
                }
            }
        }

        items
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Represents a user in Foundry
pub struct User {
    inner: JsValue,
}

impl From<JsValue> for User {
    fn from(inner: JsValue) -> Self {
        User { inner }
    }
}

impl User {
    /// Get the user's ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "id")
    }

    /// Get the user's name
    pub fn name(&self) -> Option<String> {
        get_string_property(&self.inner, "name")
    }

    /// Check if this user is a GM
    pub fn is_gm(&self) -> bool {
        get_property(&self.inner, "isGM")
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get a flag value
    pub fn get_flag(&self, scope: &str, key: &str) -> JsValue {
        // Call getFlag method on the document
        let get_flag_fn = get_property(&self.inner, "getFlag").expect("getFlag method");
        let args = js_sys::Array::new();
        args.push(jstr!(scope));
        args.push(jstr!(key));

        js_sys::Reflect::apply(get_flag_fn.unchecked_ref(), &self.inner, &args)
            .unwrap_or(JsValue::UNDEFINED)
    }

    /// Set a flag value
    pub async fn set_flag(&self, scope: &str, key: &str, value: &JsValue) -> Result<(), JsValue> {
        let set_flag_fn = get_property(&self.inner, "setFlag")?;
        let args = js_sys::Array::new();
        args.push(jstr!(scope));
        args.push(jstr!(key));
        args.push(value);

        let promise = js_sys::Reflect::apply(set_flag_fn.unchecked_ref(), &self.inner, &args)?;

        JsFuture::from(js_sys::Promise::from(promise)).await?;
        Ok(())
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Collection of users
pub struct UserCollection {
    inner: JsValue,
}

impl UserCollection {
    /// Get a user by ID
    pub fn get(&self, id: &str) -> Option<User> {
        let get_fn = get_property(&self.inner, "get").ok()?;
        let args = js_sys::Array::new();
        args.push(jstr!(id));

        let inner = js_sys::Reflect::apply(get_fn.unchecked_ref(), &self.inner, &args).ok()?;

        Some(inner.into())
    }

    /// Iterate over all users
    pub fn iter(&self) -> impl Iterator<Item = User> {
        let mut users = Vec::new();
        if let Ok(Some(iter)) = js_sys::try_iter(&self.inner) {
            for item in iter {
                if let Ok(inner) = item {
                    users.push(inner.into());
                }
            }
        }
        users.into_iter()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GMStrategy {
    Normal,
    Never,
    OnlyIfExclusive,
    #[default]
    IfNoPlayers,
}

impl GMStrategy {
    pub fn from_setting_value(value: &str) -> Self {
        match value {
            "normal" => GMStrategy::Normal,
            "never" => GMStrategy::Never,
            "onlyIfExclusive" => GMStrategy::OnlyIfExclusive,
            "ifNoPlayers" => GMStrategy::IfNoPlayers,
            _ => GMStrategy::default(),
        }
    }

    pub fn to_setting_value(&self) -> &'static str {
        match self {
            GMStrategy::Normal => "normal",
            GMStrategy::Never => "never",
            GMStrategy::OnlyIfExclusive => "onlyIfExclusive",
            GMStrategy::IfNoPlayers => "ifNoPlayers",
        }
    }

    pub fn from_settings(module_id: &str) -> Self {
        let value = get_setting(module_id, "gmStrategy");
        if let Some(s) = value.as_string() {
            Self::from_setting_value(&s)
        } else {
            Self::default()
        }
    }

    /// Register the GMStrategy setting
    pub fn register_setting(module_id: &str) {
        SettingConfig::new()
            .name("GM Ownership Strategy")
            .hint("How should GM ownership be counted when determining if damage popouts appear?")
            .scope("world")
            .config(true)
            .type_string()
            .default_string("onlyIfExclusive")
            .choices(&[
                ("normal", "GM uses default ownership setting"),
                ("never", "Never (GM never counts as owner)"),
                (
                    "onlyIfExclusive",
                    "Only if Exclusive (GM is not considered owner if any players own the actor)",
                ),
                (
                    "ifNoPlayers",
                    "If no players (GM is considered owner if no players own the actor)",
                ),
            ])
            .register(module_id, "gmStrategy");
    }
}

/// Represents an item in Foundry
pub struct Item {
    inner: JsValue,
}

impl From<JsValue> for Item {
    fn from(inner: JsValue) -> Self {
        Item { inner }
    }
}

impl Item {
    /// Get the item's name
    pub fn name(&self) -> String {
        get_string_property(&self.inner, "name").unwrap_or_else(|| "Unknown Item".to_string())
    }

    /// Get the item's type
    pub fn item_type(&self) -> Option<String> {
        get_string_property(&self.inner, "type")
    }

    /// Get the item's ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "_id")
    }

    /// Get the item's icon/image path
    pub fn img(&self) -> Option<String> {
        get_string_property(&self.inner, "img")
    }

    /// Get the item's carry type (worn, held, stowed, etc.)
    pub fn carry_type(&self) -> Option<String> {
        if let Ok(system) = get_property(&self.inner, "system") {
            if let Ok(equipped) = get_property(&system, "equipped") {
                if let Ok(carry_type) = get_property(&equipped, "carryType") {
                    return carry_type.as_string();
                }
            }
        }
        None
    }

    /// Check if the item is currently wielded with two hands
    pub fn is_two_handed(&self) -> bool {
        // Check system.equipped.handsHeld to see how many hands are being used
        if let Ok(system) = get_property(&self.inner, "system") {
            if let Ok(equipped) = get_property(&system, "equipped") {
                if let Ok(hands_held) = get_property(&equipped, "handsHeld") {
                    if let Some(hands) = hands_held.as_f64() {
                        return hands >= 2.0;
                    }
                }
            }
        }
        false
    }

    /// Check if this is a physical inventory item (not a spell, action, effect, etc.)
    pub fn is_physical_item(&self) -> bool {
        if let Some(item_type) = self.item_type() {
            let item_type = item_type.to_lowercase();
            matches!(
                item_type.as_str(),
                "weapon"
                    | "armor"
                    | "shield"
                    | "equipment"
                    | "consumable"
                    | "treasure"
                    | "backpack"
                    | "kit"
                    | "gear"
            )
        } else {
            false
        }
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Represents an actor in Foundry
pub struct Actor {
    inner: JsValue,
}

impl From<JsValue> for Actor {
    fn from(inner: JsValue) -> Self {
        Actor { inner }
    }
}

impl Actor {
    pub fn name(&self) -> String {
        get_string_property(&self.inner, "name").unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "id")
    }

    /// Check if a specific user owns this actor (ownership level >= 3)
    pub fn is_owned_by(&self, user: &User, count_gm: GMStrategy) -> bool {
        let Some(user_id) = user.id() else {
            return false;
        };
        let is_gm = user.is_gm();

        let Ok(ownership) = get_property(&self.inner, "ownership") else {
            return false;
        };
        let level_num = get_property(&ownership, &user_id)
            .map(|l| l.as_f64())
            .ok()
            .flatten()
            .unwrap_or_default();
        let owns = level_num >= 3.0;

        match count_gm {
            GMStrategy::Normal => owns,
            GMStrategy::Never => {
                if is_gm {
                    false
                } else {
                    owns
                }
            }
            GMStrategy::OnlyIfExclusive => {
                if is_gm && owns {
                    let has_non_gm_owners = self.has_non_gm_owners();
                    !has_non_gm_owners
                } else {
                    owns
                }
            }
            GMStrategy::IfNoPlayers => {
                if is_gm {
                    let has_non_gm_owners = self.has_non_gm_owners();
                    !has_non_gm_owners
                } else {
                    owns
                }
            }
        }
    }

    /// Check if there are any non-GM owners of this actor
    fn has_non_gm_owners(&self) -> bool {
        let Ok(ownership) = get_property(&self.inner, "ownership") else {
            return false;
        };
        let Ok(game) = Game::instance() else {
            return false;
        };
        let Ok(users) = game.users() else {
            return false;
        };

        let ownership_obj: &js_sys::Object = ownership.unchecked_ref();
        let keys = js_sys::Object::keys(ownership_obj);

        for i in 0..keys.length() {
            let Some(user_id) = keys.get(i).as_string() else {
                continue;
            };

            // Skip "default" ownership key
            if user_id == "default" {
                continue;
            }

            // Get ownership level (3 = OWNER)
            let Ok(level) = get_property(&ownership, &user_id) else {
                continue;
            };
            let Some(level_num) = level.as_f64() else {
                continue;
            };

            if level_num >= 3.0 {
                // Check if this owner is not a GM
                if let Some(user) = users.get(&user_id) {
                    if !user.is_gm() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if the current user owns this actor
    pub fn is_owned_by_current_user(&self, count_gm: GMStrategy) -> bool {
        let Ok(game) = Game::instance() else {
            return false;
        };
        let Ok(user) = game.user() else { return false };
        self.is_owned_by(&user, count_gm)
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Represents a chat message
pub struct Message {
    inner: JsValue,
}

impl From<JsValue> for Message {
    fn from(inner: JsValue) -> Self {
        Message { inner }
    }
}

impl Message {
    /// Get the message ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "_id")
    }

    /// Get the message content
    pub fn content(&self) -> Option<String> {
        get_string_property(&self.inner, "content")
    }

    /// Get the rolls associated with this message
    pub fn rolls(&self) -> Vec<Roll> {
        let mut rolls = Vec::new();
        if let Ok(rolls_val) = get_property(&self.inner, "rolls") {
            if let Ok(Some(iter)) = js_sys::try_iter(&rolls_val) {
                for item in iter {
                    if let Ok(inner) = item {
                        rolls.push(inner.into());
                    }
                }
            }
        }
        rolls
    }

    /// Get the first roll (convenience method)
    pub fn first_roll(&self) -> Option<Roll> {
        self.rolls().into_iter().next()
    }

    /// Get PF2e context information
    pub fn pf2e_context(&self) -> Option<DamageContext> {
        let flags = get_property(&self.inner, "flags").ok()?;
        let pf2e = get_property(&flags, "pf2e").ok()?;
        let context = get_property(&pf2e, "context").ok()?;

        let type_val = get_property(&context, "type").ok()?;
        let type_str = type_val.as_string()?;

        if type_str == "damage-roll" {
            Some(context.into())
        } else {
            None
        }
    }

    /// Get target tokens from pf2e-toolbelt targetHelper
    pub async fn toolbelt_targets(&self) -> Vec<Token> {
        let mut targets = Vec::new();

        let Ok(targets_array) = get_path!(&self.inner, "flags.pf2e-toolbelt.targetHelper.targets")
        else {
            return vec![];
        };

        let Ok(Some(iter)) = js_sys::try_iter(&targets_array) else {
            return vec![];
        };

        for target_result in iter {
            if let Ok(target_uuid) = target_result {
                if let Some(uuid_str) = target_uuid.as_string() {
                    if let Ok(token_js) = from_uuid_raw(&uuid_str).await {
                        targets.push(token_js.into());
                    }
                }
            }
        }

        targets
    }

    /// Pop out this message into its own window
    pub async fn popup(&self) -> Result<(), JsValue> {
        let global = js_sys::global();
        let foundry = get_property(&global, "foundry")?;
        let applications = get_property(&foundry, "applications")?;
        let sidebar = get_property(&applications, "sidebar")?;
        let apps = get_property(&sidebar, "apps")?;
        let chat_popout_class = get_property(&apps, "ChatPopout")?;

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, jstr!("message"), &self.inner)?;

        let args = js_sys::Array::new();
        args.push(&options);
        let popout = js_sys::Reflect::construct(chat_popout_class.unchecked_ref(), &args)?;

        let render_fn = get_property(&popout, "render")?;
        let render_args = js_sys::Array::new();
        render_args.push(&JsValue::from(true)); // force: true

        let promise = js_sys::Reflect::apply(render_fn.unchecked_ref(), &popout, &render_args)?;

        JsFuture::from(js_sys::Promise::from(promise)).await?;
        Ok(())
    }

    /// Create a new chat message
    pub async fn create(content: &str) -> Result<Message, JsValue> {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, jstr!("content"), jstr!(content))?;

        let global = js_sys::global();
        let chat_message_class = get_property(&global, "ChatMessage")?;
        let create_fn = get_property(&chat_message_class, "create")?;

        let args = js_sys::Array::new();
        args.push(&obj);

        let promise =
            js_sys::Reflect::apply(create_fn.unchecked_ref(), &chat_message_class, &args)?;

        let inner = JsFuture::from(js_sys::Promise::from(promise)).await?;
        Ok(inner.into())
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Represents a roll result
pub struct Roll {
    inner: JsValue,
}

impl From<JsValue> for Roll {
    fn from(inner: JsValue) -> Self {
        Roll { inner }
    }
}

impl Roll {
    /// Get the total result of the roll
    pub fn total(&self) -> f64 {
        get_f64_property(&self.inner, "total").unwrap_or(0.0)
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// PF2e damage context information
pub struct DamageContext {
    inner: JsValue,
}

impl From<JsValue> for DamageContext {
    fn from(inner: JsValue) -> Self {
        DamageContext { inner }
    }
}

impl DamageContext {
    /// Get the target actor UUID
    pub fn target_actor_uuid(&self) -> Option<String> {
        // First try the target.actor field (for targeted damage)
        if let Ok(target) = get_property(&self.inner, "target") {
            if !target.is_null() {
                if let Some(uuid) = get_string_property(&target, "actor") {
                    return Some(uuid);
                }
            }
        }

        // Fall back to the actor field directly (for self-targeted like healing)
        get_string_property(&self.inner, "actor")
    }

    /// Get the target actor
    pub async fn target_actor(&self) -> Result<Actor, JsValue> {
        let uuid = self
            .target_actor_uuid()
            .ok_or_else(|| JsValue::from_str("No target actor UUID"))?;
        Game::from_uuid(&uuid).await
    }

    /// Get the item name (weapon/spell that caused the damage)
    pub fn item_name(&self) -> Option<String> {
        let item = get_property(&self.inner, "item").ok()?;
        get_string_property(&item, "name")
    }

    /// Get the underlying JsValue (for compatibility)
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

pub struct HtmlElement {
    inner: JsValue,
}

impl From<JsValue> for HtmlElement {
    fn from(inner: JsValue) -> Self {
        HtmlElement { inner }
    }
}

impl HtmlElement {
    /// Query for a child element using a CSS selector
    pub fn query_selector(&self, selector: &str) -> Result<Option<HtmlElement>, JsValue> {
        let query_fn = get_property(&self.inner, "querySelector")?;
        let args = js_sys::Array::new();
        args.push(jstr!(selector));

        let result = js_sys::Reflect::apply(query_fn.unchecked_ref(), &self.inner, &args)?;

        if result.is_null() || result.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(HtmlElement { inner: result }))
        }
    }

    pub fn append_child(&self, child: &HtmlElement) -> Result<(), JsValue> {
        let append_fn = get_property(&self.inner, "appendChild")?;
        let args = js_sys::Array::new();
        args.push(&child.inner);
        js_sys::Reflect::apply(append_fn.unchecked_ref(), &self.inner, &args)?;
        Ok(())
    }

    pub fn set_attribute(&self, name: &str, value: &str) -> Result<(), JsValue> {
        let set_attr_fn = get_property(&self.inner, "setAttribute")?;
        let args = js_sys::Array::new();
        args.push(jstr!(name));
        args.push(jstr!(value));
        js_sys::Reflect::apply(set_attr_fn.unchecked_ref(), &self.inner, &args)?;
        Ok(())
    }

    pub fn set_class_name(&self, class_name: &str) -> Result<(), JsValue> {
        js_sys::Reflect::set(&self.inner, jstr!("className"), jstr!(class_name))?;
        Ok(())
    }

    pub fn set_inner_html(&self, html: &str) -> Result<(), JsValue> {
        js_sys::Reflect::set(&self.inner, jstr!("innerHTML"), jstr!(html))?;
        Ok(())
    }

    pub fn set_style(&self, style: &str) -> Result<(), JsValue> {
        js_sys::Reflect::set(&self.inner, jstr!("style"), jstr!(style))?;
        Ok(())
    }

    pub fn add_event_listener(
        &self,
        event_type: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> Result<(), JsValue> {
        let add_listener_fn = get_property(&self.inner, "addEventListener")?;
        let args = js_sys::Array::new();
        args.push(jstr!(event_type));
        args.push(callback.as_ref());
        js_sys::Reflect::apply(add_listener_fn.unchecked_ref(), &self.inner, &args)?;
        Ok(())
    }

    /// Insert HTML adjacent to this element
    /// position: "beforebegin", "afterbegin", "beforeend", or "afterend"
    pub fn insert_adjacent_html(&self, position: &str, html: &str) -> Result<(), JsValue> {
        let insert_fn = get_property(&self.inner, "insertAdjacentHTML")?;
        let args = js_sys::Array::new();
        args.push(jstr!(position));
        args.push(jstr!(html));
        js_sys::Reflect::apply(insert_fn.unchecked_ref(), &self.inner, &args)?;
        Ok(())
    }

    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

pub struct Document;

impl Document {
    /// Create a new HTML element
    pub fn create_element(tag_name: &str) -> Result<HtmlElement, JsValue> {
        let document = js_sys::Reflect::get(&js_sys::global(), jstr!("document"))?;
        let create_fn = get_property(&document, "createElement")?;
        let args = js_sys::Array::new();
        args.push(jstr!(tag_name));

        let element = js_sys::Reflect::apply(create_fn.unchecked_ref(), &document, &args)?;
        Ok(HtmlElement { inner: element })
    }
}

/// Represents the PF2E Bestiary Tracking application
pub struct BestiaryApp {
    inner: JsValue,
}

impl From<JsValue> for BestiaryApp {
    fn from(inner: JsValue) -> Self {
        BestiaryApp { inner }
    }
}

impl BestiaryApp {
    /// Get the selected monster's UUID from the app
    /// Returns None if no monster is selected or if the monster has no linked actor
    pub fn selected_monster_uuid(&self) -> Option<String> {
        let selected = get_property(&self.inner, "selected").ok()?;
        let monster = get_property(&selected, "monster").ok()?;

        if monster.is_null() || monster.is_undefined() {
            return None;
        }

        let system = get_property(&monster, "system").ok()?;
        let uuid = get_property(&system, "uuid").ok()?;

        if uuid.is_null() || uuid.is_undefined() {
            None
        } else {
            uuid.as_string()
        }
    }

    /// Get the underlying JsValue
    pub fn as_js_value(&self) -> &JsValue {
        &self.inner
    }
}

/// Module for Foundry VTT Applications
pub mod application {
    use super::*;

    /// Render a Handlebars template with context data
    pub async fn render_template<T: serde::ser::Serialize + ?Sized>(
        template_path: &str,
        context: &T,
    ) -> Result<String, JsValue> {
        let context_js = serde_wasm_bindgen::to_value(&context)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize context: {e}")))?;

        let global = js_sys::global();
        let render_template_fn = get_property(&global, "renderTemplate")?;

        let args = js_sys::Array::new();
        args.push(jstr!(template_path));
        args.push(&context_js);

        let promise =
            js_sys::Reflect::apply(render_template_fn.unchecked_ref(), &JsValue::NULL, &args)?;
        let html_value = JsFuture::from(js_sys::Promise::from(promise)).await?;

        html_value
            .as_string()
            .ok_or_else(|| JsValue::from_str("Template did not return a string"))
    }

    /// Show a simple dialog window with custom HTML content
    pub async fn show_dialog(
        title: &str,
        content: String,
        buttons: Vec<(&str, &str, Option<js_sys::Function>)>,
    ) -> Result<(), JsValue> {
        let global = js_sys::global();
        let dialog_class = get_property(&global, "Dialog")?;

        let noop_fn = js_sys::Function::new_no_args("");

        let buttons_obj = js_sys::Object::new();
        for (id, label, callback) in buttons {
            let button = js_sys::Object::new();
            js_sys::Reflect::set(&button, jstr!("label"), jstr!(label))?;
            let callback_fn = callback.unwrap_or_else(|| noop_fn.clone());
            js_sys::Reflect::set(&button, jstr!("callback"), &callback_fn)?;
            js_sys::Reflect::set(&buttons_obj, jstr!(id), &button)?;
        }

        let dialog_data = js_sys::Object::new();
        js_sys::Reflect::set(&dialog_data, jstr!("title"), jstr!(title))?;
        js_sys::Reflect::set(&dialog_data, jstr!("content"), jstr!(&content))?;
        js_sys::Reflect::set(&dialog_data, jstr!("buttons"), &buttons_obj)?;

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, jstr!("height"), jstr!("auto"))?;

        let args = js_sys::Array::new();
        args.push(&dialog_data);
        args.push(&options);
        let dialog = js_sys::Reflect::construct(dialog_class.unchecked_ref(), &args)?;

        let render_fn = get_property(&dialog, "render")?;
        let render_args = js_sys::Array::new();
        render_args.push(&JsValue::from(true));
        js_sys::Reflect::apply(render_fn.unchecked_ref(), &dialog, &render_args)?;

        Ok(())
    }
}
