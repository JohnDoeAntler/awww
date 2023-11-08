use std::{path::Path, fs};

use serde::{Serialize, Deserialize};
use crate::utils::get_widget_dir_path;

const LAYERS: [&str; 4] = [
  "background",
  "bottom",
  "overlay",
  "top",
];

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WindowAnchorConfiguration {
  pub top: Option<bool>,
  pub right: Option<bool>,
  pub bottom: Option<bool>,
  pub left: Option<bool>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WindowMarginConfiguration {
  pub top: Option<i32>,
  pub right: Option<i32>,
  pub bottom: Option<i32>,
  pub left: Option<i32>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WindowSizeConfiguration {
  pub width: Option<i32>,
  pub height: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetConfiguration {
  // required items
  #[serde(skip)]
  pub application_name: String,
  #[serde(skip)]
  pub layer: String,
  // optional items
  pub monitors: Option<Vec<i32>>,
  pub exclusive: Option<bool>,
  pub anchors: Option<WindowAnchorConfiguration>,
  pub margins: Option<WindowMarginConfiguration>,
  pub default_size: Option<WindowSizeConfiguration>,
  pub keyboard_interactivity: Option<bool>,
  pub keyboard_mode: Option<String>,
  pub click_through: Option<bool>,
  pub default_visible: Option<bool>,
}

impl WidgetConfiguration {
  pub fn new (application_name: String, layer: String) -> WidgetConfiguration {
    WidgetConfiguration {
      application_name,
      layer,
      anchors: Some(WindowAnchorConfiguration {
          top: Some(true),
          right: Some(true),
          bottom: Some(true),
          left: Some(true),
        }),
      margins: Some(WindowMarginConfiguration {
        top: Some(0),
        right: Some(0),
        bottom: Some(0),
        left: Some(0),
      }),
      click_through: Some(true),
      exclusive: Some(false),
      monitors: None,
      default_size: None,
      keyboard_interactivity: Some(true),
      keyboard_mode: Some("on_demand".to_string()),
      default_visible: Some(true),
    }
  }

  pub fn get_webview_root (&self) -> String {
    format!("/{}/{}", self.layer, self.application_name)
  }
}

pub fn get_widget_configurations () -> Vec<WidgetConfiguration> {
  let widget_dir_path = get_widget_dir_path();

  let available_layers = LAYERS.iter().filter(|e| Path::new(format!("{widget_dir_path}/{e}").as_str()).is_dir());
  let mut configs: Vec<WidgetConfiguration> = vec![];

  for layer_dir in available_layers {
    let abs_dir = format!("{widget_dir_path}/{}", layer_dir);

    let application_dirs = Path::new(abs_dir.as_str())
        .read_dir()
        .unwrap()
        .filter(|e| e.as_ref().unwrap().file_type().unwrap().is_dir());

    for application in application_dirs {
      let app = application.unwrap();
      let config = app.path()
          .read_dir()
          .unwrap()
          .into_iter()
          .map(|e| e.unwrap())
          .find(|e| e.file_name().into_string().unwrap() == "config.json");

      let index = app
          .path()
          .read_dir()
          .unwrap()
          .into_iter()
          .map(|e| e.unwrap())
          .find(|e| e.file_name().into_string().unwrap() == "index.html");

      if index.is_none() {
        continue;
      }

      let config = match config {
        Some(config) => {
          let mut ret: WidgetConfiguration = serde_json::from_str(
            &std::fs::read_to_string(config.path()).unwrap()
          ).unwrap();
          ret.application_name = app
            .file_name()
            .into_string()
            .unwrap();
          ret.layer = layer_dir.to_string();
          ret
        },
        None => {
          let config = WidgetConfiguration::new(
            app
              .file_name()
              .into_string()
              .unwrap(),
            layer_dir.to_string(),
          );

          fs::write(
            format!("{}/config.json", app.path().to_string_lossy()),
            serde_json::to_string_pretty(&config).unwrap()
          ).unwrap();

          println!("config {} created", app.path().to_string_lossy());

          config
        }
      };

      if config.anchors.is_none() || config.anchors.as_ref().is_some_and(|e|
        e.top.is_none() && e.bottom.is_none() && e.left.is_none() && e.right.is_none()
      ) {
        println!("config {} ignored because no anchor is set", app.path().to_string_lossy());
        continue;
      }

      configs.push(config);
    }
  }

  configs
}
