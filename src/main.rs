use std::process::Command;
use std::time::Duration;
use std::thread;

#[macro_use]
extern crate chan;
extern crate chan_signal;

extern crate chrono;
extern crate systemstat;

use chan_signal::Signal;
use systemstat::{Platform, System};

fn plugged(sys: &System) -> String {
    if let Ok(plugged) = sys.on_ac_power() {
        if plugged {
            "AC: ✓".to_string()
        } else {
            "DC: ✘".to_string()
        }
    } else {
        "UN: ?".to_string()
    }
}

fn battery(sys: &System) -> String {
    if let Ok(bat) = sys.battery_life() {
        format!("BAT: {:.1}%", bat.remaining_capacity * 100.)
    } else {
        "".to_string()
    }
}

fn ram(sys: &System) -> String {
    if let Ok(mem) = sys.memory() {
        let used = mem.total - mem.free;
        format!("MEM: {}/{}", used, mem.total)
    } else {
        "▯ _".to_string()
    }
}

fn cpu(sys: &System) -> String {
    if let Ok(cpu) = sys.load_average() {
        format!("CPU: load: {:.2}",
                cpu.one)
    } else {
        "CPU: _".to_string()
    }
}

fn cputemp(sys: &System) -> String {
    if let Ok(cputemp) = sys.cpu_temp() {
        format!("TEMP: {} C", cputemp)
    } else {
        "TEMP: _".to_string()
    }
}

fn date() -> String {
    chrono::Local::now().format("📆 %a, %d %h ⸱ 🕓 %T").to_string()
}

fn separated(s: String) -> String {
    if s == "" { s } else { s + " ⸱ " }
}

fn status(sys: &System) -> String {
    separated(plugged(sys)) + &separated(battery(sys)) + &separated(ram(sys)) +
    &separated(cputemp(sys)) + &separated(cpu(sys)) + &date()
}

fn update_status(status: &String) {
    // Don't panic if we fail! We'll do better next time!
    let _ = Command::new("xsetroot").arg("-name").arg(status).output();
}

fn run(_sdone: chan::Sender<()>) {
    let sys = System::new();
    let mut banner = String::new();

    loop {
        let next_banner = status(&sys);
        if next_banner != banner {
            banner = next_banner;
            update_status(&banner);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn main() {
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);
    // Run work.
    std::thread::spawn(move || run(sdone));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            update_status(&format!("rust-dwm-status stopped with signal {:?}.", signal));
        },
        rdone.recv() => {
            update_status(&"rust-dwm-status: done.".to_string());
        }
    }
}
