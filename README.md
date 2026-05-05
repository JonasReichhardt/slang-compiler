# slang
machine-code compiler for a simple C-like programming language

## Example program

```c
var i: int;

fn putInt(x: int): void { /* largest printable number = 9999 */
    var c0: char;
    var c1: char;
    var c2: char;
    var c3: char;

    c3 = CHR(48 + x % 10); x = x / 10;
    c2 = CHR(48 + x % 10); x = x / 10;
    c1 = CHR(48 + x % 10); x = x / 10;
    c0 = CHR(48 + x % 10);

    if (c0 > '0') { put(c0); put(c1); put(c2); }
    elseif (c1 > '0') { put(c1); put(c2); }
    elseif (c2 > '0') { put(c2); }
    put(c3);
}

fn main(): void { /* print odd numbers */
    i = 1;
    while (i < 100) {
        putInt(i);
        putLn();
        i = i + 2;
    }
}
```

Built-in functions:

| Name    | Parameters | Return value | Description                                       |
| ------- | ---------- | ------------ | ------------------------------------------------- |
| putLn() | -          | void         | starts a new line                                 |
| put()   | char       | void         | prints the expression of type char to the console |
| ORD()   | char       | int          | converts the character to an integer              |
| CHR()   | int        | char         | converts the integer to a character               |



## ATG

```
COMPILER SimpleLang

CHARACTERS
  tab           = '\u0009'. /*  9 = tabulator */
  lf            = '\u000a'. /* 10 = line feed */
  cr            = '\u000d'. /* 13 = carriage return */

  zero          = '0'.
  nonZeroDigit  = "123456789".
  digit         = '0' + nonZeroDigit .
  letter        = 'A' .. 'Z' + 'a' .. 'z' + '_' + '$'.

  char          = ANY - "'" - '\\' - cr - lf.

TOKENS
  ident         = letter { letter | digit }.
  number = (  zero | nonZeroDigit { digit } ).
  charCon =
    "'" ( char
        | "\\" ("b" | "t" | "n" | "f" | "r" | "\"" | "\'" | "\\")
        )
    "'".

COMMENTS FROM "/*" TO "*/"
COMMENTS FROM "//" TO lf

IGNORE lf + cr + tab

/*---------------------------------------------------------------------------*/

PRODUCTIONS


SimpleLang = Declaration {Declaration}.
Declaration = VarDecl | FnDecl.
VarDecl = "var" ident ":" Type ";".
Type = ident.
FnDecl = "fn" ident Parameters "{" {VarDecl} StatSeq "}".
Parameters = "(" [Param {"," Param}] ")" [":" Type].
Param = ident ":" Type.
StatSeq = Statement {Statement}.
Statement =
      ident ( "=" Expression | ActParameters ) ";"
    | "if" "(" Condition ")" "{" StatSeq "}" {"elseif" "(" Condition ")" "{" StatSeq "}" } ["else" "{" StatSeq "}"]
    | "while" "(" Condition ")" "{" StatSeq "}"
    | "return" [Expression] ";"
.
Condition = Expression Relop Expression.
Expression = [Addop] Term {Addop Term}.
Term = Factor {Mulop Factor}.
Factor = ident [ActParameters] | number | charCon | "(" Expression ")".
ActParameters = "(" [Expression {"," Expression}] ")".
Relop = "=" | "#" | "<" | ">" | ">=" | "<=".
Addop = "+" | "-".
Mulop = "*" | "/" | "%".


END SimpleLang.
```
