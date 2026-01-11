// Foundry VTT API bindings for Rust/WASM
//
// This module provides a Rust-native interface to Foundry VTT's JavaScript API.
// It wraps the JavaScript objects in strongly-typed Rust structs.
#![allow(dead_code)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);
}

macro_rules! cprintln {
    ($($t:tt)*) => (log(&format!("🦀{}", format_args!($($t)*))))
}
pub(crate) use cprintln;

/// Register a Foundry VTT hook with automatic async support
///
/// # Examples
///
/// ```
/// // Sync hook with no arguments
/// hook!("init", || {
///     cprintln!("Module initialized");
/// });
/// // Async hook with one argument
/// hook!("createChatMessage", async |message: JsValue| {
///     handle_message(message).await;
/// });
/// ```
#[macro_export]
macro_rules! hook {
    // Sync hook with no arguments
    ($hook_name:expr, || $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|| $body) as Box<dyn Fn()>
        );
        $crate::foundry::hooks_on($hook_name, &closure);
        closure.forget();
    }};

    // Async hook with no arguments
    ($hook_name:expr, async || $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn()>
        );
        $crate::foundry::hooks_on($hook_name, &closure);
        closure.forget();
    }};

    // Sync hook with one argument
    ($hook_name:expr, |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg $(: $arg_type)?| $body) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        $crate::foundry::hooks_on_1($hook_name, &closure);
        closure.forget();
    }};

    // Async hook with one argument
    ($hook_name:expr, async |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg $(: $arg_type)?| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        $crate::foundry::hooks_on_1($hook_name, &closure);
        closure.forget();
    }};
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = Hooks, js_name = on)]
    pub fn hooks_on(hook: &str, r#fn: &Closure<dyn Fn()>) -> i32;

    #[wasm_bindgen(js_namespace = Hooks, js_name = on)]
    pub fn hooks_on_1(hook: &str, r#fn: &Closure<dyn Fn(JsValue)>) -> i32;

    // fromUuid global function
    #[wasm_bindgen(catch, js_name = fromUuid)]
    async fn from_uuid_raw(uuid: &str) -> Result<JsValue, JsValue>;

    // Game settings API
    #[wasm_bindgen(js_namespace = ["game", "settings"], js_name = register)]
    fn register_setting_raw(module: &str, key: &str, data: &JsValue);

    #[wasm_bindgen(js_namespace = ["game", "settings"], js_name = get)]
    pub fn get_setting(module: &str, key: &str) -> JsValue;
}

fn get_property(obj: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
}

fn get_string_property(obj: &JsValue, key: &str) -> Option<String> {
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
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("name"),
            &JsValue::from_str(name),
        )
        .unwrap();
        self
    }

    pub fn hint(self, hint: &str) -> Self {
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("hint"),
            &JsValue::from_str(hint),
        )
        .unwrap();
        self
    }

    pub fn scope(self, scope: &str) -> Self {
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("scope"),
            &JsValue::from_str(scope),
        )
        .unwrap();
        self
    }

    pub fn config(self, config: bool) -> Self {
        js_sys::Reflect::set(&self.config, &JsValue::from_str("config"), &JsValue::from(config))
            .unwrap();
        self
    }

    pub fn type_string(self) -> Self {
        let global = js_sys::global();
        let string_constructor =
            js_sys::Reflect::get(&global, &JsValue::from_str("String")).unwrap();
        js_sys::Reflect::set(&self.config, &JsValue::from_str("type"), &string_constructor)
            .unwrap();
        self
    }

    pub fn type_boolean(self) -> Self {
        let global = js_sys::global();
        let boolean_constructor =
            js_sys::Reflect::get(&global, &JsValue::from_str("Boolean")).unwrap();
        js_sys::Reflect::set(&self.config, &JsValue::from_str("type"), &boolean_constructor)
            .unwrap();
        self
    }

    pub fn type_number(self) -> Self {
        let global = js_sys::global();
        let number_constructor =
            js_sys::Reflect::get(&global, &JsValue::from_str("Number")).unwrap();
        js_sys::Reflect::set(&self.config, &JsValue::from_str("type"), &number_constructor)
            .unwrap();
        self
    }

    pub fn default_string(self, default: &str) -> Self {
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("default"),
            &JsValue::from_str(default),
        )
        .unwrap();
        self
    }

    pub fn default_bool(self, default: bool) -> Self {
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("default"),
            &JsValue::from(default),
        )
        .unwrap();
        self
    }

    pub fn default_number(self, default: f64) -> Self {
        js_sys::Reflect::set(
            &self.config,
            &JsValue::from_str("default"),
            &JsValue::from(default),
        )
        .unwrap();
        self
    }

    pub fn choices(self, choices: &[(&str, &str)]) -> Self {
        let choices_obj = js_sys::Object::new();
        for (key, value) in choices {
            js_sys::Reflect::set(
                &choices_obj,
                &JsValue::from_str(key),
                &JsValue::from_str(value),
            )
            .unwrap();
        }
        js_sys::Reflect::set(&self.config, &JsValue::from_str("choices"), &choices_obj).unwrap();
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
        let inner = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("game"))?;
        Ok(Game { inner })
    }

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
        args.push(&JsValue::from_str(id));

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
        args.push(&JsValue::from_str(scope));
        args.push(&JsValue::from_str(key));

        js_sys::Reflect::apply(get_flag_fn.unchecked_ref(), &self.inner, &args)
            .unwrap_or(JsValue::UNDEFINED)
    }

    /// Set a flag value
    pub async fn set_flag(&self, scope: &str, key: &str, value: &JsValue) -> Result<(), JsValue> {
        let set_flag_fn = get_property(&self.inner, "setFlag")?;
        let args = js_sys::Array::new();
        args.push(&JsValue::from_str(scope));
        args.push(&JsValue::from_str(key));
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
        args.push(&JsValue::from_str(id));

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
    Always,
    Never,
    #[default]
    OnlyIfExclusive,
}

impl GMStrategy {
    pub fn from_setting_value(value: &str) -> Self {
        match value {
            "always" => GMStrategy::Always,
            "never" => GMStrategy::Never,
            "onlyIfExclusive" => GMStrategy::OnlyIfExclusive,
            _ => GMStrategy::default(),
        }
    }

    pub fn to_setting_value(&self) -> &'static str {
        match self {
            GMStrategy::Always => "always",
            GMStrategy::Never => "never",
            GMStrategy::OnlyIfExclusive => "onlyIfExclusive",
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
                ("always", "Always (GM always gets popouts)"),
                ("never", "Never (GM never gets popouts)"),
                ("onlyIfExclusive", "Only if Exclusive (GM only gets popouts if no players own the actor)"),
            ])
            .register(module_id, "gmStrategy");
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
    /// Get the actor's name
    pub fn name(&self) -> String {
        get_string_property(&self.inner, "name").unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get the actor's ID
    pub fn id(&self) -> Option<String> {
        get_string_property(&self.inner, "id")
    }

    /// Check if a specific user owns this actor (ownership level >= 3)
    pub fn is_owned_by(&self, user: &User, count_gm: GMStrategy) -> bool {
        let Some(user_id) = user.id() else {
            return false;
        };
        let is_gm = user.is_gm();

        // Check basic ownership
        let Ok(ownership) = get_property(&self.inner, "ownership") else {
            return false;
        };
        let Ok(level) = get_property(&ownership, &user_id) else {
            return false;
        };
        let Some(level_num) = level.as_f64() else {
            return false;
        };
        let owns = level_num >= 3.0;

        match count_gm {
            GMStrategy::Always => owns,
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

    /// Get all users who own this actor (ownership level >= 3)
    pub fn owner_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        let Ok(ownership) = get_property(&self.inner, "ownership") else {
            return names;
        };
        let Ok(game) = Game::instance() else {
            return names;
        };
        let Ok(users) = game.users() else {
            return names;
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
                if let Some(user) = users.get(&user_id) {
                    if let Some(name) = user.name() {
                        names.push(name);
                    }
                }
            }
        }

        names
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

        // Check if this is a damage roll
        let type_val = get_property(&context, "type").ok()?;
        let type_str = type_val.as_string()?;

        if type_str == "damage-roll" {
            Some(context.into())
        } else {
            None
        }
    }

    /// Pop out this message into its own window
    pub async fn popup(&self) -> Result<(), JsValue> {
        let Some(id) = self.id() else {
            return Err(JsValue::from_str("Message has no ID"));
        };

        cprintln!("Attempting to pop out message: {}", id);

        // Navigate to foundry.applications.sidebar.apps.ChatPopout
        let global = js_sys::global();
        let foundry = get_property(&global, "foundry")?;
        let applications = get_property(&foundry, "applications")?;
        let sidebar = get_property(&applications, "sidebar")?;
        let apps = get_property(&sidebar, "apps")?;
        let chat_popout_class = get_property(&apps, "ChatPopout")?;

        // Create options object with message property
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &JsValue::from_str("message"), &self.inner)?;

        // Create new ChatPopout instance
        let args = js_sys::Array::new();
        args.push(&options);
        let popout = js_sys::Reflect::construct(chat_popout_class.unchecked_ref(), &args)?;

        cprintln!("Created ChatPopout instance");

        // Render the popout
        let render_fn = get_property(&popout, "render")?;
        let render_args = js_sys::Array::new();
        render_args.push(&JsValue::from(true)); // force: true

        let promise = js_sys::Reflect::apply(render_fn.unchecked_ref(), &popout, &render_args)?;

        // Await the render promise
        JsFuture::from(js_sys::Promise::from(promise)).await?;

        cprintln!("Rendered ChatPopout successfully");
        Ok(())
    }

    /// Create a new chat message
    pub async fn create(content: &str) -> Result<Message, JsValue> {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("content"),
            &JsValue::from_str(content),
        )?;

        // Get ChatMessage class
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
        let target = get_property(&self.inner, "target").ok()?;
        get_string_property(&target, "actor")
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
