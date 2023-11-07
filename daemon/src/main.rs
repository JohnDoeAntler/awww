mod widget;
mod config;
mod utils;
mod webserver;

use actix_web::rt;
use widget::WidgetInstanceInstruction;
use widget::start_widgets;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::fs;

// include the module "webserver"

const SOCKET_PATH: &str = "/tmp/awww.sock";

fn main() {
  // let (tx, rx) = mpsc::channel();
  let server_handle = webserver::start_web_server();

  let (tx, rx) = mpsc::channel();

  start_widgets(rx);

  ctrlc::set_handler(move || {
    rt::System::new().block_on(server_handle.stop(true));
    std::process::exit(0);
  }).expect("Error setting Ctrl-C handler");

  listen_unix_socket(tx);
}

fn listen_unix_socket(tx: mpsc::Sender<WidgetInstanceInstruction>) {
    let socket = Path::new(SOCKET_PATH);

    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let stream = match UnixListener::bind(&socket) {
        Err(_) => panic!("failed to bind socket"),
        Ok(stream) => stream,
    };

    for stream in stream.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client");
                BufReader::new(stream).lines().for_each(|line| {
                    println!("{}", line.unwrap());
                    // create a arg parser
                });
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    println!();
}

// manager.register_script_message_handler("external");
// manager.connect_script_message_received(Some("external"), move |_ucm, jsr| {
//   let msg = jsr.js_value().unwrap().to_string();

//   match msg.as_str() {
//     "ping" => {
//       println!("A ping received ");
//       // I want to response a pong back
//       c.as_ref().run_javascript("console.log('hello world')", gtk::gio::Cancellable::NONE, |r| { });
//     }
//     _ => {}
//   }
// });

// println!("executed: {}", );
