package main

import "fmt"

// Top-level function declaration
func greet() {
    fmt.Println("Hello from top-level function!")
}

// A struct with a method named in the same way as the top-level function
type Greeter struct {
    name string
}

// Method in the struct
func (g Greeter) greet() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}

func (g *Greeter) greetPointer() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}

func main() {
    // Call the top-level function
    greet()

    // Create instances of the generic struct and call its method
    greeter := Greeter{name: "Bob"}
    greeter.greet()
}