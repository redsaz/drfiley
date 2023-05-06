use drfiley;

mod configuration;

fn main() {
    let config = configuration::config().expect("DrFiley Collector must be configured.");

    if let Err(_) = drfiley::stat_all(&config.path) {
        eprintln!("Error running drfiley.")
    }
}
