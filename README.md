# Array rotations in Rust

This page is inspired by [https://github.com/scandum/rotate](https://github.com/scandum/rotate).

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

## Intoduction

A rotation is to swap the left side of an array with the right side:

`[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]`

after the rotation the data is as following:

`[10 11 12 13 14 15: 1  2  3, 4  5  6  7  8  9]`

# Algorithms

## Auxiliary rotation

“This is an easy and fast way to rotate, but since it requires auxiliary memory
it is of little interest to in-place algorithms. It’s a good strategy for array
sizes of `1000` elements or less. The smaller half is copied to swap memory, the
larger half is moved, and the swap memory is copied back to the main array.”
[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Examples

```text
       left = 9    dim     mid   right = 6
[ 1  2  3  4  5  6 :7  8  9,            12-15]
                                         └──┴───────┬─────┐
[              1-6 :7 ... 9  .  .  .  .  .  .]    [10 .. 15]
               └────┬─────┴─────────────────┐                  move
[ .  .  .  .  .  . :1 ~~~~~~~~~~~~~~~~~~~~~ 9]    [10-15   ]
  ┌──────────────┬──────────────────────────────────┴──┘
[10 ~~~~~~~~~~~ 15 :1 ~~~~~~~~~~~~~~~~~~~~~ 9]
```

```text
                                 mid
          left = 11              | right = 4
[ 1  2  3  4: 5  6  7  8  9 10 11,      12-15]
                                         └──┴───────┬─────┐
[ 1 ....... : ................ 11  .  .  .  .]    [12 .. 15]
  └───────────┬─────────────────┴───────────┐                  move
[ .  .  .  .: 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11]    [12-15   ]
  ┌────────┬────────────────────────────────────────┴──┘
[12 ~~~~~ 15: 1...                         11]
```

```text
   left = 4  mid         right = 11
[      12-15, 1  2  3  4  5  6  7: 8  9 10 11]
        └──┴────────────────────────────────────────┬─────┐
[ .  .  .  .  1 ................ : ....... 11]    [12 .. 15]
  ┌───────────┴─────────────────┬───────────┘                  move
[ 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11: .  .  .  .]    [12-15   ]
                                   ┌────────┬───────┴──┘
[ 1 .......................... 11:12 ~~~~~ 15]
```

## Bridge rotation

"This is a slightly more complex auxiliary rotation than
auxiliary rotation that reduces the maximum auxiliary memory
requirement from `50%` to `33.(3)%`. If the overlap between the
two halves is smaller than the halves themselves it copies
the overlap to swap memory instead. Its first known publication
was in *2021* by *Igor van den Hoven*." [https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Examples

Here bridge (elements from `dim` to `mid`) is less than `left` || `right`.
Otherwise, algo fallbacks to `auxiliary`.

```text
         left = 9 dim       mid  right = 6
[ 1  2  3  4  5  6:     7-9,10 11 12 13 14 15]
                        └─┴────────────────────────┬─────┐
  a-->              b-->    c-->                   |     |
[ 1 ............ 6: .  .  . 10 ........... 15]    [7  8  9]
  └─────────────────┐       |
  ╭─ a ──────────── |─ b ───╯  c
[10  2 ......... 6: 1  .  .  . 11 ........ 15]    [7 ... 9]
     └─────────────────┐       |
     ╭─ a ──────────── |─ b ───╯  c
[10 11  3 ...... 6: 1  2  .  .  . 12 ..... 15]    [7 ... 9]
        └─────────────────┐       |
        ╭─ a ──────────── |─ b ───╯  c
[10 ~~ 12  4  ...6: 1 ~~~ 3  .  .  . 13 .. 15]    [7 ... 9]
           └─────────────────┐       |
           ╭─ a ──────────── |─ b ───╯  c
[10 ~~~~~ 13  5  6: 1 ~~~~~~ 4  .  .  . 14 15]    [7 ... 9]
              └─────────────────┐       |
              ╭─ a ──────────── |─ b ───╯  c
[10 ~~~~~~~~ 14  6: 1 ~~~~~~~~~ 5  .  .  . 15]    [7 ... 9]
                 └─────────────────┐       |
                 ╭──────────────── |─ b ───╯
[10 ~~~~~~~~~~~ 15: 1 ~~~~~~~~~~~~ 6  .  .  .]    [7-9    ]
                                      ┌─────┬──────┴─┘
[10 ........... 15: 1 ............ 6  7 ~~~ 9]
```

```text
      left = 6    mid           right = 6
[10 11 12 13 14 15,     1-3: 4  5  6  7  8  9]
                        └─┴────────────────────────┬─────┐
                    b       c               d      |     |
[10 ........... 15  .  .  .: 4 ............ 9]    [1  2  3]
                 ╰─────────────────────────╮|
                 b       c┌───────────── d |┘
[10 ........ 14  .: .  .  9  4 ......... 8 15]    [1 ... 3]
              ╰─────────────────────────╮|
              b       c┌───────────── d |┘
[10 11 .. 13  .  .: .  8  9  4 ...... 7 14 15]    [1 ... 3]
           ╰─────────────────────────╮|
           b       c┌───────────── d |┘
[10 11 12  .  .  .: 7 ~~~ 9  4 ... 6 13 ~~ 15]    [1 ... 3]
        ╰─────────────────────────╮|
        b       c┌───────────── d |┘
[10 11  .  .  .  6:~~~~~~ 9  4  5 12 ~~~~~ 15]    [1 ... 3]
     ╰─────────────────────────╮|
     b       c┌───────────── d |┘
[10  .  .  .  5 ~~:~~~~~~ 9  4 11 ~~~~~~~~ 15]    [1 ... 3]
  ╰─────────────────────────╮|
          c┌───────────── d |┘
[ .  .  .  4  5 ~~:~~~~~~ 9 10 ~~~~~~~~~~~ 15]    [1-3    ]
  ┌─────┬──────────────────────────────────────────┴─┘
[ 1 ~~~ 3  4 ............ 9 10 ........... 15]
```

## Juggling rotation

"Also known as the dolphin algorithm. This is a relatively complex
and inefficient way to rotate in-place, though it does so in the
minimal number of moves. Its first known publication was in *1966*.
It computes the greatest common divisor and uses a loop to create
a chain of consecutive swaps."
[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Example

```text
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]
  |        |        |        |        └─────────────────╮
  |        |        |        └──────────────────╮       |
  |        |        └─────────────────┐         |       |
  |        └─────────────────┐        |         |       |
  └─────────────────┐        |        |         |       |
~──────────╮        |        |        |         |       |
~─╮        |        |        |        |         |       |
[10  2  3 13  5  6: 1  8  9, 4 11 12  7 14 15][10  2  3 13...
     |        |        |        |        └──────────────────╮
     |        |        |        └──────────────────╮        |
     |        |        └─────────────────┐         |        |
     |        └─────────────────┐        |         |        |
     └─────────────────┐        |        |         |        |
~─────────────╮        |        |        |         |        |
~────╮        |        |        |        |         |        |
[10 11  3 13 14  6: 1  2  9, 4  5 12  7  8 15][10 11  3 13 14...
        |        |        |        |        └──────────────────╮
        |        |        |        └──────────────────╮        |
        |        |        └─────────────────┐         |        |
        |        └─────────────────┐        |         |        |
        └─────────────────┐        |        |         |        |
~────────────────╮        |        |        |         |        |
~───────╮        |        |        |        |         |        |
[   ...12 ..... 15: ..... 3 ...... 6 ...... 9][ .... 12 ..... 15:...
```

## Triple reversal rotation

"This is an easy and reliable way to rotate in-place. You reverse the
left side, next you reverse the right side, next you reverse the entire
array. Upon completion the left and right block will be swapped. There's
no known first publication, but it was prior to *1981*."
[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Example

```text
       left = 9            mid  right = 6
[ 1  2  3  4  5  6 :7  8  9,10 11 12 13 14 15]
  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓                      reverse left
[ 9  8  7  6  5  4  3  2  1 10 11 12 13 14 15]
                             ↓  ↓  ↓  ↓  ↓  ↓    reverse right
[ 9  8  7  6  5  4  3  2  1 15 14 13 12 11 10]
  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓    reverse all
[10 11 12 13 14 15 :1  2  3, 4  5  6  7  8  9]
```

## Gries-Mills rotation

"In some cases this rotation outperforms the classic triple reversal rotation
while making fewer moves. You swap the smallest array linearly towards its
proper location, since the blocks behind it are in the proper location you
can forget about them. What remains of the larger array is now the smallest
array, which you rotate in a similar manner, until the smallest side shrinks
to `0` elements. Its first known publication was in *1981* by David Gries and
Harlan Mills." [https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Example

```text
          left = 9         mid   right = 6
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\/┬──────────────┐
[ 1 ... 3 10 ........... 15  4 ~~~~~~~~~~~~ 9]

   l = 3       r = 6
[ 1 .. 3, 10 .. 12 13 .. 15] 4 ~~~~~~~~~~~~ 9    swap
  └────┴/\/┴─────┘
  ┌────┬\/\┬─────┐
[10 ~ 12   1 ... 3 13 .. 15] 4 ~~~~~~~~~~~~ 9

           l = 3    r = 3
 10 ~ 12[ 1 .... 3;13 .. 15] 4 ~~~~~~~~~~~~ 9    swap
          └──────┴/\┴─────┘
          ┌──────┬\/┬─────┐
 10 ~ 12[13 ~~~ 15  1 ~~~ 3] 4 ~~~~~~~~~~~~ 9

[10 ........... 15: 1 ... 3  4 ............ 9]
```

## Grail rotation

"The grail rotation from the Holy Grail Sort Project is Gries-Mills derived
and tries to improve locality by shifting memory either left or right depending on which
side it's swapped from.

In addition it performs an auxiliary rotation on stack memory when the smallest side reaches
a size of 1 element, which is the worst case for the Gries-Mills rotation. The flow diagram
is identical to that of Gries-Mills, but due to memory being shifted from the right the
visualization differs."

[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Examples

```text
          left = 9         mid   right = 6
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1 ... 3;10    12 13 .. 15] 4 ~~~~~~~~~~~~ 9    swap
  └─────┴/\┴─────┘
  ┌─────┬~/┬─────┐
[10 ~~ 12  1 ... 3 13 .. 15] 4 ~~~~~~~~~~~~ 9    swap
           └─────┴/\┴─────┘
           ┌─────┬~~┬─────┐
 10 ~~ 12[13 ~~ 15  1 ~~~ 3] 4 ~~~~~~~~~~~~ 9    swap

[10 ........... 15: 1 ... 3  4 ~~~~~~~~~~~~ 9]
```

## Drill rotation

"The drill rotation is a grail variant that utilizes a piston main loop
and a helix inner loop. Performance is similar to the helix rotation.
The flow diagram and visualization are identical to the grail rotation."
*2021* - Drill rotation by Igor van den Hoven (grail derived with piston
and helix loops)
[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Examples

```text
          left = 9         mid   right = 6
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
           └──────────────┴/\┴──────────────┘
           ┌──────────────┬\~┬──────────────┐
[ 1 ... 3;10 .. 12 13 .. 15] 4 ~~~~~~~~~~~~ 9    swap
  └─────┴/\┴─────────────┘
   ┌─────────────┬~~┬─────┐
[ 10 ~~~~~~~~~~ 15  1 ~~~ 3  4 ~~~~~~~~~~~~ 9]   swap
```

## Successive rotation

"First described by *Gries and Mills* in *1981*, this rotation is very similar to
the Gries-Mills rotation but performs non-linear swaps. It is implemented as
the *Piston Rotation* in the benchmark, named after a loop optimization that
removes up to `log n` branch mispredictions by performing both a left and
rightward rotation in each loop." [https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Example

```text
          left = 9         mid  right = 6
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
  └──────────────┴/\/\/\/\/\/┴──────────────┘
  ┌──────────────┬\/\/\/\/\/\┬──────────────┐
[10 ~~~~~~~~~~~ 15: 7 ... 9  1 ............ 6]

                      l = 3        r = 6
 10 ~~~~~~~~~~~ 15[ 7 ... 9, 1 ... 3: 7 ... 6]   swap
                    └─────┴/\/\/\/\/\/┴─────┘
                    ┌─────┬\/\/\/\/\/\┬─────┐
 10 ~~~~~~~~~~~ 15[ 4 ... 6  1 ... 3  7 ~~~ 9]

                      l = 3   r = 3
 10 ~~~~~~~~~~~ 15[ 4 ... 6; 1 ... 3] 7 ~~~ 9    swap
                    └─────┴/\┴─────┘
                    ┌─────┬\/┬─────┐
 10 ~~~~~~~~~~~ 15[ 1 ~~~~~~~~~~~~ 6] 7 ~~~ 9

[10 ........... 15: 1 ..................... 9]
```

## Contrev (Conjoined triple reversal) rotation

2021 - Conjoined Triple Reversal rotation by Igor van den Hoven

"The conjoined triple reversal is derived from the triple reversal rotation. Rather than three
separate reversals it conjoins the three reversals, improving locality and reducing
the number of moves. Its first known publication was in 2021 by Igor van den
Hoven." [https://github.com/scandum/rotate](https://github.com/scandum/rotate)

### Examples

Case: `right > left`, `9 - 6`.
```text
 ls-->                <--le rs-->       <--re
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
  ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮|
  ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆╯
  ┆ ls                le  |   |rs       re┆
[10  2 ............... 8  1,15╯11 ..... 14╰~9]   //(ls, le, re, rs)
     ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮|
     ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆╯
     ┆ ls          le  |        | rs re┆
[10 11  3 ......... 7  2  1,15 14 12 13╰~8  9]
        ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ╮|
        ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|╯
        ┆ ls    le  |             re┆
[10 ~~ 12  4 ... 6  3  2  1,15 14 13╰~7 ~~~ 9]   //(ls, le, re)
           ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
           ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
           ┆ ls  |             re┆
[10 ~~~~~ 13  5  4  3  2  1,15 14╰~6 ~~~~~~ 9]   //(ls, re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[10 ~~~~~ 13 14 15: 1  2  3, 4  5  6 ~~~~~~ 9]
```

Case: `left < right`, `6 - 9`.

```text
 ls-->       <--le rs-->                <--re
[ 1  2  3  4  5  6, 7  8  9:10 11 12 13 14 15]   //(re -> rs -> ls -> le -> re)
  | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
  ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
    ┆ls      le |   | rs                re  ┆
[ 7~╯2 ...... 5 ╰1,15  8 .............. 14  6]   //(re, rs, ls, le)
     | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
     ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
       ls le|          | rs          re  ┆
[ 7  8~╯3  4╰~2  1,15 14  9 ........ 13  5  6]
        |╭ ┆┈┈┈┈┈┈┈┈┈┈┈╯        ╭─────╯
        ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
         ┆ls              | rs    re  ┆
[ 7 ~~~ 9╯╰3  2  1,15 14 13 10 .. 12  4 ~~~ 6]   //(re, rs, ls)
           ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
            ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
            ┆ ls             | re  ┆
[ 7 ~~~~~ 10╯ 2  1,15 14 13 12 11  3 ~~~~~~ 6]   //(re, ls)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[ 7 ~~~~~ 10 11 12,13 14 15: 1  2  3 ~~~~~~ 6]
```

Case: `left > right`, `8 - 7`.

```text
 ls-->             <--le rs-->          <--re
[ 1  2  3  4  5  6  7: 8, 9 10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
  ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
  ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭───────────── ┆╯
  ┆ ls             le  |   |rs          re┆
[ 9  2 ............ 7: 1,15╯10 11 12 13 14╰~8]   //(ls, le, re, rs)
     ╰────────╮     ╰┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈┈╮|
     ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈╯ ╭─────── ┆╯
     ┆ ls       le  |         |rs    re┆
[ 9 10  3 ...... 6  2: 1,15 14╯11 12 13╰~7  8]   //(ls, le, re, rs)
        ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮|
        ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆╯
        ┆ ls le  |               |rs┆
[ 9 ~~ 11  4  5╮ 3  2: 1,15 14 13╯12╰~6 ~~~ 8]   //(ls, le, rs)
           ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
           ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆╯
           | ls                re┆
[ 9 ~~~~~ 12  4  3  2: 1,15 14 13╰~5 ~~~~~~ 8]   //(ls, re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[ 9 ~~~~~ 12 13 14 15: 1, 2  3  4  5 ~~~~~~ 8]
```

## Combined rotations

### Trinity

2021 - Trinity rotation by Igor van den Hoven (Conjoined Triple Reversal + Bridge rotation)

"The trinity rotation (aka conjoined triple reversal) is derived from the triple reversal
rotation. Rather than three separate reversals it conjoins the three reversals, improving
locality and reducing the number of moves. Optionally, if the overlap is smaller than
`32 * size_of(usize)`, it skips the trinity rotation and performs an auxiliary
or bridge rotation on stack memory. Its first known publication was in 2021 by Igor van den Hoven."
[https://github.com/scandum/rotate](https://github.com/scandum/rotate)

#### Example

Case: `right > left`, `9 - 6`.

```text
 ls-->                <--le rs-->       <--re
[ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
  ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮|
  ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆╯
  ┆ ls                le  |   |rs       re┆
[10  2 ............... 8  1,15╯11 ..... 14╰~9]   //(ls, le, re, rs)
     ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮|
     ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆╯
     ┆ ls          le  |        | rs re┆
[10 11  3 ......... 7  2  1,15 14 12 13╰~8  9]
        ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ╮|
        ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|╯
        ┆ ls    le  |             re┆
[10 ~~ 12  4 ... 6  3  2  1,15 14 13╰~7 ~~~ 9]   //(ls, le, re)
           ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
           ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
           ┆ ls  |             re┆
[10 ~~~~~ 13  5  4  3  2  1,15 14╰~6 ~~~~~~ 9]   //(ls, re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[10 ~~~~~ 13 14 15: 1  2  3, 4  5  6 ~~~~~~ 9]
```

Case: `left < right`, `6 - 9`.

```text
 ls-->       <--le rs-->                <--re
[ 1  2  3  4  5  6, 7  8  9:10 11 12 13 14 15]   //(re -> rs -> ls -> le -> re)
  | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
  ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
    ┆ls      le |   | rs                re  ┆
[ 7~╯2 ...... 5 ╰1,15  8 .............. 14  6]   //(re, rs, ls, le)
     | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
     ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
       ls le|          | rs          re  ┆
[ 7  8~╯3  4╰~2  1,15 14  9 ........ 13  5  6]
        |╭ ┆┈┈┈┈┈┈┈┈┈┈┈╯        ╭─────╯
        ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
         ┆ls              | rs    re  ┆
[ 7 ~~~ 9╯╰3  2  1,15 14 13 10 .. 12  4 ~~~ 6]   //(re, rs, ls)
           ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
            ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
            ┆ ls             | re  ┆
[ 7 ~~~~~ 10╯ 2  1,15 14 13 12 11  3 ~~~~~~ 6]   //(re, ls)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[ 7 ~~~~~ 10 11 12,13 14 15: 1  2  3 ~~~~~~ 6]
```

Case: `left > right`, `8 - 7`.

```text
 ls-->             <--le rs-->          <--re
[ 1  2  3  4  5  6  7: 8, 9 10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
  ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
  ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭───────────── ┆╯
  ┆ ls             le  |   |rs          re┆
[ 9  2 ............ 7: 1,15╯10 11 12 13 14╰~8]   //(ls, le, re, rs)
     ╰────────╮     ╰┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈┈╮|
     ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈╯ ╭─────── ┆╯
     ┆ ls       le  |         |rs    re┆
[ 9 10  3 ...... 6  2: 1,15 14╯11 12 13╰~7  8]   //(ls, le, re, rs)
        ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮|
        ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆╯
        ┆ ls le  |               |rs┆
[ 9 ~~ 11  4  5╮ 3  2: 1,15 14 13╯12╰~6 ~~~ 8]   //(ls, le, rs)
           ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
           ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆╯
           | ls                re┆
[ 9 ~~~~~ 12  4  3  2: 1,15 14 13╰~5 ~~~~~~ 8]   //(ls, re)
              ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
              ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
[ 9 ~~~~~ 12 13 14 15: 1, 2  3  4  5 ~~~~~~ 8]
```

### Default

Combines juggler, auxiliary and piston rotations.

*Algorithm 1* is used for small values of `left + right` or for large `T`. The elements are moved
into their final positions one at a time starting at `mid - left` and advancing by `right` steps
modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
`mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
elements. For example:

```text
left = 10, right = 6
the `^` indicates an element in its final place
6 7 8 9 10 11 12 13 14 15 . 0 1 2 3 4 5
after using one step of the above algorithm (The X will be overwritten at the end of the round,
and 12 is stored in a temporary):
X 7 8 9 10 11 6 13 14 15 . 0 1 2 3 4 5
               ^
after using another step (now 2 is in the temporary):
X 7 8 9 10 11 6 13 14 15 . 0 1 12 3 4 5
              ^                 ^
after the third step (the steps wrap around, and 8 is in the temporary):
X 7 2 9 10 11 6 13 14 15 . 0 1 12 3 4 5
    ^         ^                 ^
after 7 more steps, the round ends with the temporary 0 getting put in the X:
0 7 2 9 4 11 6 13 8 15 . 10 1 12 3 14 5
^   ^   ^    ^    ^       ^    ^    ^
```

Fortunately, the number of skipped over elements between finalized elements is always equal, so
we can just offset our starting position and do more rounds (the total number of rounds is the
`gcd(left + right, right)` value). The end result is that all elements are finalized once and
only once.

*Algorithm 2* is used if `left + right` is large but `min(left, right)` is small enough to
fit onto a stack buffer. The `min(left, right)` elements are copied onto the buffer, `memmove`
is applied to the others, and the ones on the buffer are moved back into the hole on the
opposite side of where they originated.

Algorithms that can be vectorized outperform the above once `left + right` becomes large enough.
*Algorithm 1* can be vectorized by chunking and performing many rounds at once, but there are too
few rounds on average until `left + right` is enormous, and the worst case of a single
round is always there. Instead, algorithm 3 utilizes repeated swapping of
`min(left, right)` elements until a smaller rotate problem is left.

```text
                                  mid
             left = 11            | right = 4
[ 5  6  7  8  9 10 11:12 13 14 15 |1  2  3  4]   swap
                       └────────┴/\┴────────┘
                       ┌────────┬~~┬────────┐
[ 5 .............. 11: 1 ~~~~~~ 4 12 13 14 15]
                                   ^  ^  ^  ^
[ 5 ... 7  1  2  3  4: 8  9 10 11 12 ~~~~~ 15    swap
           └────────┴/\┴────────┘
           ┌────────┬~~┬────────┐
[ 5 ... 7  8 ~~~~~ 11: 1 ~~~~~~ 4 12 ..... 15
we cannot swap any more, but a smaller rotation problem is left to solve
```

when `left < right` the swapping happens from the left instead.

## Benchmarks

To run benchmarks do:

```text
cargo bench
```

The results could be found in the `target/criterion/{name_of_the_benchmarks_group}/report`.

You would have to install `gnuplot` to get the pictures.

Note that benchmarking could take some time :)
