use drfiley;
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = Path::new(&args[1]);

    if let Err(_) = drfiley::stat_all(&path) {
        eprintln!("Error running drfiley.")
    }
}
