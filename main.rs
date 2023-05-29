#![allow(unused)]

use std::error::Error;
use std::f32::consts::PI;
use std::mem;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use inotify::{Inotify, WatchMask};
use libloading::Library;
use libloading::os::unix::Symbol;
use midi::{Midi, LaunchpadX, LaunchControlXL};
use spin::Mutex;

mod midi;

fn main() -> Result<()> {
    let mut ctrl: Midi<LaunchControlXL> = Midi::open("Launch Control XL:Launch Control XL")?;
    let mut pad: Midi<LaunchpadX> = Midi::open("Launchpad X:Launchpad X LPX MIDI")?;

    let mut inotify = Inotify::init()?;
    inotify.add_watch(std::env::current_dir()?.join("target/release"), WatchMask::CREATE)?;

    let lib = Lib::load()?;
    let this = (lib.default)();
    let program = Arc::new(Mutex::new(Program {
        this,
        lib,
        t: 0.0,
    }));
    let _program = Arc::clone(&program);


    let (jack, _) = jack::Client::new("synth", jack::ClientOptions::NO_START_SERVER)?;
    let mut out_left = jack.register_port("out_left", jack::AudioOut)?;
    let mut out_right = jack.register_port("out_right", jack::AudioOut)?;

    let process = jack::ClosureProcessHandler::new(move |client, ps| -> jack::Control {
        const RATE: f64 = 48_000.0; // TODO: make dynamic

        let mut p = _program.lock();

        let mut out_left = out_left.as_mut_slice(ps);
        let mut out_right = out_right.as_mut_slice(ps);

        for frame in out_left.iter_mut().zip(out_right.iter_mut()) {
            p.t += 1.0 / RATE;
            let (l, r) = (p.lib.sample)(p.this, p.t);
            *frame.0 = l as f32;
            *frame.1 = r as f32;
        }

        jack::Control::Continue
    });
    let _process = jack.activate_async(Notifications, process);

    let (jack, _) = jack::Client::new("synth", jack::ClientOptions::NO_START_SERVER)?;
    jack.connect_ports_by_name("synth:out_left", "Scarlett 2i4 Analog Surround 4.0:playback_FL")?;
    jack.connect_ports_by_name("synth:out_right", "Scarlett 2i4 Analog Surround 4.0:playback_FR")?;

    loop {
        let mut buf = [0; 256];
        match inotify.read_events(&mut buf) {
            Ok(events) => for event in events {
                if event.name.unwrap().to_str().unwrap() == "libprogram.so" {
                    let mut p = program.lock();

                    // serialize and drop `this`
                    let serialized = (p.lib.serialize)(p.this);
                    (p.lib.drop)(p.this);

                    // reload .so, carefully...
                    let lib = mem::replace(&mut p.lib, unsafe { mem::zeroed() });
                    drop(lib);
                    let lib = Lib::load()?;
                    let zeroed = mem::replace(&mut p.lib, lib);
                    mem::forget(zeroed);

                    // deserialize `this`
                    p.this = match (p.lib.deserialize)(&serialized) {
                        Ok(this) => this,
                        Err(e) => {
                            println!("failed to deserialize state: {e}");
                            (p.lib.default)()
                        },
                    };

                    println!("reloaded libprogram.so");
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
            Err(e) => anyhow::bail!(e),
        };

        {
            for input in pad.recv() {
                let p = program.lock();
                (p.lib.pad_in)(p.this, p.t, input);
            }

            let p = program.lock();
            let output = (p.lib.pad_out)(p.this, p.t);
            pad.send(output);
        }

        {
            use types::launch_control_xl::*;

            for input in ctrl.recv() {
                let p = program.lock();
                (p.lib.ctrl_in)(p.this, p.t, input);
            }

            let p = program.lock();
            let output = (p.lib.ctrl_out)(p.this, p.t);
            ctrl.send(output);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

struct Program {
    this: *mut (),
    lib: Lib,
    t: f64,
}

unsafe impl Send for Program {}

#[allow(clippy::type_complexity)]
struct Lib {
    lib: Library,
    sample: Symbol<fn(*mut (), f64) -> (f64, f64)>,
    pad_in: Symbol<fn(*mut (), f64, types::launchpad_x::Input)>,
    pad_out: Symbol<fn(*mut (), f64) -> types::launchpad_x::Output>,
    ctrl_in: Symbol<fn(*mut (), f64, types::launch_control_xl::Input)>,
    ctrl_out: Symbol<fn(*mut (), f64) -> types::launch_control_xl::Output>,
    default: Symbol<fn() -> *mut ()>,
    serialize: Symbol<fn(*mut ()) -> Vec<u8>>,
    deserialize: Symbol<fn(&[u8]) -> std::result::Result<*mut (), Box<dyn Error>>>,
    drop: Symbol<fn(*mut ())>,
}

impl Lib {
    pub fn load() -> Result<Self> {
        unsafe {
            let lib = Library::new("target/release/libprogram.so")?;
            Ok(Self {
                sample: lib.get::<fn(*mut (), f64) -> (f64, f64)>(b"sample\0")?.into_raw(),
                pad_in: lib.get::<fn(*mut (), f64, types::launchpad_x::Input)>(b"pad_in\0")?.into_raw(),
                pad_out: lib.get::<fn(*mut (), f64) -> types::launchpad_x::Output>(b"pad_out\0")?.into_raw(),
                ctrl_in: lib.get::<fn(*mut (), f64, types::launch_control_xl::Input)>(b"ctrl_in\0")?.into_raw(),
                ctrl_out: lib.get::<fn(*mut (), f64) -> types::launch_control_xl::Output>(b"ctrl_out\0")?.into_raw(),
                default: lib.get::<fn() -> *mut ()>(b"default\0")?.into_raw(),
                serialize: lib.get::<fn(*mut ()) -> Vec<u8>>(b"serialize\0")?.into_raw(),
                deserialize: lib.get::<fn(&[u8]) -> std::result::Result<*mut (), Box<dyn Error>>>(b"deserialize\0")?.into_raw(),
                drop: lib.get::<fn(*mut ())>(b"drop\0")?.into_raw(),

                lib
            })
        }
    }
}

struct Notifications;
impl jack::NotificationHandler for Notifications {
    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {}

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: rate={}", srate);
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun");
        jack::Control::Continue
    }
}
