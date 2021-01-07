use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use std::fs::File;
use std::io::prelude::*;

use std::process::Command;

use daemonize::Daemonize;

fn main_loop() {

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();

    let mut file = File::open("/sys/class/backlight/intel_backlight/max_brightness").unwrap();
    let mut contents = String::new();
    let max_brightness;

    match file.read_to_string(&mut contents) {
        Ok(_) => {
            contents = contents.replace('\n', "");
            max_brightness = contents.parse::<f32>().unwrap();
            println!("Max brightness: {}", max_brightness);        
        },
        Err(e) => {
            panic!("{}", e);
        }
    }

    watcher
        .watch(
            "/sys/class/backlight/intel_backlight/brightness",
            RecursiveMode::Recursive,
        )
        .unwrap();

    set_brightness(max_brightness);

    loop {
        match rx.recv() {
            Ok(_) => {  
                set_brightness(max_brightness);
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn set_brightness(max_brightness: f32) {
    let mut file =
        File::open("/sys/class/backlight/intel_backlight/brightness").unwrap();
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {
            contents = contents.replace('\n', "");
            let brightness = contents.parse::<f32>().unwrap();
            let prct = brightness / max_brightness;
            println!("Set brightness: {}", prct);
            Command::new("redshift")
                .arg("-P")
                .arg("-O 6500")
                .arg(format!("-b {}:{}", prct, prct))
                .output()
                .unwrap();
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}

fn main() {

    let daemonize = Daemonize::new()
        .pid_file("/tmp/oled-redshift-daemon.pid");

    match daemonize.start() {
        Ok(_) => main_loop(),
        Err(e) => eprintln!("Error, {}", e),
    }
}
