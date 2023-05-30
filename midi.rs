use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub use launchpad_x::LaunchpadX;
pub use launch_control_xl::LaunchControlXL;

pub trait Device: Default + Send + 'static {
    type Input: Send;
    type Output: Send;

    fn process_input(&mut self, data: &[u8]) -> Option<<Self as Device>::Input>;
    fn process_output(&mut self, output: <Self as Device>::Output) -> Vec<Vec<u8>>;

    fn setup(midi: &mut Midi<Self>) {}
}


pub struct Midi<D: Device> {
    inner: Arc<Mutex<MidiInner<D>>>,
    in_conn: MidiInputConnection<()>,
    out_conn: MidiOutputConnection,
}

struct MidiInner<D: Device> {
    device: D,
    inputs: Vec<D::Input>,
    outputs: Vec<D::Output>,
}

impl<D: Device> Midi<D> {
    pub fn list() -> Result<()> {
        let midi_in = MidiInput::new("synth")?;
        for port in midi_in.ports() {
            println!("IN: '{:?}'", midi_in.port_name(&port).unwrap());
        }

        let midi_out = MidiOutput::new("synth")?;
        for port in midi_out.ports() {
            println!("OUT: '{:?}'", midi_out.port_name(&port).unwrap());
        }

        Ok(())
    }

    pub fn open(name: &str) -> Result<Self> {
        let inner = Arc::new(Mutex::<MidiInner<D>>::new(MidiInner {
            device: D::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }));

        let midi_in = MidiInput::new(&format!("synth_in_{}", name))?;
        let midi_out = MidiOutput::new(&format!("synth_out_{}", name))?;

        let in_port = midi_in
            .ports().into_iter()
            .find(|p| midi_in.port_name(p).unwrap().starts_with(name))
            .ok_or_else(|| anyhow!("input not found: {name}"))?;
        // let in_name = midi_in.port_name(&in_port).unwrap();

        let out_port = midi_out
            .ports().into_iter()
            .find(|p| midi_out.port_name(p).unwrap().starts_with(name))
            .ok_or_else(|| anyhow!("output not found: {name}"))?;
        // let out_name = midi_out.port_name(&out_port).unwrap();

        let out_conn = midi_out
            .connect(&out_port, "out")
            .map_err(|_| anyhow!("failed to create output port"))?;

        let _inner = Arc::clone(&inner);
        let in_conn = midi_in
            .connect(
                &in_port,
                "in",
                move |_, data, _| {
                    let mut inner = _inner.lock().unwrap();
                    if let Some(input) = inner.device.process_input(data) {
                        inner.inputs.push(input);
                    }
                },
                (),
            )
            .map_err(|_| anyhow!("failed to create input port"))?;

        let mut this = Self {
            inner,
            in_conn,
            out_conn,
        };
        D::setup(&mut this);

        Ok(this)
    }

    pub fn send(&mut self, output: D::Output) {
        let mut inner = self.inner.lock().unwrap();
        let data = inner.device.process_output(output);
        for frame in data {
            self.out_conn.send(&frame).unwrap();
        }
    }

    pub fn send_raw(&mut self, data: &[u8]) {
        self.out_conn.send(data).unwrap();
    }

    pub fn recv(&mut self) -> impl Iterator<Item = D::Input> + '_ {
        self.inner.lock().unwrap()
            .inputs.drain(..)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

mod launchpad_x {
    use types::launchpad_x::*;

    #[derive(Default)]
    pub struct LaunchpadX;

    fn float(v: u8) -> f64 {
        (v as f64) / 127.0
    }
    fn byte(f: f64) -> u8 {
        (f.clamp(0.0, 1.0) * 127.0) as u8
    }

    fn pos_to_byte(x: i8, y: i8) -> u8 {
        let x = std::cmp::min(x, 8);
        let y = std::cmp::min(x, 8);
        ((y + 1) * 10 + (x + 1)) as u8
    }
    fn byte_to_pos(b: u8) -> (i8, i8) {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        (x as i8, y as i8)
    }

    fn index_to_byte(index: u8) -> u8 {
        let i = std::cmp::min(index, 80);
        let x = i % 9;
        let y = i / 9;
        (y + 1) * 10 + (x + 1)
    }
    fn byte_to_index(b: u8) -> u8 {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        y * 9 + x
    }

    impl super::Device for LaunchpadX {
        type Input = Input;
        type Output = Output;

        fn process_input(&mut self, raw: &[u8]) -> Option<Input> {
            Some(match raw[0] {
                0x90 => (byte_to_pos(raw[1]), float(raw[2])),
                0xA0 => (byte_to_pos(raw[1]), float(raw[2])),
                0xB0 => (byte_to_pos(raw[1]), float(raw[2])),
                _ => return None
            })
        }

        fn process_output(&mut self, output: Output) -> Vec<Vec<u8>> {
            let mut data = Vec::with_capacity(8 + (81 * 4));
            data.extend_from_slice(&[0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x3]);

            for (i, color) in output.into_iter().enumerate() {
                let (r, g, b) = (byte(color.0), byte(color.1), byte(color.2));
                data.extend_from_slice(&[0x3, index_to_byte(i as u8), r, g, b]);
            }

            data.push(0xF7);
            vec![data]
        }

        fn setup(midi: &mut super::Midi<Self>) {
            const MODE: Mode = Mode::Programmer;
            const VELOCITY: Velocity = Velocity::Medium;
            const PRESSURE: Pressure = Pressure::Polyphonic;
            const CURVE: PressureCurve = PressureCurve::Medium;
            const BRIGHTNESS: f64 = 1.0;

            let mode = match MODE {
                Mode::Live => 0,
                Mode::Programmer => 1,
            };
            midi.send_raw(&[0xF0, 0x00, 0x20, 0x29, 0x2, 0x0C, 0x0E, mode, 0xF7]);


            let curve = match VELOCITY {
                Velocity::Low => 0,
                Velocity::Medium => 1,
                Velocity::High => 2,
                Velocity::Fixed(_) => 3,
            };
            let fixed = match VELOCITY {
                Velocity::Fixed(v) => v,
                _ => 0x00
            };
            midi.send_raw(&[0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0x04, curve, fixed, 0xF7]);

            let pressure = match PRESSURE {
                Pressure::Polyphonic => 0,
                Pressure::Channel => 1,
                Pressure::Off => 2,
            };
            let curve = match CURVE {
                PressureCurve::Low => 0,
                PressureCurve::Medium => 1,
                PressureCurve::High => 2,
            };
            midi.send_raw(&[0xF0, 0x0, 0x20, 0x29, 0x2, 0xC, 0xB, pressure, curve, 0xF7]);

            let brightness = byte(BRIGHTNESS);
            midi.send_raw(&[0xF0, 0x00, 0x20, 0x29, 0x2, 0xC, 0x8, brightness, 0xF7]);
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Mode {
        Live,
        Programmer
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Velocity {
        Low,
        Medium,
        High,
        Fixed(u8)
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Pressure {
        Polyphonic,
        Channel,
        Off
    }

    #[derive(Copy, Clone, Debug)]
    pub enum PressureCurve {
        Low,
        Medium,
        High,
    }
}

mod launch_control_xl {
    use types::launch_control_xl::*;

    #[derive(Default)]
    pub struct LaunchControlXL;

    fn float(v: u8) -> f64 {
        (v as f64) / 127.0
    }
    fn byte(f: f64) -> u8 {
        (f.clamp(0.0, 1.0) * 127.0) as u8
    }

    fn float_diverging(v: u8) -> f64 {
        if v >= 0x40 {
            ((v - 0x40) as f64) / 63.0
        } else {
            -1.0 + ((v as f64) / 64.0)
        }
    }

    fn light_sysex(idx: u8, color: Color) -> Vec<u8> {
        vec![
            0xf0, 0x00, 0x20, 0x29, 0x02, 0x11, 0x78,
            0x0,
            idx,
            color.mask(),
            0xf7
        ]
    }

    impl super::Device for LaunchControlXL {
        type Input = Input;
        type Output = Output;

        fn process_input(&mut self, raw: &[u8]) -> Option<Input> {
            Some(match raw[0] & 0xf0 {
                0x90 => {
                    match raw[1] {
                        0x29..=0x2c => Input::Button(raw[1] as i8 - 0x29, 1, true),
                        0x39..=0x3c => Input::Button(4 + raw[1] as i8 - 0x39, 1, true),
                        0x49..=0x4c => Input::Button(raw[1] as i8 - 0x49, 0, true),
                        0x59..=0x5c => Input::Button(4 + raw[1] as i8 - 0x59, 0, true),
                        0x69..=0x6c => Input::Select(3 - (raw[1] as i8 - 0x69), true),
                        _ => return None,
                    }
                },
                0x80 => {
                    match raw[1] {
                        0x29..=0x2c => Input::Button(raw[1] as i8 - 0x29, 1, false),
                        0x39..=0x3c => Input::Button(4 + raw[1] as i8 - 0x39, 1, false),
                        0x49..=0x4c => Input::Button(raw[1] as i8 - 0x49, 0, false),
                        0x59..=0x5c => Input::Button(4 + raw[1] as i8 - 0x59, 0, false),
                        0x69..=0x6c => Input::Select(3 - (raw[1] as i8 - 0x69), false),
                        _ => return None,
                    }
                }
                0xb0 => {
                    match raw[1] {
                        0x0d..=0x14 => Input::Knob(2, raw[1] as i8 - 0x0d, float_diverging(raw[2])),
                        0x1d..=0x24 => Input::Knob(1, raw[1] as i8 - 0x1d, float_diverging(raw[2])),
                        0x31..=0x38 => Input::Knob(0, raw[1] as i8 - 0x31, float_diverging(raw[2])),
                        0x4d..=0x54 => Input::Slider(raw[1] as i8 - 0x4d, float(raw[2])),
                        0x68 => Input::Up(raw[2] == 0x7f),
                        0x69 => Input::Down(raw[2] == 0x7f),
                        0x6a => Input::Left(raw[2] == 0x7f),
                        0x6b => Input::Right(raw[2] == 0x7f),
                        _ => return None,
                    }
                },
                _ => return None,
            })
        }

        fn process_output(&mut self, output: Output) -> Vec<Vec<u8>> {
            let mut cmds = vec![];

            for (i, color) in output.knobs.into_iter().enumerate() {
                // reverse the rows, indexes go top to bottom by default
                let j = match i {
                    0..=7 => i + 16,
                    8..=15 => i,
                    _ => i - 16,
                };
                cmds.push(light_sysex(j as u8, color));
            }

            for (i, color) in output.buttons.into_iter().enumerate() {
                // reverse the rows, indexes go top to bottom by default
                let j = match i {
                    0..=7 => i + 8,
                    _ => i - 8,
                };
                cmds.push(light_sysex(0x18 + j as u8, color));
            }

            cmds.push(light_sysex(0x2c, output.up));
            cmds.push(light_sysex(0x2d, output.down));
            cmds.push(light_sysex(0x2e, output.left));
            cmds.push(light_sysex(0x2f, output.right));

            for (i, b) in output.select.into_iter().enumerate() {
                cmds.push(vec![
                    0xf0, 0x00, 0x20, 0x29, 0x02, 0x11, 0x78,
                    0x0,
                    0x28 + i as u8,
                    if b { 63 } else { 0 },
                    0xf7
                ]);
            }
            cmds
        }
    }
}
