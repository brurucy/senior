fn greet() {
    println!("Hello from top-level function!");
}

struct Greeter {
    name: String,
}

impl Greeter {
    fn greet(&self) {
        println!("Hello from {}, inside the Greeter struct!", self.name);
    }
}

struct GenericGreeter<T> {
    name: T,
}

impl<T: std::fmt::Display> GenericGreeter<T> {
    fn greet(&self) {
        println!("Hello from {}, inside the GenericGreeter struct!", self.name);
    }
}

fn main() {
    greet();

    let greeter = Greeter {
        name: String::from("Alice"),
    };
    greeter.greet();
}