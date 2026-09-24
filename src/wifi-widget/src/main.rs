mod app;
mod bar;
mod card;
mod join;
mod login;
mod preview;
mod row;
use std::{process::ExitCode, time::Instant};
use wifi_widget::{model::Store, probe, wifi};
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [arg, directory] if arg == "--render-preview" => {
            match preview::render(std::path::Path::new(directory)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        [arg, directory] if arg == "--render-chips" => {
            match preview::render_chips(std::path::Path::new(directory)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        [arg] if arg == "--dump" => {
            let mut store = Store::default();
            let reading = wifi::read();
            let associated = reading.as_ref().is_some_and(|r| r.associated && r.power);
            store.update_link(reading, Instant::now());
            if associated {
                store.update_probe(probe::run(), Instant::now());
            }
            let now = Instant::now();
            print!("{}", store.snapshot(now).dump(now));
            ExitCode::SUCCESS
        }
        [arg] if arg == "--ui-diagnostics" => {
            app::run(true);
            ExitCode::SUCCESS
        }
        [] => {
            app::run(false);
            ExitCode::SUCCESS
        }
        [arg] if arg == "--help" || arg == "-h" => {
            println!(
                "Usage: wifi-widget [--dump | --ui-diagnostics]\nRead Wi-Fi measurements and probe captive.apple.com. Does not change your connection."
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown arguments. Usage: wifi-widget [--dump | --ui-diagnostics]");
            ExitCode::from(2)
        }
    }
}
