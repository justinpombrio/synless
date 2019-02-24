# Pretty-printing Laws

The notations defined in the `pretty` crate obey a variety of algebraic laws.
These laws are described below, using `+` for concatenation, `^` for vertical
concatenation, `|` for choie, and `⊥` for the impossible notation.

## Associativity

    a + (b + c) = (a + b) + c
    a | (b | c) = (a | b) | c
    a ^ (b ^ c) = (a ^ b) ^ c

## Distributivity

    (a | b) + c = (a + c) | (b + c)
    a + (b | c) = (a + b) | (a + c)
   
    (a | b) ^ c = a ^ c | b ^ c
    a ^ (b | c) = a ^ b | a ^ c

    no_wrap(a + b) = no_wrap(a) + no_wrap(b)
    no_wrap(a | b) = no_wrap(a) | no_wrap(b)

## Identity

    a + empty() = a
    empty() + a = a

    ⊥ | a = a
    a | ⊥ = a

## Idempotence

    no_wrap(no_wrap(a)) = no_wrap(a)
    a | a = a

## Annihilation

    ⊥ + a = ⊥
    a + ⊥ = ⊥
    ⊥ ^ a = ⊥
    a ^ ⊥ = ⊥
    no_wrap(⊥) = ⊥

## Absorbtion

    a | (a + b) = a
    (a + b) | a = a
    a | (a ^ b) = a
    (a ^ b) | a = a
    a | no_wrap(a) = a
    no_wrap(a) | a = a

## Misc

    a ^ (b + c) = (a ^ b) + c
    no_wrap(a ^ b) = ⊥
    no_wrap(literal(s)) = literal(s)
