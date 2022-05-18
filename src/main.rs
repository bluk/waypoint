// Copyright 2022 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    rust_2018_idioms,
    missing_docs,
    missing_debug_implementations,
    unused_lifetimes,
    unused_qualifications
)]

use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use clap::{Arg, Command};
use mio::{Events, Interest, Poll, Token};
use tracing::{debug, error, info};

struct Args {
    addr: SocketAddr,
}

fn get_args() -> io::Result<Args> {
    let matches = Command::new("Demo app")
        .version("0.0.1")
        .about("A demo application.")
        .arg(
            Arg::new("ip")
                .long("ip")
                .short('a')
                .env("IP_ADDR")
                .value_name("IP")
                .help("The IP address to bind to")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .env("PORT")
                .value_name("PORT")
                .help("The port to bind to")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let ip = matches.value_of("ip").unwrap_or("0.0.0.0");
    let port = matches.value_of("port").unwrap_or("6881");

    debug!(%ip, %port);

    Ok(Args {
        addr: format!("{}:{}", ip, port)
            .to_socket_addrs()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .into_iter()
            .next()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "could not lookup socket address")
            })?,
    })
}

fn run(socket_addr: SocketAddr) -> io::Result<()> {
    let mut socket = mio::net::UdpSocket::bind(socket_addr)?;
    info!(%socket_addr, "Listening");

    let dht_token = Token(0);

    let mut poll = Poll::new()?;
    poll.registry().register(
        &mut socket,
        dht_token,
        Interest::READABLE | Interest::WRITABLE,
    )?;

    let mut events = Events::with_capacity(1024);

    let mut buf = [0; 65535];

    loop {
        poll.poll(&mut events, None)?;

        'recv: loop {
            if events.is_empty() {
                break 'recv;
            }

            let (bytes_recv, src_addr) = match socket.recv_from(&mut buf) {
                Ok((len, src_addr)) => {
                    debug!(%src_addr, %len, "Received");
                    (len, src_addr)
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        break 'recv;
                    }

                    error!(%e, "Non-blocking receive error");
                    return Err(e);
                }
            };

            let filled_buf = &buf[..bytes_recv];

            match socket.send_to(filled_buf, src_addr) {
                Ok(len) => {
                    debug!(%src_addr, %len, "Echoed back");
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        error!(%e, "Could not send back");
                        break 'recv;
                    }

                    error!(%e, "Non-blocking send error");
                    return Err(e);
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let Args { addr } = get_args()?;

    run(addr)
}
