//! Macros for Foundry VTT integration

/// Console print macro with crab emoji prefix
macro_rules! cprintln {
    ($($t:tt)*) => (crate::foundry::log(&format!("🦀{}", format_args!($($t)*))))
}
pub(crate) use cprintln;


///https://foundryvtt.com/api/classes/foundry.helpers.Hooks.html#on
/// 
/// ```
/// hook!("init", || {
///     cprintln!("Module initialized");
/// });
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
        let hook_id = $crate::foundry::hooks_on($hook_name, &closure);
        closure.into_js_value();
        hook_id
    }};

    // Async hook with no arguments
    ($hook_name:expr, async || $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn()>
        );
        let hook_id = $crate::foundry::hooks_on($hook_name, &closure);
        closure.into_js_value();
        hook_id
    }};

    // Sync hook with one argument
    ($hook_name:expr, |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(move |$arg $(: $arg_type)?| $body) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        let hook_id = $crate::foundry::hooks_on_1($hook_name, &closure);
        closure.into_js_value();
        hook_id
    }};

    // Async hook with one argument
    ($hook_name:expr, async |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg $(: $arg_type)?| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        let hook_id = $crate::foundry::hooks_on_1($hook_name, &closure);
        closure.into_js_value();
        hook_id
    }};

    // Sync hook with two arguments
    ($hook_name:expr, |$arg1:ident $(: $arg1_type:ty)?, $arg2:ident $(: $arg2_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg1 $(: $arg1_type)?, $arg2 $(: $arg2_type)?| $body) as Box<dyn Fn(::wasm_bindgen::JsValue, ::wasm_bindgen::JsValue)>
        );
        let hook_id = $crate::foundry::hooks_on_2($hook_name, &closure);
        closure.forget();
        hook_id
    }};

    // Async hook with two arguments
    ($hook_name:expr, async |$arg1:ident $(: $arg1_type:ty)?, $arg2:ident $(: $arg2_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg1 $(: $arg1_type)?, $arg2 $(: $arg2_type)?| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn(::wasm_bindgen::JsValue, ::wasm_bindgen::JsValue)>
        );
        let hook_id = $crate::foundry::hooks_on_2($hook_name, &closure);
        closure.forget();
        hook_id
    }};
}

/// https://foundryvtt.com/api/classes/foundry.helpers.Hooks.html#once
#[macro_export]
macro_rules! hook_once {
    // Sync hook with one argument
    ($hook_name:expr, |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(move |$arg $(: $arg_type)?| $body) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        $crate::foundry::hooks_once_1($hook_name, &closure);
        closure.into_js_value();
    }};

    // Async hook with one argument
    ($hook_name:expr, async |$arg:ident $(: $arg_type:ty)?| $body:block) => {{
        let closure = ::wasm_bindgen::prelude::Closure::wrap(
            Box::new(|$arg $(: $arg_type)?| {
                ::wasm_bindgen_futures::spawn_local(async move $body);
            }) as Box<dyn Fn(::wasm_bindgen::JsValue)>
        );
        $crate::foundry::hooks_once_1($hook_name, &closure);
        closure.into_js_value();
    }};
}

/// Convenience macro for creating JsValue string references
#[macro_export]
macro_rules! jstr {
    ($str:expr) => {
        &::wasm_bindgen::JsValue::from_str($str)
    };
}
