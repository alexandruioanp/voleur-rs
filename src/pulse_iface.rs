extern crate libpulse_binding as pulse;

use self::pulse::mainloop::standard::Mainloop;
use self::pulse::mainloop::standard::InterateResult;
use self::pulse::context::subscribe::subscription_masks;
use self::pulse::proplist::*;

use self::pulse::context::ContextInternal;
use self::pulse::context::subscribe::*;
use self::pulse::context::introspect::*;

use std::os::raw::c_void;
use std::time::Duration;
use std::thread::sleep;

//use std::sync::mpsc;

extern crate futures;
use futures::sync::mpsc;
//use futures::sync::mpsc::{Sender, Receiver};
use futures::sink::*;
use futures::{Future, Stream, Async};

use vol_voleur_msg::*;

struct Hack
{
    ch: mpsc::Sender<VolVoleurUpdateMsg>,
}

pub fn recv_commands(mut rx: mpsc::Receiver<VolVoleurUpdateMsg>) {
    println!("recv");
    
//    let rx = receiver.map(|x| println!("{:?}", x));
    // TODO act on received struct
    while let Ok(Async::Ready(Some(v))) = rx.poll()
    {
        println!("stream: {:?}", v);
    }
}

pub fn listen(send: mpsc::Sender<VolVoleurUpdateMsg>) {
    let mut proplist = pulse::proplist::Proplist::new().unwrap();
    proplist.sets(pulse::proplist::properties::APPLICATION_NAME, "VolVoleur")
        .unwrap();

    let mainloop = Mainloop::new().unwrap();

    let context = pulse::context::Context::new_with_proplist(
        mainloop.get_api(),
        "VolVoleurContext",
        &mut proplist
        ).unwrap();

    context.connect(None, pulse::context::flags::NOFLAGS, None).unwrap();
    
    // Wait for context to be ready
    loop {
        match mainloop.iterate(false) {
            InterateResult::Quit(_) |
            InterateResult::Err(_) => {
                eprintln!("iterate state was not success, quitting...");
                return;
            },
            InterateResult::Success(_) => {},
        }
        match context.get_state() {
            pulse::context::State::Ready => { break; },
            pulse::context::State::Failed |
            pulse::context::State::Terminated => {
                eprintln!("context state failed/terminated, quitting...");
                return;
            },
            _ => {},
        }
    }
    
    let interest =  subscription_masks::SINK_INPUT;

    let mut h = Hack { ch: send };
    
//    let chan_ptr: *mut c_void = &h as *mut _ as *mut c_void;
    
    let chan_ptr: *mut c_void = &mut h as *mut _ as *mut c_void;
    
	context.set_subscribe_callback(Some((my_subscription_callback, chan_ptr)));
//	context.set_subscribe_callback(Some((my_subscription_callback, ::std::ptr::null_mut())));
    let op = context.subscribe(interest, (x, ::std::ptr::null_mut()));
    
    match op
    {
    	Some(_) => println!("subscribe ok"),
    	None 	=> {
    		eprintln!("error subscribing, quitting...");
            return;
    	}
    } 
    
    // Just loop
    loop {
        match mainloop.iterate(false) {
            InterateResult::Quit(_) |
            InterateResult::Err(_) => {
                eprintln!("iterate state was not success, quitting...");
                return;
            },
            InterateResult::Success(_) => {},
        }
        match context.get_state() {
            pulse::context::State::Ready => {},
            pulse::context::State::Failed |
            pulse::context::State::Terminated => {
                eprintln!("context state failed/terminated, quitting...");
                break;
            },
            _ => {},
        }
        sleep(Duration::from_millis(10));
    }
    
    // Clean shutdown
    mainloop.quit(0); // uncertain whether this is necessary
}

extern "C" fn x(_: *mut ContextInternal, _: i32, _: *mut c_void)
{
	//DONOTINHG
}

extern "C"
fn my_subscription_callback(
    ct: *mut ContextInternal, // Ignoring context pointer
    _: EventType,            // The combined facility and operation
    sink_index: u32,                  // Ignoring index
    data: *mut c_void)          // Ignoring userdata pointer
{
//    println!("got something!");
//    println!("Event type {:?} sink_index {:?}", t, sink_index);
    
    let context = pulse::context::Context::from_raw_weak(ct);
    let intr: Introspector = context.introspect();
//    println!("Requesting sink input info");
//    intr.get_sink_input_info(sink_index, (sink_input_cb, ::std::ptr::null_mut()));
    intr.get_sink_input_info(sink_index, (sink_input_cb, data));

//    intr = pulse::context::introspect::Introspector;
//    intr.get_sink_input_info(sink_index, sink_input_cb);
}

extern "C"
fn sink_input_cb(_: *mut ContextInternal, info_int: *const SinkInputInfoInternal, _: i32, data: *mut c_void)
{
//    println!("Got sink input info");
//    let please: *mut SinkInputInfoInternal = unsafe { std::mem::transmute(info_int) };
//      let info = unsafe { SinkInputInfo::from((*please)) };
//    let vv: SinkInputInfo = SinkInputInfo::from(info_int); 

    let chan: &mut Hack = unsafe { &mut *(data as *mut Hack) };
    let send_chan = &chan.ch;
    
    if !info_int.is_null()
    {
        let v: &SinkInputInfo = unsafe { ::std::mem::transmute(info_int) };
//        let si_name = unsafe { ::std::ffi::CStr::from_ptr((*v).name) };
//        let si_name = unsafe { ::std::ffi::CStr::from_ptr((*v).name) };
        let proplist: &Proplist =  &Proplist::from_raw_weak((*v).proplist);
//        println!("{:?}", (*proplist).to_string());
        let si_name = (*proplist).gets(properties::APPLICATION_NAME).unwrap();
        let vol: pulse::volume::CVolume = (*v).volume;
        let vol_rel: u32 = 100 * vol.values[0] / pulse::volume::VOLUME_NORM;
//        println!("{:?}", &(*si_name)[..si_name.len()-1]);
//        println!("lib: Vol {:?} Name {:?}", vol_rel, si_name.to_vec());
        let to_send = VolVoleurUpdateMsg{payload: Some(vec!(VolVoleurSinkDetails{volume: vol_rel, name: si_name}))};
        send_chan.clone().send(to_send).wait().unwrap();
    } 
}
