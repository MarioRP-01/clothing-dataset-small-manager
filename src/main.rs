use std::process;

fn main() {

    if let Err(e) = dataset_manager::run() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
