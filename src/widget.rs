use glib::clone;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;
use gdk::cairo::RectangleInt;
use gdk::cairo::Region;
use gtk::{ApplicationWindow, Application, gdk::Display};
use gio::{prelude::*, ApplicationFlags};
// use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt};
use gtk::prelude::*;
use gtk_layer_shell::{Edge, LayerShell};
use webkit2gtk::WebViewExt;
use webkit2gtk::WebView;
use crate::config::{WidgetConfiguration, get_widget_configurations};
use crate::utils;
use gtk::glib;

#[derive(Clone, Debug)]
struct WidgetInstance {
  id: i32,
  layer: String,
  application_name: String,
  window: ApplicationWindow,
  webview: WebView,
}

static APPLICATION_ID: Mutex<i32> = Mutex::new(0);

impl WidgetInstance {
  fn new (layer: String, application_name: String, window: ApplicationWindow, webview: WebView) -> WidgetInstance {
    // generate unique id
    let mut id = APPLICATION_ID.lock().expect("Could not lock APPLICATION_ID");
    *id += 1;
    WidgetInstance {
      id: *id,
      layer,
      application_name,
      window,
      webview,
    }
  }
}

#[derive(Clone, Debug)]
pub enum WidgetInstanceInstruction {
    // 1. layer
    // 2. application name
    Show(String, String),
    // 1. layer
    // 2. application_name
    Hide(String, String),
    // 1. layer
    // 2. application_name
    Reload(String, String),
    List,
}

fn create_webview (root: String) -> WebView {
  let webview = WebView::new();
  webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
  webview.load_uri(format!("http://127.0.0.1:8082/{}", root).as_str());
  webview
}

fn create_window(application: &Application) -> ApplicationWindow {
  let window = ApplicationWindow::new(application);
  window.set_visual(Some(&WidgetExt::screen(&window).unwrap().rgba_visual().unwrap()));
  window.set_decorated(false);
  window.set_app_paintable(true);
  window
}

fn set_layer_shell(window: &ApplicationWindow, config: &WidgetConfiguration) {
  // Display above normal windows
  window.set_layer(utils::string_to_layer(config.layer.to_owned()).unwrap());

  if let Some(anchors) = &config.anchors {
    if let Some(top) = anchors.top {
      window.set_anchor(Edge::Top, top);
    }

    if let Some(right) = anchors.right {
      window.set_anchor(Edge::Right, right);
    }

    if let Some(bottom) = anchors.bottom {
      window.set_anchor(Edge::Bottom, bottom);
    }

    if let Some(left) = anchors.left {
      window.set_anchor(Edge::Left, left);
    }
  }

  if let Some(margins) = &config.margins {
    if let Some(top) = margins.left {
      window.set_layer_shell_margin(Edge::Left, top);
    }

    if let Some(right) = margins.right {
      window.set_layer_shell_margin(Edge::Right, right);
    }

    if let Some(bottom) = margins.bottom {
      window.set_layer_shell_margin(Edge::Bottom, bottom);
    }

    if let Some(left) = margins.left {
      window.set_layer_shell_margin(Edge::Left, left);
    }
  }

  if let Some(exclusive) = &config.exclusive {
    if exclusive.to_owned() {
      window.auto_exclusive_zone_enable();
    }
  }

  if let Some(keyboard_interactivity) = &config.keyboard_interactivity {
    window.set_keyboard_interactivity(keyboard_interactivity.to_owned());
  }

  // LayerShell::();
}

fn apply_layer_shell(window: &ApplicationWindow, config: &WidgetConfiguration) {
  // Before the window is first realized, set it up to be a layer surface
  window.init_layer_shell();
  set_layer_shell(window, config);
}

fn show_window(window: &ApplicationWindow, config: &WidgetConfiguration) {
  window.show_all();
  window.set_title(config.application_name.as_str());

  if let Some(size) = &config.default_size {
    if let Some(width) = size.width {
      window.set_width_request(width);
    }

    if let Some(height) = size.height {
      window.set_height_request(height);
    }
  }

  if let Some(click_through) = &config.click_through {
    if click_through.to_owned() {
      let rectangle_int = RectangleInt::new(0,0,0,0);
      let rectangle = Region::create_rectangle(&rectangle_int);
      window.input_shape_combine_region(Some(&rectangle));
    }
  }
}

fn build_widget(application: &Application, config: WidgetConfiguration) -> Vec<WidgetInstance> {
  let widget_monitors: Vec<i32> = {
    let count = Display::default().unwrap().n_monitors();

    if let Some(monitors) = config.to_owned().monitors {
      monitors.into_iter().filter(|e| *e >= 0 && *e < count).collect()
    } else {
      (0 .. count).collect()
    }
  };

  println!("widget monitors: {:?}", widget_monitors);

  widget_monitors.into_iter().map(|monitor| {
    // create webview by using the config root
    let webview = create_webview(config.get_webview_root());
    // create a window by the gtk application
    let window = create_window(&application);
    // add the webview into the window
    window.add(&webview);

    // make it either overlay or background
    apply_layer_shell(window.as_ref(), &config);
    // set monitor
    window.set_monitor(&Display::default().unwrap().monitor(monitor).unwrap());
    // make it visible
    show_window(window.as_ref(), &config);
    // push back the widget instance
    WidgetInstance::new(
      config.layer.clone(),
      config.application_name.clone(),
      window,
      webview,
    )
  }).collect()
}

pub fn start_widgets(rx: Arc<Mutex<async_channel::Receiver<WidgetInstanceInstruction>>>) {
  thread::spawn(move || {
    gtk::init().unwrap();

    let app = Application::new(Some("org.gnome.webkit6-rs.example"), ApplicationFlags::FLAGS_NONE);

    app.connect_activate(move | app | {
      let all = Arc::new(
        Mutex::new(
          get_widget_configurations().into_iter().map(|config| build_widget(&app, config)).flatten().collect::<Vec<WidgetInstance>>()
        )
      );

      glib::spawn_future_local(clone!(@strong all, @strong rx => async move {
        let t = rx.lock().expect("could not lock rx");

        println!("running");

        loop {
          println!("running something");
          match t.recv().await {
              Ok(r) => {
                  println!("reading something {:?}", r);

                  let mut all = all.lock().unwrap();

                  match r {
                    WidgetInstanceInstruction::List => {
                        println!("received");
                        for e in all.to_owned() {
                            println!("{}: {}", e.application_name, e.id);
                        }
                    },
                    WidgetInstanceInstruction::Show(layer, app) => {
                        for e in all.to_owned() {
                            if e.layer == layer && e.application_name == app {
                                e.window.show();
                            }
                        }
                        *all = vec![];
                    },
                    WidgetInstanceInstruction::Hide(layer, app) => {
                    },
                    WidgetInstanceInstruction::Reload(layer, app) => {
                    },
                  }
              },
              Err(_) => {}
          }
        }
      }));
    });

    app.run_with_args::<&str>(&[]);
  });
}

