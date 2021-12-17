use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{JoinHandle, spawn};

fn send_reply(writer: &mut impl Write, response: &str) -> Result<()> {
    writer.write_all(response.as_bytes())?;
    writer.write_all("\r\n".as_bytes())?;
    writer.flush()
}

fn receive_command(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = String::new();
    if reader.read_line(&mut buf)? == 0 {
        Err(Error::from(ErrorKind::ConnectionAborted))
    } else {
        Ok(buf)
    }
}

fn command_iterator(command: &mut String) -> impl Iterator<Item=&str> {
    command.truncate(command.len() - 2);
    command.split_whitespace()
}

fn user(writer: &mut impl Write, username: &str) -> Result<()> {
    send_reply(writer, "230 User logged in, proceed.")
}

fn dispatch(writer: &mut impl Write, mut command: String) -> Result<()> {
    let mut iter = command_iterator(&mut command);
    let action = iter.next().unwrap();
    let params: Vec<&str> = iter.collect();

    match (action, params.len()) {
        ("USER", 1) => user(writer, params[0]),
        ("USER", _) => send_reply(writer, "501 Syntax error in parameters or arguments."),
        ("QUIT", 0) => {
            send_reply(writer, "221 Service closing control connection.");
            Err(Error::from(ErrorKind::NotConnected))
        }
        _ => send_reply(writer, "500 Syntax error, command unrecognized.")
    }
}

fn worker(mut reader: impl BufRead, mut writer: impl Write) -> Result<()> {
    send_reply(&mut writer, "220 Service ready for new user.")?;

    loop {
        dispatch(&mut writer, receive_command(&mut reader)?)?;
    }
}

fn spawn_worker(stream: TcpStream) -> JoinHandle<()> {
    spawn(|| {
        worker(
            BufReader::new(stream.try_clone().unwrap()),
            BufWriter::new(stream)
        ).unwrap();
    })
}

pub fn serve(address: &str) -> Result<()> {
    let mut listener = TcpListener::bind(address)?;
    let mut workers = Vec::new();

    for stream in listener.incoming() {
        workers.push(spawn_worker(stream?));
    }

    Ok(())
}

/*struct Server;
struct Command;
struct Reply;
struct ControlConnection;
struct DataConnection;
enum DataType {
    Ascii(FormatControl),
    Ebcdic(FormatControl),
    Image,
    Local,
}
enum FormatControl {
    NonPrint,
    Telnet,
    Carriage
}
struct File;
struct Record;
struct Page;*/
