use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    window_manager::run()
}
