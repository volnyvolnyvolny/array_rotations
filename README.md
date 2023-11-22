# Array rotations in Rust

This page citates [https://github.com/scandum/rotate](https://github.com/scandum/rotate).

## Quick Start

To build project with documentation:

```text
cargo build
cargo doc
```

To benchmark:

```text
cargo bench
```

Benchmarking could take some time :)

## Introduction

Rotating an array is replacing the left side of it with the right one:

```text
                  dim      mid
       left = 9   | 𝑏𝑟𝑖𝑑𝑔𝑒 |    right = 6
[ 1  2  3  4  5  6: 7  8  9* A  B  C  D  E  F]
        └─────────────────┘
              shadow
```

Result:

```text
[ A  B  C  D  E  F: 1  2  3* 4  5  6  7  8  9]
```

# Algorithms

## Special (edge) cases

### Left side == Right side

The fastest algorithm in this case is the one that swaps elements one by one:

```text
ptr::swap_nonoverlapping(start, mid, right);
```

If `left == 1 && right == 1` it's a two-element array and it is faster to use:

```text
ptr::swap(start, mid);
```

where `start`, `mid` are corresponding pointers.

See `ptr_edge_rotate`.

### The smallest side has `1` or `2` elements

In this case the fastest would be to copy smallest side to auxiliary memory and then
to shift left or right.

```text
               mid
    left = 5   | right = 1
[ 1: 2  3  4  5* a]
                 └──────┐
[           1-5  ✘]    [a]
  └──┬────────┴──┐
[ ✘: 1  2  3  4  5]    [a]
  ┌─────────────────────┘
[ a: 1  .  .  .* 5]
```

See `utils::shift_left` and `utils::shift_right` for the benchmarks prooven fastests
implementation.

## 💾 Auxiliary rotation

The easiest, but not always fastest way to rotate, is to copy a smaller half to an auxiliary
memory. Since it requires additional memory it is of little interest to in-place algorithms.
It's good for cases when the smallest part has size `1` or `2`.

### Examples

```text
                   dim     mid
       left = 9    |       |        right = 6
[ 1  2  3  4  5  6 :7  8  9*            10-15]                 move
                                         └──┴───────┬─────┐
[              1-6 :7  .  9  ✘  ✘  ✘  ✘  ✘  ✘]    [10 .. 15]   move
               └────┬─────┴─────────────────┐
[ ✘  ✘  ✘  ✘  ✘  ✘ :1 ~~~~~~~~~~~~~~~~~~~~~ 9]    [10-15   ]   move
  ┌──────────────┬──────────────────────────────────┴──┘
[10 ~~~~~~~~~~~ 15 :1  .  3* 4  .  .  .  .  9]
```

```text
                                 mid
          left = 11              | right = 4
[ 1  2  3  4: 5  6  7  8  9 10 11*      12-15]                 move
                                         └──┴───────┬─────┐
[ 1  .  .  .  .  .  .  .  .  . 11  ✘  ✘  ✘  ✘]    [12 .. 15]   move
  └───────────┬─────────────────┴───────────┐
[ ✘  ✘  ✘  ✘  1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11]    [12-15   ]   move
  ┌────────┬────────────────────────────────────────┴──┘
[12 ~~~~~ 15: 1  .  .  .  .  .  7* 8  .  . 11]
```

```text
            mid
   left = 4 |          right = 11
[      12-15* 1  2  3  4  5  6  7: 8  9 10 11]                 move
        └──┴────────────────────────────────────────┬─────┐
[ ✘  ✘  ✘  ✘  1  .  .  .  .  .  .  .  .  . 11]    [12 .. 15]   move
  ┌───────────┴─────────────────┬───────────┘
[ 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11  .  .  .  .]    [12-15   ]   move
                                   ┌────────┬───────┴──┘
[ 1  .  .  4* 5  .  .  .  .  . 11:12 ~~~~~ 15]
```

## 🌉 Bridge rotation

"This is a slightly more complex auxiliary rotation than
auxiliary rotation that reduces the maximum auxiliary memory
requirement from `50%` to `1/3`. If the overlap between the
two halves is smaller than the halves themselves it copies
the overlap to swap memory instead. Its first known publication
was in *2021* by *Igor van den Hoven*."[^1]

### Examples

Here `bridge` is less than `left` || `right`.
Otherwise, algorithm fallbacks to *auxiliary*.

Case `left > right`:

```text
                  dim      mid
         left = 9 | 𝑏𝑟𝑖𝑑𝑔𝑒   |   right = 6
[ 1  2  3  4  5  6: 7-9    *10 11 12 13 14 15]
                    └─┴────────────────────────────┬─────┐
  a-->              b-->     c-->                  |     |
[ 1  .  .  .  .  6: ✘  ✘  ✘*10  .  .  .  . 15]    [7  8  9]
  └─────────────────┐       |
  ╭──────────────── ┆───────╯
  ↓  a              ↓  b        c
[10  2  .  .  .  6  1  ✘  ✘  ✘ 11  .  .  . 15]    [7  .  9]
     └─────────────────┐       |
     ╭──────────────── ┆───────╯
     ↓  a              ↓  b        c
[10 11  3  .  .  6  1  2  ✘  ✘  ✘ 12  .  . 15]    [7  .  9]
        └─────────────────┐       |
        ╭──────────────── ┆───────╯
        ↓  a              ↓  b        c
[10  . 12  4  .  6  1  .  3  ✘  ✘  ✘ 13  . 15]    [7  .  9]
           └─────────────────┐       |
           ╭──────────────── ┆───────╯
           ↓  a              ↓  b        c
[10  .  . 13  5  6  1  .  .  4  ✘  ✘  ✘ 14 15]    [7  .  9]
              └─────────────────┐       |
              ╭──────────────── ┆───────╯
              ↓  a              ↓  b        c
[10  .  .  . 14  6  1  .  .  .  5  ✘  ✘  ✘ 15]    [7  .  9]
                 └─────────────────┐       |
                 ╭──────────────── ┆───────╯
                 ↓                 ↓  b
[10 ~~~~~~~~~~~ 15  1 ~~~~~~~~~~~~ 6  ✘  ✘  ✘]    [7-9    ]
                                      ┌─────┬──────┴─┘
[10  .  .  .  . 15: 1  .  3* 4  .  6  7 ~~~ 9]
```

Case `left < right`:

```text
                  mid      dim
      left = 6    | 𝑏𝑟𝑖𝑑𝑔𝑒   |  right = 9
[10 11 12 13 14 15*     1-3: 4  5  6  7  8  9]
                        └─┴────────────────────────┬─────┐
                    b        c              d      |     |
[10  .  .  .  . 15* ✘  ✘  ✘: 4  .  .  .  .  9]    [1  2  3]
                 ╰─────────────────────────╮|
                          ┌─────────────── ┆┘
                 b       c↓                ↓d
[10  .  .  . 14  ✘  ✘  ✘  9  4  .  .  .  8 15]    [1  .  3]
              ╰─────────────────────────╮|
                       ┌─────────────── ┆┘
              b       c↓                ↓d
[10  .  . 13  ✘  ✘  ✘  8  9  4  .  .  7 14 15]    [1  .  3]
           ╰─────────────────────────╮|
                    ┌─────────────── ┆┘
           b       c↓                ↓d
[10  . 12  ✘  ✘  ✘  7  .  9  4  .  6 13  . 15]    [1  .  3]
        ╰─────────────────────────╮|
                 ┌─────────────── ┆┘
        b       c↓                ↓d
[10 11  ✘  ✘  ✘  6  .  .  9  4  5 12  .  . 15]    [1  .  3]
     ╰─────────────────────────╮|
              ┌─────────────── ┆┘
     b       c↓                ↓d
[10  ✘  ✘  ✘  5  .  .  .  9  4 11  .  .  . 15]    [1  .  3]
  ╰─────────────────────────╮|
           ┌─────────────── ┆┘
          c↓               d↓
[ ✘  ✘  ✘  4 ~~~~~~~~~~~~ 9 10 ~~~~~~~~~~~ 15]    [1-3    ]
  ┌─────┬──────────────────────────────────────────┴─┘
[ 1 ~~~ 3  4  .  6* 7  .  9:10  .  .  .  . 15]
```

## 🤹 Juggling rotation

"Also known as the dolphin algorithm. This is a relatively complex
and inefficient way to rotate in-place, though it does so in the
minimal number of moves. Its first known publication was in *1966*.
It computes the greatest common divisor and uses a loop to create
a chain of consecutive swaps."[^1]

### Example

```text
                           mid
          left = 9         |    right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]
  |        |        |        |        └──────────────────╮
  |        |        |        └──────────────────╮        ┆
  |        |        └─────────────────┐         ┆        ┆
  |        └─────────────────┐        ┆         ┆        ┆
  └─────────────────┐        ┆        ┆         ┆        ┆
~──────────╮        ┆        ┆        ┆         ┆        ┆
~─╮        ┆        ┆        ┆        ┆         ┆        ┆
  ↓        ↓        ↓        ↓        ↓         ↓        ↓
[10  2  3 13  5  6  1  8  9  4 11 12  7 14 15][10  2  3 13...
     |        |        |        |        └──────────────────╮
     |        |        |        └──────────────────╮        ┆
     |        |        └─────────────────┐         ┆        ┆
     |        └─────────────────┐        ┆         ┆        ┆
     └─────────────────┐        ┆        ┆         ┆        ┆
~─────────────╮        ┆        ┆        ┆         ┆        ┆
~────╮        ┆        ┆        ┆        ┆         ┆        ┆
     ↓        ↓        ↓        ↓        ↓         ↓        ↓
[10 11  3 13 14  6  1  2  9  4  5 12  7  8 15][10 11  3 13 14...
        |        |        |        |        └──────────────────╮
        |        |        |        └──────────────────╮        ┆
        |        |        └─────────────────┐         ┆        ┆
        |        └─────────────────┐        ┆         ┆        ┆
        └─────────────────┐        ┆        ┆         ┆        ┆
~────────────────╮        ┆        ┆        ┆         ┆        ┆
~───────╮        ┆        ┆        ┆        ┆         ┆        ┆
        ↓        ↓        ↓        ↓        ↓         ↓        ↓
[10  . 12  .  . 15: .  .  3* .  .  6  .  .  9][ .  . 12  .  . 15...
```

## ▽ Triple reversal rotation

"This is an easy and reliable way to rotate in-place. You reverse the
left side, next you reverse the right side, next you reverse the entire
array. Upon completion the left and right block will be swapped. There's
no known first publication, but it was prior to *1981*."[^1]

### Example

```text
                           mid
       left = 9            |    right = 6
[ 1  2  3  4  5  6 :7  8  9*10 11 12 13 14 15]  // reverse left
  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
[ 9  8  7  6  5  4  3  2  1 10 11 12 13 14 15]  // reverse right
                             ↓  ↓  ↓  ↓  ↓  ↓
[ 9  8  7  6  5  4  3  2  1 15 14 13 12 11 10]  // reverse all
  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
[10 11 12 13 14 15 :1  2  3* 4  5  6  7  8  9]
```

## ↬ Gries-Mills rotation

"In some cases this rotation outperforms the classic triple reversal rotation
while making fewer moves. You swap the smallest array linearly towards its
proper location, since the blocks behind it are in the proper location you
can forget about them. What remains of the larger array is now the smallest
array, which you rotate in a similar manner, until the smallest side shrinks
to `0` elements. Its first known publication was in *1981* by *David Gries* and
*Harlan Mills*."[^1]

### Example

```text
                 𝑠ℎ𝑎𝑑𝑜𝑤    mid
          left = 9         |    right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap r-side and shadow
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1  .  3 10  .  .  .  . 15  4 ~~~~~~~~~~~~ 9]

   l = 3     𝑠ℎ. r = 6
[ 1  .  3*10  . 12:13  . 15] 4  .  .  .  .  9   // swap new l-side and new shadow
  └─────┴/\┴─────┘
  ┌─────┬~/┬─────┐
[10 ~~ 12  1  .  3 13  . 15] 4  .  .  .  .  9

           l = 3    r = 3
 10 ~~ 12[ 1  .  3;13  . 15] 4  .  .  .  .  9   // swap equal
          └──────┴/\┴─────┘
          ┌──────┬~~┬─────┐
 10 ~~ 12[13 ~~~ 15 1 ~~~ 3] 4  .  .  .  .  9

[10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
```

## 🏆 Grail (Gries-Mills + *swap_backward*) rotation

"The grail rotation from the Holy *Grail Sort Project*[^2] is *Gries-Mills* derived
and tries to improve locality by shifting memory either left or right depending on which
side it's swapped from.

In addition it performs an auxiliary rotation on stack memory when the smallest side reaches
a size of `1` element, which is the worst case for the *Gries-Mills rotation*. The flow diagram
is identical to that of *Gries-Mills*, but due to memory being shifted from the right the
visualization differs."[^1]

### Examples

```text
                 𝑠ℎ𝑎𝑑𝑜𝑤    mid
          left = 9         |    right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap r-side and shadow
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1  .  3 10  .  .  .  . 15  4 ~~~~~~~~~~~~ 9]

   l = 3     𝑠ℎ. r = 6
[ 1  .  3*10  . 12:13  . 15] 4  .  .  .  .  9   // swap new l-side and new shadow
  └─────┴/\┴─────┘
  ┌─────┬~/┬─────┐
[10 ~~ 12  1  .  3 13  . 15] 4  .  .  .  .  9

           l = 3    r = 3
 10 ~~ 12[ 1  .  3;13  . 15] 4  .  .  .  .  9   // swap equal
          └──────┴/\┴─────┘
          ┌──────┬~~┬─────┐
 10 ~~ 12[13 ~~~ 15 1 ~~~ 3] 4  .  .  .  .  9

[10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
```

## ⊛ Drill rotation

"The drill rotation is a grail variant that utilizes a piston main loop
and a helix inner loop. Performance is similar to the helix rotation.
The flow diagram and visualization are identical to the grail rotation."[^1]

*2021* - Drill rotation by *Igor van den Hoven* (*Grail* derived with *Piston*
and *Helix* loops).

### Examples

```text
                           mid
          left = 9         |     right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1  .  3;10  . 12 13  . 15] 4 ~~~~~~~~~~~~ 9   // swap
  └─────┴/\┴─────────────┘
   ┌─────────────┬~~┬─────┐
[ 10 ~~~~~~~~~~ 15: 1 ~~~ 3* 4  .  .  .  .  9]
```

## 🪠 Successive aka Piston rotation

"First described by *Gries and Mills* in *1981*, this rotation is very similar to
the Gries-Mills rotation but performs non-linear swaps. It is implemented as
the *Piston Rotation* in the benchmark, named after a loop optimization that
removes up to `log n` branch mispredictions by performing both a left and
rightward rotation in each loop."[^1]

### Algorithm

1. Swap the smallest side to its place;
2. repeat with smaller array.

### Properties

* During computation `mid` is never shifted.

### Example

```text
                           mid
          left = 9         |   right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
  └──────────────┴─────/\────┴──────────────┘
  ┌──────────────┬─────~/────┬──────────────┐
[10 ~~~~~~~~~~~ 15: 7  .  9  1  .  .  .  .  6]

                      l = 3        r = 6
 10  .  .  .  . 15[ 7  .  9* 1  .  3: 4  .  6]  // swap
                    └─────┴────/\─────┴─────┘
                    ┌─────┬────\~─────┬─────┐
 10  .  .  .  . 15[ 4  .  6  1  .  3  7 ~~~ 9]

                      l = 3   r = 3
 10  .  .  .  . 15[ 4  .  6; 1  .  3] 7  .  9   // swap
                    └─────┴/\┴─────┘
                    ┌─────┬~~┬─────┐
 10  .  .  .  . 15[ 1 ~~~ 3  4 ~~~ 6] 7  .  9

[10  .  .  .  . 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
```

# 🧬 Helix rotation

"The helix rotation has similarities with the *Gries-Mills
rotation* but has a distinct sequential movement pattern. It is
an improvement upon the *Grail* rotation by merging the two inner
loops into a single loop, significantly improving performance when
the relative size difference between the two halves is large. In
addition it doesn't stop when the smallest block no longer fits,
but continues and recalculates the left or right side. The utilization
of the merged loops is counter-intuitive and is likely novel. Its
first known publication was in *2021* by *Control* from the *Holy Grail
Sort Project*[^2]."[^1]

## Examples

```text
                           mid
          left = 9         |    right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1  .  3 10  . 12 13  . 15  4 ~~~~~~~~~~~~ 9]  // swap
  └─────┴─────/\────┴─────┘
  ┌─────┬─────\~────┬─────┐
[13  . 15;10  . 12] 1 ~~~ 3  4  .  .  .  .  9   // swap
  └─────┴/\┴─────┘
  ┌─────┬~~┬─────┐
[10 ~~ 12 13 ~~ 15] 1 ~~~~~~~~~~~~~~~~~~~~~ 9

[10 ~~~~~~~~~~~ 15: 1  .  3* 4  .  .  .  .  9]
```

```text
                        mid
         left = 8       |      right = 7
[ 1  2  3  4  5  6  7: 8* 9 10 11 12 13 14 15]  // swap
     └─────────────────┴/\┴─────────────────┘
     ┌─────────────────┬\~┬─────────────────┐
[ 1  9 .............. 15  2 ~~~~~~~~~~~~~~~ 8]  // swap
  └─────────/\─────────┘
  ┌─────────\~─────────┐
[15; 9 10 11 12 13 14] 1 ~~~~~~~~~~~~~~~~~~ 8]  // aux rotate

[ 9 ~~~~~~~~~~~~~~ 15: 1* .  .  .  .  .  .  8]

```

## ⎊ Contrev (Conjoined triple reversal) rotation

"The conjoined triple reversal is derived from the triple reversal rotation. Rather than three
separate reversals it conjoins the three reversals, improving locality and reducing
the number of moves. Its first known publication was in *2021* by *Igor van den
Hoven*."[^1]

*2021* - *Conjoined Triple Reversal rotation* by *Igor van den Hoven*.

### Examples

Case: `right > left`, `9 - 6`.

```text
                           mid
  ls-->               <--le|rs-->       <--re
[ 1  2  3  4  5  6: 7  8  9* a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
  ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮ |
  ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆─╯
  ↓  sl               le  |   | sr      re┆
[ a  2  .  .  .  .  .  8  1  f╯ b  .  .  e╰>9]  // (ls, le, re, rs)
     ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮ |
     ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆─╯
     ↓  s           e  |        |  s  e┆
[ a  b  3  .  .  .  7  2  1  f  e  c  d╰>8  9]
        ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈╭╯  |
        ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|─╯
        ↓  s     e  |              e┆
[ a ~~~ c  4  .  6  3  2  1  f  e  d╰>7 ~~~ 9]  // (ls, le, re    )
           ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
           ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
           ↓  sl-|>         <--re┆
[ a ~~~~~~ d  5  4  3  2  1  f  e╰>6 ~~~~~~ 9]  // (ls,     re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
              ↓  ↓  ↓  ↓  ↓  ↓  ↓
[ a ~~~~~~~~~ e  f: 1  2  3* 4  5 ~~~~~~~~~ 9]
```

Case: `left < right`, `6 - 9`.

```text
                  mid
  ls-->      <--le|rs-->                <--re
[ a  b  c  d  e  f* 1  2  3: 4  5  6  7  8  9]  // (ls -> le -> re -> rs -> ls)
  | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
  ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
    ┆sl      le |   |  sr               re  ↓
[ 1<╯b  .  .  e ╰a  9  2  .  .  .  .  .  8  f]  // (ls, le, re, rs)
     | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
     ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
       ┆s  e|          |  s           e  ↓
[ 1  2<╯c  d╰~b  a  9  8  3  .  .  .  7  e  f]
        |╭ ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯     ╭─────╯
        ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
         ┆┆s              |  s     e  ↓
[ 1 ~~~ 3╯╰c  b  a  9  8  7  4  5  6  d ~~~ f]  // (ls,     re, rs)
           ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
            ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
            ┆ sl-->         <|-re  ↓
[ 1 ~~~~~~ 4╯ b  a  9  8  7  6  5  c ~~~~~~ f]  // (ls,     re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
              ↓  ↓  ↓  ↓  ↓  ↓  ↓
[ 1 ~~~~~~~~~ 5  6  7  8  9: a  b ~~~~~~~~~ f]
```

Case: `left > right`, `8 - 7`.

```text
                        mid
  ls-->            <--le|rs-->          <--re
[ 1  2  3  4  5  6  7: 8* a  b  c  d  e  f  g]  // (ls -> le -> re -> rs -> ls)
  ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
  ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭──────────────┆─╯
  ↓  sl            le  |   | sr         re┆
[ a  2  .  .  .  .  7  1  g╯ b  .  .  .  f╰>8]  // (ls, le, re, rs)
     ╰────────╮     ╰┈┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈╮ |
     ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈╯╭─────── ┆─╯
     ↓  s        e  |         | s     e┆
[ a  b  3  .  .  6  2  1  g  f╯ c  .  e╰>7  8]  // (ls, le, re, rs)
        ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮ |
        ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆─╯
        ↓  s  e  |               | e┆
[ a ~~~ c  4  5╮ 3  2  1  g  f  e╯ d╰>6 ~~~ 8]  // (ls, le,     rs)
           ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
           ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆─╯
           ↓  sl-->         <--re┆
[ a ~~~~~~ d  4  3  2  1  g  f  e╰>5 ~~~~~~ 8]  // (ls,     re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
              ↓  ↓  ↓  ↓  ↓  ↓  ↓
[ a ~~~~~~~~~ e  f  g: 1* 2  3  4 ~~~~~~~~~ 8]
```


## GenContrev (Generalized conjoined triple reversal) rotation

It is the generalization of the Contrev rotation. Instead of moving separate
elements, we are moving blocks of elements.

In the situation when `gcd(left, right) = 1` it became the usual Contrev.

## Example
Case: `left > rignt`, `9 > 6`:
```text
                            mid
  ls-->          <--le      |rs--> <--re
[ 1  2  3  4  5  6: 7  8  9 *a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
  |  |  |    ╭┈┈┈┈┈ |┈ |┈ |┈┈┴┈┈┴┈┈╯  |  |  |
  ╰──┴──┴────╮      ╰──┴──┴────╮┈┈┈┈┈┈┴┈┈┴┈┈╯
  ╭┈┈┈┈┈┈┈┈┈ ╰──────╮        | ╰──────╮
  ↓        ls       ↓        ↓re      ↓
[ a ~~~ c  4  .  6  1 ~~~ 3  d  -  f  7 ~~~ 9]  // (ls,         re)
           |     |    ╭┈┈┈┈┈┈┴┈┈┴┈┈╯
           ╰──┴──┴────╮
           ╭┈┈┈┈┈┈┈┈┈ ╰──────╮
           ↓        ls       ↓re
[ a ~~~ c  d ~~~ f: 1 ~~~ 3 *4 ~~~ 6  7 ~~~ 9]


[ A        B      : C      * D        E      ]
[ D ~~~~~~ B        A ~~~~~~ E        C ~~~~ ]
[ D ~~~~~~ E ~~~~~: A ~~~~~* B ~~~~~~ C ~~~~ ]
```



Case: `left > right`, `8 > 6`:

```text
                         mid
  ls-->          <--le   |rs-->    <--re
[ 1  2  3  4  5  6: 7  8 *a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
  |  |    ╭┈┈┈┈┈┈┈┈ |┈ |┈┈┴┈┈╯        |  |
  ╰──┴────╮         ╰──┴───────╮┈┈┈┈┈┈┴┈┈╯
  ╭┈┈┈┈┈┈ ╰─────────╮     |    ╰──────╮
  ↓     ls    le    ↓     ↓     re    ↓
[ a  b  3  4  5  6  1  2  e  f  c  d  7  8]  // (ls,   le,   re)
        |  |  ╰──┤  ╭┈┈┈┈┈┈┈┈┈┈┈┴┈┈╯
        ╰──┴──╮  ╰──────────────╮
        ╭┈┈┈┈ |┈┈┈┈┈╯           |
        ↓     ↓ls         re    ↓
[ a  b  c  d  3  4  1  2  e  f  5  6  7  8]  // (ls,         re)
              |  |      ╭┈┴┈┈╯
              ╰──┴──────| ╮
              ╭┈┈┈┈┈┈┈┈┈╯ |
              ↓           ↓
[ a ~~~~~~ d  e  f  1  2  3  4  5 ~~~~~~~ 8]

[ A     B     C   : D   * E     F     G    ]
[ E ~~~ B     C     A ~~~ G     F     D ~~~]
[ E ~~~ F ~~~ B     A ~~~ G     C ~~~ D ~~~]
[ E ~~~ F ~~~ G ~~~ A ~~~ B ~~~ C ~~~ D ~~~]
```

Case: `left > right`, `8 > 7`:

```text
                        mid
  ls-->            <--le|rs-->          <--re
[ 1  2  3  4  5  6  7: 8* a  b  c  d  e  f  g]  // (ls -> le -> re -> rs -> ls)
  ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
  ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭──────────────┆─╯
  ↓  sl            le  |   | sr         re┆
[ a  2  .  .  .  .  7  1  g╯ b  .  .  .  f╰>8]  // (ls, le, re, rs)
     ╰────────╮     ╰┈┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈╮ |
     ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈╯╭─────── ┆─╯
     ↓  s        e  |         | s     e┆
[ a  b  3  .  .  6  2  1  g  f╯ c  .  e╰>7  8]  // (ls, le, re, rs)
        ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮ |
        ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆─╯
        ↓  s  e  |               | e┆
[ a ~~~ c  4  5╮ 3  2  1  g  f  e╯ d╰>6 ~~~ 8]  // (ls, le,     rs)
           ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
           ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆─╯
           ↓  sl-->         <--re┆
[ a ~~~~~~ d  4  3  2  1  g  f  e╰>5 ~~~~~~ 8]  // (ls,     re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
              ↓  ↓  ↓  ↓  ↓  ↓  ↓
[ a ~~~~~~~~~ e  f  g: 1* 2  3  4 ~~~~~~~~~ 8]
```

## Combined rotations

### ŕ Default (Stable) rotation

Combines direct, auxiliary and piston rotations.

*Direct* is used for small values of `left + right` or for large `T`. The elements are moved
into their final positions one at a time starting at `mid - left` and advancing by `right` steps
modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
`mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
elements. For example:

```text
                           mid
          left = 9         |    right = 6
[ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]                      // round
  └─────────────────┐
[ ✘  2  .  .  .  6  1  8  .  .  .  .  .  . 15] [ 7]
                                      ┌──────────┘
                    _                 ↓
[ ✘  2  .  .  .  6  1  8  .  .  . 12  7 14 15] [13]
           ┌─────────────────────────────────────┘
           ↓        _                 _
[ ✘  2  3 13  5  6  1  8  .  .  . 12  7 14 15] [ 4]
                             ┌───────────────────┘
           _        _        ↓        _
[ ✘  2  3 13  5  6  1  8  9  4 11 12  7 14 15] [10]
  ┌──────────────────────────────────────────────┘
  ↓        _        _        _        _
[10  2  3 13  5  6  1  8  9  4 11 12  7 14 15]                      // round
     |        |        |        |        └──────────────────╮
     |        |        |        └──────────────────╮        ┆
     |        |        └─────────────────┐         ┆        ┆
     |        └─────────────────┐        ┆         ┆        ┆
     └─────────────────┐        ┆        ┆         ┆        ┆
~─────────────╮        ┆        ┆        ┆         ┆        ┆
~────╮        ┆        ┆        ┆        ┆         ┆        ┆
  _  ↓     _  ↓     _  ↓     _  ↓     _  ↓      _  ↓     _  ↓
[10 11  3 13 14  6  1  2  9  4  5 12  7  8 15][10 11  3 13 14...    // round
        |        |        |        |        └──────────────────╮
        |        |        |        └──────────────────╮        ┆
        |        |        └─────────────────┐         ┆        ┆
        |        └─────────────────┐        ┆         ┆        ┆
        └─────────────────┐        ┆        ┆         ┆        ┆
~────────────────╮        ┆        ┆        ┆         ┆        ┆
~───────╮        ┆        ┆        ┆        ┆         ┆        ┆
  _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓   _  _  ↓  _  _  ↓
[10 11 12 13 14 15: 1  2  3* 4  5  6  7  8  9][10 11 12 13 14 15...
```

Fortunately, the number of skipped over elements between finalized elements is always equal, so
we can just offset our starting position and do more rounds (the total number of rounds is the
`gcd(left + right, right)` value). The end result is that all elements are finalized once and
only once.

*Auxiliary* is used if `left + right` is large but `min(left, right)` is small enough to
fit onto a stack buffer. The `min(left, right)` elements are copied onto the buffer, `memmove`
is applied to the others, and the ones on the buffer are moved back into the hole on the
opposite side of where they originated.

Algorithms that can be vectorized outperform the above once `left + right` becomes large enough.
*Direct* can be vectorized by chunking and performing many rounds at once, but there are too
few rounds on average until `left + right` is enormous, and the worst case of a single
round is always there. Instead, algorithm 3 utilizes repeated swapping of
`min(left, right)` elements until a smaller rotate problem is left.

```text
                                 mid
             left = 11           | right = 4
[ 5  6  7  8 :9 10 11 12 13 14 15* 1  2  3  4]  // swap
                       └────────┴/\┴────────┘
                       ┌────────┬~~┬────────┐
[ 5  .  .  .  .  . 11: 1 ~~~~~~ 4 12  .  . 15]

[ 5  .  7  1  .  .  4: 8  .  . 11 12 ~~~~~ 15   // swap
           └────────┴/\┴────────┘
           ┌────────┬~~┬────────┐ 
[ 5  .  7  8 :9  . 11: 1 ~~~~~~ 4*12  .  . 15
we cannot swap any more, but a smaller rotation problem is left to solve
```

when `left < right` the swapping happens from the left instead.

## Benchmarks

To run benchmarks do:

```text
cargo bench
```

Result could be found in the `target/criterion/{name_of_the_benchmarks_group}/report`.

You would have to install `gnuplot` to get the pictures.

Note that benchmarking could take some time :)

[^1]: [https://github.com/scandum/rotate](https://github.com/scandum/rotate)

[^2]: [https://github.com/HolyGrailSortProject](https://github.com/HolyGrailSortProject)
