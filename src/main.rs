mod adapters;
mod application;
mod core;
mod infrastructure;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    app.run()?;
    Ok(())
}
