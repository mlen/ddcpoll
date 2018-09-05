#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate ddc_hi;

use std::{thread, time, io, process, collections::HashMap};
use ddc_hi::Ddc;
use std::io::Read;
use std::iter::Iterator;

#[derive(Deserialize)]
struct Config {
    displays: Vec<Display>
}

impl Config {
    fn get(&self, info: &ddc_hi::DisplayInfo) -> Option<&Display> {
        self.displays.iter().filter(|d| d.matches(info)).next()
    }
}

#[derive(Deserialize)]
struct Display {
    serial: String,
    feature: u8,
    actions: Vec<Action>
}

impl Display {
    fn matches(&self, info: &ddc_hi::DisplayInfo) -> bool {
        self.serial == info.serial_number.clone().unwrap()
    }

    fn get(&self, value: u16) -> Option<&Action> {
        self.actions.iter().filter(|a| a.matches(value)).next()
    }
}

#[derive(Deserialize)]
struct Action {
    command: String,
    value: u16
}

impl Action {
    fn run(&self) -> io::Result<process::ExitStatus> {
        process::Command::new("sh").arg("-c").arg(self.command.clone()).status()
    }

    fn matches(&self, other: u16) -> bool {
        self.value == other
    }
}

fn poll(config: &Config, displays: &mut Vec<ddc_hi::Display>, prev_state: &mut HashMap<String, u16>) {
    for mut display in displays {
        if display.update_capabilities().is_ok() {
            if let Some(d) = config.get(&display.info) {
                if let Ok(value) = display.handle.get_vcp_feature(d.feature) {
                    let current = value.value();
                    let old = *prev_state.entry(d.serial.clone()).or_insert(current);
                    if current != old {
                        prev_state.insert(d.serial.clone(), current);
                        d.get(current).and_then(|a| a.run().ok());
                    }
                }
            }
        }
    }
}

fn main() {
    let mut data = String::new();
    let mut file = std::fs::File::open("config.toml").expect("config.toml not found");
    file.read_to_string(&mut data).expect("failed to read configuration");
    let config: Config = toml::from_str(data.as_str()).expect("failed to parse configuration");

    let mut prev_state = HashMap::new();
    let mut displays = ddc_hi::Display::enumerate();
    loop {
        poll(&config, &mut displays, &mut prev_state);
        thread::sleep(time::Duration::from_millis(1000));
    }
}
