use std::thread;

mod vol_voleur_msg;
mod pulse_iface;

extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate bytes;

use bytes::{BytesMut, Bytes, BufMut};

use futures::future::{Either, ok as future_ok};
use futures::{Stream,Future,Sink,Poll};
use futures::sync::mpsc;
use futures::sync::mpsc::{Sender, Receiver};
use futures::future::{loop_fn, Loop};

use hyper::{Get, Post, StatusCode};
use hyper::server::{Http, Service, Request, Response};
use hyper::mime;
use hyper::header::{ContentType, Connection, AccessControlAllowOrigin};
use hyper::Chunk;

use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use std::collections::HashMap;

extern crate url;
use url::form_urlencoded;

enum Case<A,B>{A(A), B(B)}

impl<A,B,I,E> Future for Case<A,B>
where 
    A:Future<Item=I, Error=E>,
    B:Future<Item=I, Error=E>,
{
    type Item = I;
    type Error = E;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error>{
        match *self {
            Case::A(ref mut a) => a.poll(),
            Case::B(ref mut b) => b.poll(),
        }
    }
}

// this fn replaces closures to avoid boxing in some cases
fn print_err<T:std::fmt::Debug>(t:T) {
    println!("{:?}", t);
}

struct EventService {
    tx_new: mpsc::Sender<mpsc::Sender<Result<Chunk,hyper::Error>>>,
    to_audio_iface: mpsc::Sender<vol_voleur_msg::VolVoleurUpdateMsg>,
}

impl Service for EventService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let (method, uri, _, _, body) = req.deconstruct();
        match (method, uri.path()) {
            (Get, "/events") => { println!("request events");
                let (tx_msg, rx_msg) = mpsc::channel(10);
                Box::new(self.tx_new.clone().send(tx_msg)
                    .map_err(|_| hyper::Error::Incomplete)// other errors types disallowed by hyper
                    .and_then(|_|{
                        Ok(Response::new()
                            .with_status(StatusCode::Ok)
                            .with_header(AccessControlAllowOrigin::Any)
                            .with_header(ContentType(mime::TEXT_EVENT_STREAM))
                            .with_header(Connection::keep_alive())   
                            .with_body(rx_msg))
                    }))
            },

            (Get, "/") => { println!("request html");
                
                let mut f = File::open("src/js/index.html").expect("file not found");
            
                let mut contents = String::new();
                f.read_to_string(&mut contents)
                    .expect("something went wrong reading the file");
                    
                Box::new(future_ok(Response::new()
                    .with_status(StatusCode::Ok)
                    .with_body(contents)))
            },
            
            (Post, "/setVol") => {
                Box::new(
                    body.concat2().map(|b| {
                    let params = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
                    println!("{:?}", params);
                    let vol: u32 = params.get("number").unwrap().parse().unwrap();
                    // TODO create struct, send it down channel
//                    let to_send = vol_voleur_msg::VolVoleurUpdateMsg{payload: Some(vec!(vol_voleur_msg::VolVoleurSinkDetails{volume: 33, name: String::from("h")}))};
//                    self.to_audio_iface.send(to_send).wait();
                    })
                    .and_then(|_| {
                        Ok(Response::new()
                            .with_status(StatusCode::Ok))}
                    )
                )
            },

            (method, path) => {
                println!("Alternative request method: {:?}, path: {:?}", method, path);
                // TODO change this
                let full_path = format!("{}{}", "src/js", path);

                if !Path::new(&full_path).exists()
                {
                    Box::new(future_ok(Response::new()
                        .with_status(StatusCode::NotFound)))
                }
                else
                {
                    let mut f = File::open(full_path).expect("file not found");
                
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)
                        .expect("something went wrong reading the file");
                        
                    Box::new(future_ok(Response::new()
                        .with_status(StatusCode::Ok)
                        .with_body(contents)))
                }
            }
        }
    }
}

fn main()
{
    println!("Hello ff!");

    let (sender, receiver): (Sender<vol_voleur_msg::VolVoleurUpdateMsg>, Receiver<vol_voleur_msg::VolVoleurUpdateMsg>) = mpsc::channel(1);
    
    thread::spawn(move || {
            pulse_iface::listen(sender);
        });

    let addr = "0.0.0.0:7878".parse().expect("addres parsing failed");

    let clients:Vec<mpsc::Sender<Result<Chunk, hyper::Error>>> = Vec::new();
    let (tx_new, rx_new) = mpsc::channel(10);
    
    let (audio_cmd_send, audio_cmd_recv): (Sender<vol_voleur_msg::VolVoleurUpdateMsg>, Receiver<vol_voleur_msg::VolVoleurUpdateMsg>) = mpsc::channel(1);

    thread::spawn(move || {
            pulse_iface::recv_commands(audio_cmd_recv);
    });

    let server = Http::new().bind(&addr, move || Ok(EventService{ tx_new: tx_new.clone(), to_audio_iface: audio_cmd_send.clone() })).expect("unable to create server");
    let handle = server.handle();

    let fu_rx_client = rx_new.into_future().map_err(print_err);
    let fu_rx_data = receiver.into_future().map_err(print_err);

    let broker = loop_fn((fu_rx_data, fu_rx_client, clients), move |(fu_rx_data, fu_rx_client, mut clients)|{
        fu_rx_data.select2(fu_rx_client)
            .map_err(|_| ())
            .and_then(move |done|
                match done {
                    Either::A(((item, rx_data_new), fu_rx_client)) => Case::A({//send messages
                        println!("received {:?}", item);
                        let mut buf = BytesMut::with_capacity(512).writer();
                        let msg_payload = item.unwrap().payload.unwrap();
                        // TODO: iterate over msg_payload vector
                        write!(buf, "event: vol_update\ndata: [").expect("msg write failed");
                        write!(buf, "{{\"name\": \"{}\", \"volume\": \"{}\"}}", msg_payload[0].name, msg_payload[0].volume).expect("msg write failed");
                        write!(buf, "]\n\n").expect("msg write failed");
                        
                        
                        println!("{:?}", buf);
                        let msg:Bytes = buf.into_inner().freeze();
                        let tx_iter = clients.into_iter()
                            .map(|tx| tx.send(Ok(Chunk::from(msg.clone()))));
                        futures::stream::futures_unordered(tx_iter)
                            .map(|x| Some(x))
                            .or_else(|e| { println!("{:?} client removed", e); Ok::<_,()>(None)})
                            .filter_map(|x| x)
                            .collect()
                            .and_then(move |clients|
                                future_ok(Loop::Continue((
                                    rx_data_new.into_future().map_err(print_err),
                                    fu_rx_client, 
                                    clients
                                )))                            
                            )

                    }),
                        
                    Either::B(((item, rx_new), fu_rx_data)) => Case::B({//register new client
                        match item {
                            Some(item) => {
                                clients.push(item); 
                                println!("client {} registered", clients.len());
                            },
                            None => println!("keeper loop get None"),
                        }

                        future_ok(Loop::Continue((
                            fu_rx_data,
                            rx_new.into_future().map_err(print_err), 
                            clients
                        )))
                    }),
                }
            )
    });


    handle.spawn(broker);

    println!("Listening on http://{} with 1 thread.", server.local_addr().expect("unable to get local address"));
    server.run().expect("unable to run server");
}
