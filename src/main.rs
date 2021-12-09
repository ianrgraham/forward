// use async_net::unix::{UnixListener, UnixStream};
// use futures_lite::*;
use std::os::unix::net::*;
use std::io::prelude::*;
use std::net::Shutdown;
use std::fs::File;
use daemonize::Daemonize;
use std::path::PathBuf;
use structopt::StructOpt;

use std::io::{self, BufRead};

// use futures_lite::stream::{self, StreamExt};


#[derive(StructOpt)]
struct Opt {
    #[structopt(short, default_value="/tmp/forward", parse(from_os_str))]
    socket_address: PathBuf,

    #[structopt(short, default_value="/tmp/forward.log", parse(from_os_str))]
    log: PathBuf,

    #[structopt(subcommand)]
    cmd: Option<Command>,
}

// add commands to cleanup unix socket, list running 
#[derive(StructOpt)]
enum Command {
    Server{
        #[structopt(long)]
        reset: bool,
        
        #[structopt(long)]
        kill: bool
    }
}

fn query_daemon(opt: Opt) -> anyhow::Result<()> {

    println!("connecting");
    let mut stream = UnixStream::connect(opt.socket_address)?;
    println!("writing");
    stream.write_all(b"hello world from client\n")?;
    let mut response = String::new();

    let mut buf = io::BufReader::new(stream);
    println!("reading");
    buf.read_line(&mut response)?;
    println!("{}", response);
    Ok(())
}

fn fork_daemon(opt: Opt) -> anyhow::Result<()> {

    let stdout = File::create(opt.socket_address.with_extension("out")).unwrap();
    let stderr = File::create(opt.socket_address.with_extension("err")).unwrap();
    let listener = UnixListener::bind(opt.socket_address)?;

    let daemonize = Daemonize::new()
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                /* connection succeeded */
                let mut response = String::new();
                let mut buf = io::BufReader::new(&stream);
                println!("reading");
                buf.read_line(&mut response)?;
                println!("{}", response);
                stream.write_all(b"hello world from daemon\n")?;
                println!("all done on daemon side");
            }
            Err(err) => {
                /* connection failed */
                break;
            }
        }
    }
    Ok(())
}


fn main() -> anyhow::Result<()> {

    // parse command line arguments
    let opt = Opt::from_args();
    
    if opt.cmd.is_some() {
        fork_daemon(opt)
    }
    else {
        query_daemon(opt)
    }
}
