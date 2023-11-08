mod config;
mod utils;
mod webserver;
mod widget;


use clap::{Parser, Subcommand};
use actix_web::rt;
use std::fs;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use widget::start_widgets;
use widget::WidgetInstanceInstruction;

// include the module "webserver"

const SOCKET_PATH: &str = "/tmp/awww.sock";

fn init_daemon() {
    // create the web server
    let server_handle = webserver::start_web_server();

    // the channel is used to send messages from the main thread to the ui thread
    let (tx, rx) = async_channel::bounded(1);

    let rx = Arc::new(Mutex::new(rx));

    start_widgets(rx);

    // in case of ctrl-c, we stop the web server, the widget will be automatically terminated
    ctrlc::set_handler(move || {
        rt::System::new().block_on(server_handle.stop(true));
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // create the unix socket event listener, once event received we pass the event from main thread
    // to the ui thread
    listen_unix_socket(tx);
}

fn listen_unix_socket(tx: async_channel::Sender<WidgetInstanceInstruction>) {
    let socket = Path::new(SOCKET_PATH);

    if socket.exists() {
        fs::remove_file(&socket).expect("failed to remove old socket file.");
    }

    let stream = match UnixListener::bind(&socket) {
        Err(_) => panic!("failed to bind socket"),
        Ok(stream) => stream,
    };

    for stream in stream.incoming() {
        match stream {
            Ok(stream) => {
                BufReader::new(stream).lines().for_each(|line| {
                    if let Err(_) = line {
                        return;
                    }

                    let line = line.unwrap();

                    println!("received: {}", line);

                    match line.as_str() {
                        "list" => { let _ = tx.send_blocking(WidgetInstanceInstruction::List); }
                        _ => {}
                    }
                });
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // list
    Init,
    List,
}

fn connect_to_socket() -> UnixStream {
    UnixStream::connect(SOCKET_PATH).expect("Failed to connect")
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("init");
            init_daemon();
        }
        Commands::List => {
            let mut stream = connect_to_socket();
            stream.write_all(b"list").expect("failed to write");
        }
    }
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
