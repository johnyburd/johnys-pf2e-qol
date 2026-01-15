use crate::{ID, foundry::get_setting};

pub mod auto_popout;
pub mod equipment_observation;

fn is_enabled(key: &str) -> bool {
    let value = get_setting(ID, key);
    value.as_bool().unwrap_or(true)
}

pub fn init_features() {
    equipment_observation::init();
    auto_popout::init();
}