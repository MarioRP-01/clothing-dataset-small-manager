use std::process;

fn main() {
    let config = dataset_manager::Config::build().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = dataset_manager::run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {}
