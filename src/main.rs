use std::io;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};

use clap::{value_t, App, Arg, SubCommand};
use futures::try_ready;
use tokio;
use tokio::net::UdpSocket;
use tokio::prelude::*;


const VERSION: &str = env!("CARGO_PKG_VERSION");


fn build_app() -> App<'static, 'static> {
    App::new("udp")
        .version(VERSION)
        .author("Anton Barkovsky")
        .about("A command line UDP testing/debugging utility")
        .subcommand(
            SubCommand::with_name("send")
                .about("Send a UDP packet")
                .arg(
                    Arg::with_name("HOST")
                        .required(true)
                        .index(1)
                        .help("Target host"),
                )
                .arg(
                    Arg::with_name("PORT")
                        .required(true)
                        .index(2)
                        .help("Target port"),
                ),
        )
        .subcommand(
            SubCommand::with_name("listen")
                .about("Listen to UDP packets")
                .arg(
                    Arg::with_name("PORT")
                        .required(true)
                        .index(1)
                        .help("Port on which to receive packets"),
                ),
        )
}


#[derive(Clone, Debug)]
struct ListenArgs {
    pub port: u16,
}


struct UdpListenerFuture {
    socket: UdpSocket,
}

impl Future for UdpListenerFuture {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            let mut buf = vec![0; 65536];
            let (size, addr) = try_ready!(self.socket.poll_recv_from(&mut buf));

            eprintln!("Received a packet from {}", addr);
            let packet_str = String::from_utf8_lossy(&buf[..size]);
            eprintln!("UTF-8 representation: {:?}", packet_str);
            eprintln!("Byte array representation: {:?}", &buf[..size]);
            eprintln!();
        }
    }
}


fn cli_listen(args: &ListenArgs) {
    let addr = SocketAddr::from(("0.0.0.0".parse::<IpAddr>().unwrap(), args.port));
    let socket = UdpSocket::bind(&addr).expect("Failed to open a UDP socket");
    eprintln!("Listening on port {}...", args.port);

    let listener_future = UdpListenerFuture { socket };
    tokio::run(listener_future.map_err(|e| eprintln!("Error: {:?}", e)));
}


#[derive(Clone, Debug)]
struct SendArgs {
    pub host: String,
    pub port: u16,
}


fn cli_send(args: &SendArgs) {
    let mut buf = vec![];
    std::io::stdin()
        .read_to_end(&mut buf)
        .expect("Failed reading packet from stdin");

    let socket =
        std::net::UdpSocket::bind("0.0.0.0:0").expect("Failed to open a socket");
    socket
        .send_to(&buf, (&*args.host, args.port))
        .expect("Failed to send the packet");
}


fn run() -> i32 {
    let mut app = build_app();
    let matches = app.clone().get_matches();

    match matches.subcommand_name() {
        Some("send") => {
            let submatches = matches.subcommand_matches("send").unwrap();
            let send_args = SendArgs {
                host: submatches.value_of("HOST").unwrap().to_string(),
                port: match value_t!(submatches, "PORT", u16) {
                    Ok(port) => port,
                    Err(e) => {
                        eprintln!("{}", e.message);
                        return 1;
                    }
                },
            };
            cli_send(&send_args);
            0
        }
        Some("listen") => {
            let submatches = matches.subcommand_matches("listen").unwrap();
            let listen_args = ListenArgs {
                port: match value_t!(submatches, "PORT", u16) {
                    Ok(port) => port,
                    Err(e) => {
                        eprintln!("{}", e.message);
                        return 1;
                    }
                },
            };
            cli_listen(&listen_args);
            0
        }
        None => {
            app.print_help().unwrap();
            println!();
            0
        }
        Some(_) => {
            panic!();
        }
    }
}


fn main() {
    std::process::exit(run());
}
