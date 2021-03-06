#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

extern crate toml;
extern crate ddc_hi;

use std::{thread, time, io, process, collections::HashMap};
use ddc_hi::Ddc;
use std::io::Read;
use std::iter::Iterator;
use clap::{Arg, App};

#[derive(Deserialize)]
struct Config {
    displays: Vec<Display>
}

impl Config {
    fn get(&self, info: &ddc_hi::DisplayInfo) -> Option<&Display> {
        self.displays.iter().find(|d| d.matches(info))
    }

    fn parse(path: &str) -> Config {
        let mut data = String::new();
        let mut file = std::fs::File::open(path).expect("config file not found");
        file.read_to_string(&mut data).expect("failed to read configuration");
        toml::from_str(data.as_str()).expect("failed to parse configuration")
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
        self.actions.iter().find(|a| a.matches(value))
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

struct Poller {
    config: Config,
    displays: Vec<ddc_hi::Display>,
    state: HashMap<String, u16>
}

impl Poller {
    fn new(config: Config, displays: Vec<ddc_hi::Display>) -> Poller {
        Poller { config, displays, state: HashMap::new() }
    }

    fn poll(&mut self) {
        for mut display in &mut self.displays {
            if let Some(d) = self.config.get(&display.info) {
                match display.handle.get_vcp_feature(d.feature) {
                    Ok(value) => {
                        let current = value.value();
                        let old = *self.state.entry(d.serial.clone()).or_insert(current);
                        if current != old {
                            self.state.insert(d.serial.clone(), current);
                            d.get(current).and_then(|a| a.run().ok());
                        }
                    }
                    Err(e) => eprintln!("Error while fetching value: {}", e)
                }
            }
        }
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!(", "))
        .arg( Arg::with_name("config")
            .short("f")
            .value_name("FILE")
            .takes_value(true)
            .help("Path to the configuration file")).get_matches();

    let path = matches.value_of("config").unwrap_or("config.toml");
    let mut poller = Poller::new(Config::parse(path), ddc_hi::Display::enumerate());
    loop {
        poller.poll();
        thread::sleep(time::Duration::from_secs(5));
    }
}
