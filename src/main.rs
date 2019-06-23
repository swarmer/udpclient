use std::io::Read;

use clap::{value_t, App, Arg, SubCommand};


pub const VERSION: &str = env!("CARGO_PKG_VERSION");


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
}


#[derive(Clone, Debug)]
pub struct SendArgs {
    pub host: String,
    pub port: u16,
}


pub fn cli_send(args: &SendArgs) {
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


pub fn run() -> i32 {
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
                        println!("{}", e.message);
                        return 1;
                    }
                },
            };
            cli_send(&send_args);
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
