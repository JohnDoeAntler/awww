use std::env;
use std::path::Path;
use gtk_layer_shell::Layer;

pub fn get_widget_dir_path () -> String {
  let xdg_config_home_path = env::var("XDG_CONFIG_HOME").unwrap_or(format!("{}/.config", std::env::var("HOME").unwrap()));
  return Path::new(&xdg_config_home_path).join("awww").to_str().unwrap().to_string();
}

pub fn string_to_layer (layer: String) -> Result<Layer, ()> {
  match layer.as_str() {
    "bottom" => Ok(Layer::Bottom),
    "background" => Ok(Layer::Background),
    "overlay" => Ok(Layer::Overlay),
    "top" => Ok(Layer::Top),
    _ => Err(()),
  }
}

