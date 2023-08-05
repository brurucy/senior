// Function Declaration
function greet() {
    console.log("Hello from top-level function declaration!");
}

async function asyncGreet() {
    console.log("Hello from async top-level function declaration!");
}

// Function Expression
const greetExpression = function () {
    console.log("Hello from function expression!");
}

const asyncGreetExpression = async function () {
    console.log("Hello from async function expression!");
}

// Arrow Function
const greetArrow = () => {
    console.log("Hello from arrow function!");
}

const asyncGreetArrow = async () => {
    console.log("Hello from async arrow function!");
}

// Method Definition in a Class
class Greeter {
    greet() {
        console.log("Hello from method in a class!");
    }

    async asyncGreet() {
        console.log("Hello from async method in a class!");
    }
}

// Call all the functions
greet();
asyncGreet();
greetExpression();
asyncGreetExpression();
greetArrow();
asyncGreetArrow();

let greeter = new Greeter();
greeter.greet();
greeter.asyncGreet();