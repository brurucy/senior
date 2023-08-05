## Senior

Instead of bothering **a** senior engineer for suggestions on how to improve your code, **ask senior instead**!

### Intro

Senior uses LLMs(at the moment only openAI ones however) and an advance source code parsing library, `tree-sitter`, to,
from the comfort of your terminal, suggest improvements to your code.

As an argument to the cli, you can give:

1. A path to a file
2. A path to a file alongside a function name
3. A path to a file with a class/struct/parent element and a function name

In return, you will be prompted with an improved version. If you accept it, by pressing y, then the original file will
be overwritten with the suggestion.

The only requirement is that you must have an environment variable named `OPENAI_API_KEY` with your very own token.

### Language support

| Language   | Whole File | Function | Method | Limitations                                                                                                                             |
|------------|------------|----------|--------|-----------------------------------------------------------------------------------------------------------------------------------------|
| Rust       | X          | X        | X      |                                                                                                                                         |
| Go         | X          | X        | X      | Untested with Generics. Most likely works.                                                                                              |
| Javascript | X          | X        | X      | Does not work with functions inside object literals<br/>nor anonymous functions declared inside classes (you shouldn't do that anyways) |
| Typescript | X          | X        | X      | Does not work with functions inside object literals<br/>nor anonymous functions declared inside classes (you shouldn't do that anyways) |