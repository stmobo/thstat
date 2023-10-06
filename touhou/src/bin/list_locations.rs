use touhou::th07::Location as Th07Location;
use touhou::th08::Location as Th08Location;

fn main() {
    println!("PCB:");
    for (id, location) in Th07Location::iter_all().enumerate() {
        println!("{}: {}", id, location);
    }

    println!("\nIN:");

    for (id, location) in Th08Location::iter_all().enumerate() {
        println!("{}: {}", id, location);
    }
}
