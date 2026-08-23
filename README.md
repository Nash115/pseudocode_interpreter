# Pseudocode Interpreter

Have you ever wanted to execute pseudocode?
This project, built in Rust, allows you to execute pseudocode (**English**) and **French pseudo-code**.
However, it requires strict syntax and you should respect some rules explained in the Documentation.

## Use the project

Clone the repo

```bash
git clone https://github.com/Nash115/pseudocode_interpreter.git
cd pseudocode_interpreter
```

Build the project

```bash
cargo build --release
```

The binary is located in `./target/release/pseudocode_interpreter`
or (`./target/release/pseudocode_interpreter.exe` on Windows)

Execute a file containing the code :

```bash
./target/release/pseudocode_interpreter my_code.pseudocode
```

Execute instruction by instruction (REPL) :
(type `exit` to quit)

```bash
./target/release/pseudocode_interpreter 
```

## Code examples

Pseudocode (English) :
```
function add(a,b)
  return a + b
endFunction

let result = add(1,2)
print(result)
```

The same code in pseudo-code (French) :
```
fonction ajouter(a,b)
  retourner a + b
finFonction

variable resultat = ajouter(1,2)
affiche(resultat)
```

## Work In Progress

- Lists
- For Loops
- Better errors (including lines...)
- Better environment (Avoid Scope Accumulation : Memory Leak)

## Documentation

### Keywords

- `var`,`let`,`variable` : Declares (explicitly) a variable. A variable does not require an explicit declaration : it is automatically declared on the first assignment if not declared previously.
```
let a = 2
b = 3
```
- `const`,`constante` : Declares a constant : a variable that cannot be edited
```
const pi = 3.14
```
- `fn`,`function`,`fonction`,`procedure` (and `endFn`,`finFn`,`endFunction`,`finFonction`,`endProcedure`,`finProcedure`): Used to define a function
```
fn add(a,b)
  a + b
endFn
```
- `return`,`retourner`,`renvoyer` : Return a value from a function
```
fn add(a,b)
  return a + b
endFn
```
- `if`,`then`,`else`,`si`,`alors`,`sinon` (and `endIf`, `finSi`) : Used to check conditions
```
if 1+1 == 2 then
  print("1+1 = 2")
endIf

if 1 is 2 then
  print("Math error")
else if false and (true or false) then
  print("Logical error")
else if false then
  print("Another error")
else
  print("Ok!")
endIf
```
- `while`,`then`,`tantQue`,`alors` (and `endWhile`, `finTantQue`) : Used to loop on a condition
```
i = 0
while i < 3 then
  i = i + 1
  print(i)
endWhile
```

### Comments

You can write comments using `#` or `//`

Examples :

```
# My super program
print(42) // Prints 42
```

### Built-in constants and functions

- `PI` : CONST with an f64 value of π
- `print(...)`, `affiche(...)` : FUNCTION that prints every parameter, separated by spaces.
- `time()`, `temps()` : FUNCTION that returns the time since the EPOCH.

### Variable types

- Number : a float encoded in 64 bits (`f64` Rust type)
```
a = 42
b = 3.14
```
- Strings :
```
s1 = "Hello,"
s2 = ' World!'
print(s1 + s2)
```
- Booleans : `true` / `false` (`vrai` / `faux`)
```
b1 = true
b1 = vrai
b2 = false
b2 = faux
```
- Object (JSON-like)
```
user = {
  name: "Alice",
  age: 42,
  subscribed: true,
  social : {
    followers: 100,
    friends: 10
  }
}
```

### Logical expressions

Operators :
- `!`, `not`, `non` : Get the opposite of a boolean
- `and`, `et` : Evaluates to `true` (`vrai`) if the left AND the right values are (or evaluates to) `true`
- `or`, `ou` : Evaluates to `true` (`vrai`) if the left OR the right value is (or evaluate to) `true`

Expressions :
- `==`, `is`, `est` : Evaluates to `true` (`vrai`) if the left and the right values are equals
- `!=` : Evaluates to `true` (`vrai`) if the left and the right values are different
- `<` (and `<=`) : Evaluates to `true` (`vrai`) if the left value is lower (or equals, using `<=`) that the right value
- `>` (and `>=`) : Evaluates to `true` (`vrai`) if the left value is higher (or equals, using `>=`) that the right value

Example :
```
not false or (false and (3 > 1)) == true
```
Evaluates to `true`
