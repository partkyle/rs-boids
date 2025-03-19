use bevy::app::PluginGroup;
use bevy::app::PluginGroupBuilder;
use bevy::DefaultPlugins;

#[cfg(target_os = "windows")]
pub fn default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.build()
}

#[cfg(target_os = "linux")]
pub fn default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.build()
}

#[cfg(target_arch = "wasm32")]
pub fn default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.build()
}
