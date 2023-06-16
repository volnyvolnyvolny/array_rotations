/*
Copyright (C) 2023 Valentin Vasilev.
*/

/*
Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SO↓FTWARE.
*/

#![doc = include_str!("../README.md")]
//#![feature(sized_type_properties)]

use std::mem::MaybeUninit;
//use std::mem::SizedTypeProperties;

use std::cmp;

use std::ptr;
use std::slice;

pub mod buf;
pub use buf::*;

pub mod utils;
pub use utils::*;

pub mod gm;
pub use gm::*;

/// # Edge case (left || right = 1)
///
/// Rotates the range `[mid-1, mid+right)` or `[mid-left, mid+1)` such that the element
/// at `mid` becomes the first element. Equivalently, rotates the range `left` elements
/// to the left or `right` elements to the right.
///
/// This case is optimized for the `left = 1` or `right = 1` situation.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
pub unsafe fn ptr_edge_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if left == 0 || right == 0 {
        return;
    }

    let start = mid.sub(left);

    if left == 1 {
        if right == 1 {
            ptr::swap(start, mid);
        } else {
            let tmp = start.read();

            shift_left(mid, right);
            mid.add(right - 1).write(tmp);
        }
    } else if right == 1 {
        let tmp = mid.read();

        shift_right(start, left);
        start.write(tmp);
    } else {
        // fallback
        ptr_direct_rotate::<T>(left, mid, right);
    }
}

/// # ContrevB (Generalized conjoined triple reversal) rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Algorithm
///
/// It is the generalization of the Contrev rotation. Instead of moving separate elements we move
/// blocks of elements.
///
/// When `gcd(left, right) = 1` it became the usual Contrev.
///
/// ## Example
///
/// Case: `left > rignt`, `9 > 6`:
///
/// ```text
///                             mid
///   ls-->          <--le      |rs--> <--re
/// [ 1  2  3  4  5  6: 7  8  9 *a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
///   |  |  |    ╭┈┈┈┈┈ |┈ |┈ |┈┈┴┈┈┴┈┈╯  |  |  |
///   ╰──┴──┴────╮      ╰──┴──┴────╮┈┈┈┈┈┈┴┈┈┴┈┈╯
///   ╭┈┈┈┈┈┈┈┈┈ ╰──────╮        | ╰──────╮
///   ↓        ls       ↓        ↓re      ↓
/// [ a ~~~ c  4  .  6  1 ~~~ 3  d  -  f  7 ~~~ 9]  // (ls,         re)
///            |     |    ╭┈┈┈┈┈┈┴┈┈┴┈┈╯
///            ╰──┴──┴────╮
///            ╭┈┈┈┈┈┈┈┈┈ ╰──────╮
///            ↓        ls       ↓re
/// [ a ~~~ c  d ~~~ f: 1 ~~~ 3 *4 ~~~ 6  7 ~~~ 9]
///
/// [ A        B      : C      * D        E      ]
/// [ D ~~~~~~ B        A ~~~~~~ E        C ~~~~ ]
/// [ D ~~~~~~ E ~~~~~: A ~~~~~* B ~~~~~~ C ~~~~ ]
/// ```
///
/// Case: `left > right`, `8 > 6`:
///
/// ```text
///                          mid
///   ls-->          <--le   |rs-->    <--re
/// [ 1  2  3  4  5  6: 7  8 *a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
///   |  |    ╭┈┈┈┈┈┈┈┈ |┈ |┈┈┴┈┈╯        |  |
///   ╰──┴────╮         ╰──┴───────╮┈┈┈┈┈┈┴┈┈╯
///   ╭┈┈┈┈┈┈ ╰─────────╮     |    ╰──────╮
///   ↓     ls    le    ↓     ↓     re    ↓
/// [ a  b  3  4  5  6  1  2  e  f  c  d  7  8]  // (ls,   le,   re)
///         |  |  ╰──┤  ╭┈┈┈┈┈┈┈┈┈┈┈┴┈┈╯
///         ╰──┴──╮  ╰──────────────╮
///         ╭┈┈┈┈ |┈┈┈┈┈╯           |
///         ↓     ↓ls         re    ↓
/// [ a  b  c  d  3  4  1  2  e  f  5  6  7  8]  // (ls,         re)
///               |  |      ╭┈┴┈┈╯
///               ╰──┴──────| ╮
///               ╭┈┈┈┈┈┈┈┈┈╯ |
///               ↓           ↓
/// [ a ~~~~~~ d  e  f  1  2  3  4  5 ~~~~~~~ 8]
///
/// [ A     B     C   : D   * E     F     G    ]
/// [ E ~~~ B     C     A ~~~ G     F     D ~~~]
/// [ E ~~~ F ~~~ B     A ~~~ G     C ~~~ D ~~~]
/// [ E ~~~ F ~~~ G ~~~ A ~~~ B ~~~ C ~~~ D ~~~]
/// ```
///
/// Case: `left > right`, `8 > 7`:
///
/// ```text
///                         mid
///   ls-->            <--le|rs-->          <--re
/// [ 1  2  3  4  5  6  7: 8* a  b  c  d  e  f  g]  // (ls -> le -> re -> rs -> ls)
///   ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭──────────────┆─╯
///   ↓  sl            le  |   | sr         re┆
/// [ a  2  .  .  .  .  7  1  g╯ b  .  .  .  f╰>8]  // (ls, le, re, rs)
///      ╰────────╮     ╰┈┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈╮ |
///      ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈╯╭─────── ┆─╯
///      ↓  s        e  |         | s     e┆
/// [ a  b  3  .  .  6  2  1  g  f╯ c  .  e╰>7  8]  // (ls, le, re, rs)
///         ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮ |
///         ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆─╯
///         ↓  s  e  |               | e┆
/// [ a ~~~ c  4  5╮ 3  2  1  g  f  e╯ d╰>6 ~~~ 8]  // (ls, le,     rs)
///            ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
///            ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆─╯
///            ↓  sl-->         <--re┆
/// [ a ~~~~~~ d  4  3  2  1  g  f  e╰>5 ~~~~~~ 8]  // (ls,     re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ a ~~~~~~~~~ e  f  g: 1* 2  3  4 ~~~~~~~~~ 8]
/// ```
pub unsafe fn ptr_block_contrev_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if left <= 1 || right <= 1 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    if left == right {
        ptr::swap_nonoverlapping(mid, mid.sub(left), right);
    } else {
        let block_size = gcd::binary_usize(left, right);

        if block_size == 1 {
            ptr_contrev_rotate(left, mid, right);
        } else {
            let (mut ls, mut le) = (mid.sub(left), mid.sub(block_size));
            let (mut rs, mut re) = (mid, mid.add(right).sub(block_size));

            let half_min = cmp::min(left, right) / block_size / 2;
            let half_max = cmp::max(left, right) / block_size / 2;

            for _ in 0..half_min {
                // Permutation (ls, le, re, rs)
                for _ in 0..block_size {
                    ls.write(rs.replace(re.replace(le.replace(ls.read()))));

                    ls = ls.add(1);
                    le = le.add(1);
                    rs = rs.add(1);
                    re = re.add(1);
                }

                le = le.sub(2 * block_size);
                re = re.sub(2 * block_size);
            }

            if left > right {
                for _ in 0..half_max - half_min {
                    // (ls, le, re)
                    for _ in 0..block_size {
                        ls.write(re.replace(le.replace(ls.read())));

                        ls = ls.add(1);
                        le = le.add(1);
                        re = re.add(1);
                    }

                    le = le.sub(2 * block_size);
                    re = re.sub(2 * block_size);
                }
            } else {
                for _ in 0..half_max - half_min {
                    // (rs, re, ls)
                    for _ in 0..block_size {
                        ls.write(rs.replace(re.replace(ls.read())));

                        ls = ls.add(1);
                        rs = rs.add(1);
                        re = re.add(1);
                    }

                    re = re.sub(2 * block_size);
                }
            }

            let center = (re.offset_from(ls).abs() / 2) as usize / block_size;

            for _ in 0..center {
                for _ in 0..block_size {
                    // (re, ls)
                    ls.write(re.replace(ls.read()));

                    ls = ls.add(1);
                    re = re.add(1);
                }

                re = re.sub(2 * block_size);
            }
        }
    }
}

// unsafe fn print<T>(label: &str, mut p: *const T, size: usize)
// where
//     T: std::fmt::Debug,
// {
//     print!("{} [", label);

//     for i in 0..size {
//         if i == size - 1 {
//             print!("{:?}", p.read());
//         } else {
//             print!("{:?} ", p.read());
//             p = p.add(1);
//         }
//     }

//     println!("]");
// }

/// # Triple reversal rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 1. Reverse l-side;
/// 2. reverse r-side;
/// 3. revere all.
///
/// "This is an easy and reliable way to rotate in-place. You reverse the
/// left side, next you reverse the right side, next you reverse the entire
/// array. Upon completion the left and right block will be swapped. There's
/// no known first publication, but it was prior to 1981." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                            mid
///        left = 9            |    right = 6
/// [ 1  2  3  4  5  6 :7  8  9*10 11 12 13 14 15]  // reverse left
///   ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ 9  8  7  6  5  4  3  2  1 10 11 12 13 14 15]  // reverse right
///                              ↓  ↓  ↓  ↓  ↓  ↓
/// [ 9  8  7  6  5  4  3  2  1 15 14 13 12 11 10]  // reverse all
///   ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [10 11 12 13 14 15 :1  2  3* 4  5  6  7  8  9]
/// ```
pub unsafe fn ptr_reversal_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if right <= 1 || left <= 1 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    let start = mid.sub(left);

    if left == right {
        ptr::swap_nonoverlapping(mid, start, left);
    } else {
        #[inline(always)]
        unsafe fn reverse_slice<T>(p: *mut T, size: usize) {
            let slice = slice::from_raw_parts_mut(p, size);
            slice.reverse();
        }

        reverse_slice(start, left);
        reverse_slice(mid, right);
        reverse_slice(start, left + right);
    }
}

/// # Triple block reversal rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 0. Block size = GCD(left, right);
/// 1. Reverse blocks of l-side;
/// 2. reverse blocks of r-side;
/// 3. revere all blocks.
///
/// "This is an easy and reliable way to rotate in-place. You reverse the
/// left side, next you reverse the right side, next you reverse the entire
/// array. Upon completion the left and right block will be swapped."
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                            mid
///        left = 9            |    right = 6
/// [ 1  2  3  4  5  6 :7  8  9* a  b  c  d  e  f]  // reverse left side blocks
///   ↓        ↓        ↓
/// [ 7  .  9  4  .  6  1 ~~~ 3  a  .  c  d  .  f]  // reverse right side blocks
///                              ↓        ↓
/// [ 7  .  9  4  .  6  1 ~~~ 3  d  .  f  a  .  c]  // reverse all blocks
///   ↓        ↓                 ↓        ↓
/// [ a ~~~ c  d ~~~ f  1 ~~~ 3  4 ~~~ 6  7 ~~~ 9]
/// ```
pub unsafe fn ptr_block_reversal_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if right <= 1 || left <= 1 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    let start = mid.sub(left);

    if left == right {
        ptr::swap_nonoverlapping(mid, start, left);
    } else {
        let block_size = gcd::binary_usize(left, right);

        if block_size == 1 {
            ptr_reversal_rotate(left, mid, right);
        } else {
            #[inline(always)]
            unsafe fn reverse<T>(p: *mut T, count: usize, block_size: usize) {
                let mut start = p;
                let mut end = p.add((count - 1) * block_size);

                for _ in 0..count / 2 {
                    ptr::swap_nonoverlapping(start, end, block_size);
                    start = start.add(block_size);
                    end = end.sub(block_size);
                }
            }

            reverse(start, left / block_size, block_size);
            reverse(mid, right / block_size, block_size);
            reverse(start, (left + right) / block_size, block_size);
        }
    }
}

/// # Successive aka Piston rotation (recursive variant)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 1. Swap the smallest side to its place;
/// 2. repeat for a smaller array.
///
/// "First described by *Gries and Mills* in *1981*, this rotation is very similar to
/// the Gries-Mills rotation but performs non-linear swaps. It is implemented as
/// the *Piston Rotation* in the benchmark, named after a loop optimization that
/// removes up to `log n` branch mispredictions by performing both a left and
/// rightward rotation in each loop." <<https://github.com/scandum/rotate>>
///
/// ## Properties
///
/// * During computation `mid` is never shifted.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
///   └──────────────┴─────/\────┴──────────────┘
///   ┌──────────────┬─────~/────┬──────────────┐
/// [10 ~~~~~~~~~~~ 15: 7  .  9  1  -  -  -  -  6]
///
///                      l = 3        r = 6
///  10  .  .  .  . 15[ 7  .  9* 1  -  3: 4  -  6]  // swap
///                     └─────┴────/\─────┴─────┘
///                     ┌─────┬────\~─────┬─────┐
///  10  .  .  .  . 15[ 4  -  6  1  -  3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10  .  .  .  . 15[ 4  -  6; 1  -  3] 7 ~~~ 9   // swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬~~┬─────┐
///  10  .  .  .  . 15[ 1 ~~~ 3  4 ~~~ 6] 7 ~~~ 9
///
/// [10  .  .  .  . 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
/// ```
pub unsafe fn ptr_piston_rotate_rec<T>(left: usize, mid: *mut T, right: usize) {
    if left <= 1 || right <= 1 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    let start = mid.sub(left);

    if left < right {
        ptr::swap_nonoverlapping(start, start.add(right), left);
        ptr_piston_rotate_rec(left, mid, right - left);
    } else {
        ptr::swap_nonoverlapping(mid, start, right);
        ptr_piston_rotate_rec(left - right, mid, right);
    }
}

/// # Successive aka Piston rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 1. Swap the smallest side to its place;
/// 2. repeat for a smaller array.
///
/// "First described by *Gries and Mills* in *1981*, this rotation is very similar to
/// the *Gries-Mills rotation* but performs non-linear swaps. It is implemented as
/// the *Piston rotation* in the benchmark, named after a loop optimization that
/// removes up to `log n` branch mispredictions by performing both a left and
/// rightward rotation in each loop." <<https://github.com/scandum/rotate>>
///
/// ## Properties
///
/// * During computation `mid` is never shifted.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// # Example
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
///   └──────────────┴─────/\────┴──────────────┘
///   ┌──────────────┬─────~/────┬──────────────┐
/// [10 ~~~~~~~~~~~ 15: 7  .  9  1  -  -  -  -  6]
///
///                       l = 3        r = 6
///  10  .  .  .  . 15[ 7  .  9* 1  -  3: 4  -  6]  // swap
///                     └─────┴─────/\────┴─────┘
///                     ┌─────┬─────\~────┬─────┐
///  10  .  .  .  . 15[ 4  -  6  1  -  3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10  .  .  .  . 15[ 4  -  6; 1  -  3] 7  .  9   // swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬~~┬─────┐
///  10  .  .  .  . 15[ 1 ~~~ 3  4 ~~~ 6] 7  .  9
///
/// [10  .  .  .  . 15: 1  .  3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_piston_rotate<T>(mut left: usize, mid: *mut T, mut right: usize) {
    loop {
        if left <= 1 {
            break;
        }

        while left <= right {
            ptr::swap_nonoverlapping(mid.sub(left), mid.add(right - left), left);
            right -= left;
        }

        if right <= 1 {
            break;
        }

        while left >= right {
            ptr::swap_nonoverlapping(mid, mid.sub(left), right);
            left -= right;
        }
    }

    if left == 1 || right == 1 {
        ptr_edge_rotate(left, mid, right);
    }
}

/// # Helix rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "The helix rotation has similarities with the *Gries-Mills
/// rotation* but has a distinct sequential movement pattern. It is
/// an improvement upon the *Grail rotation* by merging the two inner
/// loops into a single loop, significantly improving performance when
/// the relative size difference between the two halves is large. In
/// addition it doesn't stop when the smallest block no longer fits,
/// but continues and recalculates the left or right side. The utilization
/// of the merged loops is counter-intuitive and is likely novel. Its
/// first known publication was in *2021* by *Control* from the *Holy Grail
/// Sort Project*." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Examples
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3 10  - 12 13  - 15  4 ~~~~~~~~~~~~ 9]  // swap
///   └─────┴─────/\────┴─────┘
///   ┌─────┬─────\~────┬─────┐
/// [13  - 15;10  - 12] 1 ~~~ 3  4  .  .  .  .  9   // swap
///   └─────┴/\┴─────┘
///   ┌─────┬~~┬─────┐
/// [10 ~~ 12 13 ~~ 15] 1 ~~~~~~~~~~~~~~~~~~~~~ 9
///
/// [10 ~~~~~~~~~~~ 15: 1  .  3* 4  .  .  .  .  9]
/// ```
///
/// ```text
///                         mid
///          left = 8       |      right = 7
/// [ 1  2  3  4  5  6  7: 8* 9 10 11 12 13 14 15]  // swap
///      └─────────────────┴/\┴─────────────────┘
///      ┌─────────────────┬\~┬─────────────────┐
/// [ 1  9  -  -  -  -  - 15  2 ~~~~~~~~~~~~~~~ 8]  // swap
///   └────────/\──────────┘
///   ┌────────\~──────────┐
/// [15; 9  -  -  -  - 14] 1 ~~~~~~~~~~~~~~~~~~ 8]  // AUX or any other rotation
///
/// [ 9 ~~~~~~~~~~~~~~ 15: 1* 2  .  .  .  .  .  8]
/// ```
pub unsafe fn ptr_helix_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    let mut start = mid.sub(left);
    let mut end = mid.add(right);

    loop {
        if left >= right {
            if right <= 1 {
                break;
            }

            if left == right {
                ptr::swap_nonoverlapping(mid.sub(left), mid, right);
                return;
            }

            swap_backward(start, end.sub(left), left);

            left %= right;
            end = end.sub(left);
            mid = start.add(left);
            right -= left;
        } else {
            if left <= 1 {
                break;
            }

            swap_forward(mid, start, right);

            start = start.add(right);
            right %= left;
            mid = end.sub(right);
            left -= right;
        }
    }

    if left == 1 || right == 1 {
        ptr_edge_rotate(left, mid, right);
    }
}

/// # Direct aka Juggling aka Dolphin rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at
/// `mid` becomes the first element. Equivalently, rotates the range
/// `left` elements to the left or `right` elements to the right.
///
/// ## Algorithm
///
/// This algorithm is extracted from current rotation implementation in Rust.
///
/// "In Rust this algorithm is used for small values of `left + right` or for large `T`. The elements
/// are moved into their final positions one at a time starting at `mid - left` and advancing by `right`
/// steps modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
/// `mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
/// elements."
///
/// "Fortunately, the number of skipped over elements between finalized elements is always equal, so
/// we can just offset our starting position and do more rounds (the total number of rounds is the
/// `gcd(left + right, right)` value). The end result is that all elements are finalized once and
/// only once."
///
/// Its first known publication was in *1966*.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]
///   |        |        |        |        └──────────────────╮
///   |        |        |        └──────────────────╮        ┆
///   |        |        └─────────────────┐         ┆        ┆
///   |        └─────────────────┐        ┆         ┆        ┆
///   └─────────────────┐        ┆        ┆         ┆        ┆
/// ~──────────╮        ┆        ┆        ┆         ┆        ┆
/// ~─╮        ┆        ┆        ┆        ┆         ┆        ┆
///   ↓        ↓        ↓        ↓        ↓         ↓        ↓
/// [10  2  3 13  5  6  1  8  9  4 11 12  7 14 15][10  2  3 13...
///      |        |        |        |        └──────────────────╮
///      |        |        |        └──────────────────╮        ┆
///      |        |        └─────────────────┐         ┆        ┆
///      |        └─────────────────┐        ┆         ┆        ┆
///      └─────────────────┐        ┆        ┆         ┆        ┆
/// ~─────────────╮        ┆        ┆        ┆         ┆        ┆
/// ~────╮        ┆        ┆        ┆        ┆         ┆        ┆
///   _  ↓     _  ↓     _  ↓     _  ↓     _  ↓      _  ↓     _  ↓
/// [10 11  3 13 14  6  1  2  9  4  5 12  7  8 15][10 11  3 13 14...
///         |        |        |        |        └──────────────────╮
///         |        |        |        └──────────────────╮        ┆
///         |        |        └─────────────────┐         ┆        ┆
///         |        └─────────────────┐        ┆         ┆        ┆
///         └─────────────────┐        ┆        ┆         ┆        ┆
/// ~────────────────╮        ┆        ┆        ┆         ┆        ┆
/// ~───────╮        ┆        ┆        ┆        ┆         ┆        ┆
///   _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓   _  _  ↓  _  _  ↓
/// [10  . 12  .  . 15: .  .  3* .  .  6  .  .  9][ .  . 12  .  . 15...
/// ```
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9* a  b  c  d  e  f]
///   └─────|  └─────|  └─────|  └─────|  └─────|
///  ~╮     └───────────╮     |        └────────────╮
///   |              |  |     └───────────╮     |   |
///  ~| ───────╮     └───────────╮        |     └───| ───────╮
///   ↓        ↓        ↓        ↓        ↓         ↓        ↓
/// [ a ~~~ c  d ~~~ f  1 ~~~ 3  4 ~~~ 6  7 ~~~ 9][ a ~~~ c  d ~~~ f...
/// ```
pub unsafe fn ptr_direct_rotate<T>(left: usize, mid: *mut T, right: usize) {
    // N.B. the below algorithms can fail if these cases are not checked
    if (right == 0) || (left == 0) {
        return;
    }

    if left == right {
        let start = mid.sub(left);
        ptr::swap_nonoverlapping(start, mid, left);
        return;
    }

    let start = mid.sub(left);

    // beginning of first round
    let mut tmp: T = start.read();
    let mut i = right;

    // `gcd` can be found before hand by calculating `gcd(left + right, right)`,
    // but it is faster to do one loop which calculates the gcd as a side effect, then
    // doing the rest of the chunk
    let mut gcd = right;

    // benchmarks reveal that it is faster to swap temporaries all the way through instead
    // of reading one temporary once, copying backwards, and then writing that temporary at
    // the very end. This is possibly due to the fact that swapping or replacing temporaries
    // uses only one memory address in the loop instead of needing to manage two.
    loop {
        tmp = start.add(i).replace(tmp);

        // instead of incrementing `i` and then checking if it is outside the bounds, we
        // check if `i` will go outside the bounds on the next increment. This prevents
        // any wrapping of pointers or `usize`.
        if i >= left {
            i -= left;
            if i == 0 {
                // end of first round
                start.write(tmp);
                break;
            }
            // this conditional must be here if `left + right >= 15`
            if i < gcd {
                gcd = i;
            }
        } else {
            i += right;
        }
    }

    // finish the chunk with more rounds
    for s in 1..gcd {
        tmp = start.add(s).read();
        i = s + right;

        loop {
            tmp = start.add(i).replace(tmp);
            if i >= left {
                i -= left;
                if i == s {
                    start.add(s).write(tmp);
                    break;
                }
            } else {
                i += right;
            }
        }
    }
}

/// # Contrev (Conjoined triple reversal) rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Algorithm
///
/// "The conjoined triple reversal is derived from the triple reversal rotation. Rather than three
/// separate reversals it conjoins the three reversals, improving locality and reducing
/// the number of moves. Its first known publication was in 2021 by Igor van den
/// Hoven." <<https://github.com/scandum/rotate>>
///
/// ## Example
///
/// Case: `right > left`, `9 - 6`.
///
/// ```text
///                            mid
///   ls-->               <--le|rs-->       <--re
/// [ 1  2  3  4  5  6: 7  8  9* a  b  c  d  e  f]  // (ls -> le -> re -> rs -> ls)
///   ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮ |
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆─╯
///   ↓  sl               le  |   | sr      re┆
/// [ a  2  .  .  .  .  .  8  1  f╯ b  .  .  e╰>9]  // (ls, le, re, rs)
///      ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮ |
///      ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆─╯
///      ↓  s           e  |        |  s  e┆
/// [ a  b  3  .  .  .  7  2  1  f  e  c  d╰>8  9]
///         ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈╭╯  |
///         ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|─╯
///         ↓  s     e  |              e┆
/// [ a ~~~ c  4  .  6  3  2  1  f  e  d╰>7 ~~~ 9]  // (ls, le, re    )
///            ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
///            ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
///            ↓  sl-|>         <--re┆
/// [ a ~~~~~~ d  5  4  3  2  1  f  e╰>6 ~~~~~~ 9]  // (ls,     re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ a ~~~~~~~~~ e  f: 1  2  3* 4  5 ~~~~~~~~~ 9]
/// ```
///
/// Case: `left < right`, `6 - 9`.
///
/// ```text
///                   mid
///   ls-->      <--le|rs-->                <--re
/// [ a  b  c  d  e  f* 1  2  3: 4  5  6  7  8  9]  // (ls -> le -> re -> rs -> ls)
///   | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
///   ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
///     ┆sl      le |   |  sr               re  ↓
/// [ 1<╯b  .  .  e ╰a  9  2  .  .  .  .  .  8  f]  // (ls, le, re, rs)
///      | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
///      ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
///        ┆s  e|          |  s           e  ↓
/// [ 1  2<╯c  d╰~b  a  9  8  3  .  .  .  7  e  f]
///         |╭ ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯     ╭─────╯
///         ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
///          ┆┆s              |  s     e  ↓
/// [ 1 ~~~ 3╯╰c  b  a  9  8  7  4  5  6  d ~~~ f]  // (ls,     re, rs)
///            ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
///             ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
///             ┆ sl-->         <|-re  ↓
/// [ 1 ~~~~~~ 4╯ b  a  9  8  7  6  5  c ~~~~~~ f]  // (ls,     re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ 1 ~~~~~~~~~ 5  6  7  8  9: a  b ~~~~~~~~~ f]
/// ```
///
/// Case: `left > right`, `8 - 7`.
///
/// ```text
///                         mid
///   ls-->            <--le|rs-->          <--re
/// [ 1  2  3  4  5  6  7: 8* a  b  c  d  e  f  g]  // (ls -> le -> re -> rs -> ls)
///   ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭──────────────┆─╯
///   ↓  sl            le  |   | sr         re┆
/// [ a  2  .  .  .  .  7  1  g╯ b  .  .  .  f╰>8]  // (ls, le, re, rs)
///      ╰────────╮     ╰┈┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈╮ |
///      ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈╯╭─────── ┆─╯
///      ↓  s        e  |         | s     e┆
/// [ a  b  3  .  .  6  2  1  g  f╯ c  .  e╰>7  8]  // (ls, le, re, rs)
///         ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮ |
///         ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆─╯
///         ↓  s  e  |               | e┆
/// [ a ~~~ c  4  5╮ 3  2  1  g  f  e╯ d╰>6 ~~~ 8]  // (ls, le,     rs)
///            ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮ |
///            ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆─╯
///            ↓  sl-->         <--re┆
/// [ a ~~~~~~ d  4  3  2  1  g  f  e╰>5 ~~~~~~ 8]  // (ls,     re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈ ╭┈ ╭ ╰┆┈┈╮ ┈╮ ┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ a ~~~~~~~~~ e  f  g: 1* 2  3  4 ~~~~~~~~~ 8]
/// ```
pub unsafe fn ptr_contrev_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if left == 0 || right == 0 {
        return;
    }

    if left == right {
        ptr::swap_nonoverlapping(mid, mid.sub(left), right);
    } else {
        let (mut ls, mut le) = (mid.sub(left), mid.sub(1));
        let (mut rs, mut re) = (mid, mid.add(right).sub(1));

        let half_min = cmp::min(left, right) / 2;
        let half_max = cmp::max(left, right) / 2;

        for _ in 0..half_min {
            // Permutation (ls, le, re, rs)
            ls.write(rs.replace(re.replace(le.replace(ls.read()))));
            ls = ls.add(1);
            le = le.sub(1);
            rs = rs.add(1);
            re = re.sub(1);
        }

        if left > right {
            for _ in 0..half_max - half_min {
                // (ls, le, re)
                ls.write(re.replace(le.replace(ls.read())));
                ls = ls.add(1);
                le = le.sub(1);
                re = re.sub(1);
            }
        } else {
            for _ in 0..half_max - half_min {
                // (rs, re, ls)
                ls.write(rs.replace(re.replace(ls.read())));
                ls = ls.add(1);
                rs = rs.add(1);
                re = re.sub(1);
            }
        }

        // for _ in 0..re.offset_from(ls).abs() / 2 { // (re, ls)
        // ls.write(
        // re.replace(ls.read())
        // );
        // ls = ls.add(1);
        // re = re.sub(1);
        // }

        let center = slice::from_raw_parts_mut(ls, re.offset_from(ls).unsigned_abs() + 1);
        center.reverse();
    }
}

// /// # Harmony rotation
// ///
// /// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
// /// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
// /// right.
// ///
// /// ## Safety
// ///
// /// The specified range must be valid for reading and writing.
// ///
// /// ## Algorithm
// ///
// /// `size_of(T) <= 1 * usize' case:
// ///
// /// Depending of the size:
// ///
// /// * For the array with `<= 14` elements (`size_of(T) <= 1 * usize') we use *direct rotation*;
// ///
// /// * `> 14` elements:
// /// ** `left < right` the *reversal rotation is used*;
// /// ** otherwise, *direct rotation*.
// ///
// /// * `> 20` elements we use *reversal rotation*.
// ///
// /// *Algorithm 1* (*Direct*) is used for small values of `left + right` or for large `T`. The elements
// /// are moved into their final positions one at a time starting at `mid - left` and advancing by `right`
// /// steps modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
// /// `mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
// /// elements. For example:
// ///
// /// *Algorithm 2* (*AUX*) is used if `left + right` is large but `min(left, right)` is small enough to
// /// fit onto a stack buffer. The `min(left, right)` elements are copied onto the buffer, `memmove`
// /// is applied to the others, and the ones on the buffer are moved back into the hole on the
// /// opposite side of where they originated.
// ///
// /// Algorithms that can be vectorized outperform the above once `left + right` becomes large enough.
// /// *Algorithm 1* can be vectorized by chunking and performing many rounds at once, but there are too
// /// few rounds on average until `left + right` is enormous, and the worst case of a single
// /// round is always there. Instead, *algorithm 3* (*GM*) utilizes repeated swapping of
// /// `min(left, right)` elements until a smaller rotate problem is left.
// ///
// /// ```text
// ///                                   mid
// ///              left = 11            | right = 4
// /// [ 5  6  7  8: 9 10 11 12 13 14 15 "1  2  3  4]   swap
// ///                        └────────┴/\┴────────┘
// ///                        ┌────────┬~~┬────────┐
// /// [ 5  .  .  .  .  . 11  1 ~~~~~~ 4 12 13 14 15]
// ///
// /// [ 5  .  7  1  2  3  4  8  9 10 11 12 ~~~~~ 15    swap
// ///            └────────┴/\┴────────┘
// ///            ┌────────┬~~┬────────┐
// /// [ 5  .  7  8: 9  . 11: 1 ~~~~~~ 4"12  .  . 15
// /// we cannot swap any more, but a smaller rotation problem is left to solve
// /// ```
// ///
// /// when `left < right` the swapping happens from the left instead.
// pub unsafe fn ptr_harmony_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
//     type BufType = [usize; 32];

//     // if T::IS_ZST {
//     // return;
//     // }

//     let t_size = std::mem::size_of::<T>();

//     loop {
//         if (right == 0) || (left == 0) {
//             return;
//         }

//         if left == right {
//             let start = mid.sub(left);
//             ptr::swap_nonoverlapping(start, mid, left);
//         }

//         let size = left + right;

//         if t_size <= std::mem::size_of::<usize>() {
//             if size <= 14 {
//                 ptr_direct_rotate(left, mid, right);
//             } else if size <= 24 {
//                 if left < right {
//                     ptr_reversal_rotate(left, mid, right);
//                 } else {
//                     ptr_direct_rotate(left, mid, right);
//                 }
//             } else if size < 40 {
//                 ptr_reversal_rotate(left, mid, right);
//             }
//         } else {
//         }
//     }
// }

/// # Default (Stable) rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Algorithm
///
/// *Algorithm 1* (*Direct*) is used for small values of `left + right` or for large `T`. The elements
/// are moved into their final positions one at a time starting at `mid - left` and advancing by `right`
/// steps modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
/// `mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
/// elements. For example:
///
/// ```text
///                            mid
///           left = 9         |    right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]                      // round
///   └─────────────────┐
/// [ ✘  2  .  .  .  6  1  8  .  .  .  .  .  . 15] [ 7]
///                                       ┌──────────┘
///                     _                 ↓
/// [ ✘  2  .  .  .  6  1  8  .  .  . 12  7 14 15] [13]
///            ┌─────────────────────────────────────┘
///            ↓        _                 _
/// [ ✘  2  3 13  5  6  1  8  .  .  . 12  7 14 15] [ 4]
///                              ┌───────────────────┘
///            _        _        ↓        _
/// [ ✘  2  3 13  5  6  1  8  9  4 11 12  7 14 15] [10]
///   ┌──────────────────────────────────────────────┘
///   ↓        _        _        _        _
/// [10  2  3 13  5  6  1  8  9  4 11 12  7 14 15]                      // round
///      |        |        |        |        └──────────────────╮
///      |        |        |        └──────────────────╮        ┆
///      |        |        └─────────────────┐         ┆        ┆
///      |        └─────────────────┐        ┆         ┆        ┆
///      └─────────────────┐        ┆        ┆         ┆        ┆
/// ~─────────────╮        ┆        ┆        ┆         ┆        ┆
/// ~────╮        ┆        ┆        ┆        ┆         ┆        ┆
///   _  ↓     _  ↓     _  ↓     _  ↓     _  ↓      _  ↓     _  ↓
/// [10 11  3 13 14  6  1  2  9  4  5 12  7  8 15][10 11  3 13 14...    // round
///         |        |        |        |        └──────────────────╮
///         |        |        |        └──────────────────╮        ┆
///         |        |        └─────────────────┐         ┆        ┆
///         |        └─────────────────┐        ┆         ┆        ┆
///         └─────────────────┐        ┆        ┆         ┆        ┆
/// ~────────────────╮        ┆        ┆        ┆         ┆        ┆
/// ~───────╮        ┆        ┆        ┆        ┆         ┆        ┆
///   _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓  _  _  ↓   _  _  ↓  _  _  ↓
/// [10 11 12 13 14 15: 1  2  3* 4  5  6  7  8  9][10 11 12 13 14 15...
/// ```
///
/// Fortunately, the number of skipped over elements between finalized elements is always equal, so
/// we can just offset our starting position and do more rounds (the total number of rounds is the
/// `gcd(left + right, right)` value). The end result is that all elements are finalized once and
/// only once.
///
/// *Algorithm 2* (*AUX*) is used if `left + right` is large but `min(left, right)` is small enough to
/// fit onto a stack buffer. The `min(left, right)` elements are copied onto the buffer, `memmove`
/// is applied to the others, and the ones on the buffer are moved back into the hole on the
/// opposite side of where they originated.
///
/// Algorithms that can be vectorized outperform the above once `left + right` becomes large enough.
/// *Algorithm 1* can be vectorized by chunking and performing many rounds at once, but there are too
/// few rounds on average until `left + right` is enormous, and the worst case of a single
/// round is always there. Instead, *algorithm 3* (*GM*) utilizes repeated swapping of
/// `min(left, right)` elements until a smaller rotate problem is left.
///
/// ```text
///                                   mid
///              left = 11            | right = 4
/// [ 5  6  7  8: 9 10 11 12 13 14 15 "1  2  3  4]   swap
///                        └────────┴/\┴────────┘
///                        ┌────────┬~~┬────────┐
/// [ 5  .  .  .  .  . 11  1 ~~~~~~ 4 12 13 14 15]
///
/// [ 5  .  7  1  2  3  4  8  9 10 11 12 ~~~~~ 15    swap
///            └────────┴/\┴────────┘
///            ┌────────┬~~┬────────┐
/// [ 5  .  7  8: 9  . 11: 1 ~~~~~~ 4"12  .  . 15
/// we cannot swap any more, but a smaller rotation problem is left to solve
/// ```
///
/// when `left < right` the swapping happens from the left instead.
pub unsafe fn stable_ptr_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    //Taken from https://github.com/rust-lang/rust/blob/11d96b59307b1702fffe871bfc2d0145d070881e/library/core/src/slice/rotate.rs .

    type BufType = [usize; 32];

    // if T::IS_ZST {
    // return;
    // }

    loop {
        // N.B. the below algorithms can fail if these cases are not checked
        if (right == 0) || (left == 0) {
            return;
        }

        if (left + right < 24) || (std::mem::size_of::<T>() > std::mem::size_of::<[usize; 4]>()) {
            // Algorithm 1
            // Microbenchmarks indicate that the average performance for random shifts is better all
            // the way until about `left + right == 32`, but the worst case performance breaks even
            // around 16. 24 was chosen as middle ground. If the size of `T` is larger than 4
            // `usize`s, this algorithm also outperforms other algorithms.
            // SAFETY: callers must ensure `mid - left` is valid for reading and writing.
            let x = unsafe { mid.sub(left) };
            // beginning of first round
            // SAFETY: see previous comment.
            let mut tmp: T = unsafe { x.read() };
            let mut i = right;
            // `gcd` can be found before hand by calculating `gcd(left + right, right)`,
            // but it is faster to do one loop which calculates the gcd as a side effect, then
            // doing the rest of the chunk
            let mut gcd = right;
            // benchmarks reveal that it is faster to swap temporaries all the way through instead
            // of reading one temporary once, copying backwards, and then writing that temporary at
            // the very end. This is possibly due to the fact that swapping or replacing temporaries
            // uses only one memory address in the loop instead of needing to manage two.
            loop {
                // [long-safety-expl]
                // SAFETY: callers must ensure `[left, left+mid+right)` are all valid for reading and
                // writing.
                //
                // - `i` start with `right` so `mid-left <= x+i = x+right = mid-left+right < mid+right`
                // - `i <= left+right-1` is always true
                //   - if `i < left`, `right` is added so `i < left+right` and on the next
                //     iteration `left` is removed from `i` so it doesn't go further
                //   - if `i >= left`, `left` is removed immediately and so it doesn't go further.
                // - overflows cannot happen for `i` since the function's safety contract ask for
                //   `mid+right-1 = x+left+right` to be valid for writing
                // - underflows cannot happen because `i` must be bigger or equal to `left` for
                //   a subtraction of `left` to happen.
                //
                // So `x+i` is valid for reading and writing if the caller respected the contract
                tmp = unsafe { x.add(i).replace(tmp) };
                // instead of incrementing `i` and then checking if it is outside the bounds, we
                // check if `i` will go outside the bounds on the next increment. This prevents
                // any wrapping of pointers or `usize`.
                if i >= left {
                    i -= left;
                    if i == 0 {
                        // end of first round
                        // SAFETY: tmp has been read from a valid source and x is valid for writing
                        // according to the caller.
                        unsafe { x.write(tmp) };
                        break;
                    }
                    // this conditional must be here if `left + right >= 15`
                    if i < gcd {
                        gcd = i;
                    }
                } else {
                    i += right;
                }
            }
            // finish the chunk with more rounds
            for start in 1..gcd {
                // SAFETY: `gcd` is at most equal to `right` so all values in `1..gcd` are valid for
                // reading and writing as per the function's safety contract, see [long-safety-expl]
                // above
                tmp = unsafe { x.add(start).read() };
                // [safety-expl-addition]
                //
                // Here `start < gcd` so `start < right` so `i < right+right`: `right` being the
                // greatest common divisor of `(left+right, right)` means that `left = right` so
                // `i < left+right` so `x+i = mid-left+i` is always valid for reading and writing
                // according to the function's safety contract.
                i = start + right;
                loop {
                    // SAFETY: see [long-safety-expl] and [safety-expl-addition]
                    tmp = unsafe { x.add(i).replace(tmp) };
                    if i >= left {
                        i -= left;
                        if i == start {
                            // SAFETY: see [long-safety-expl] and [safety-expl-addition]
                            unsafe { x.add(start).write(tmp) };
                            break;
                        }
                    } else {
                        i += right;
                    }
                }
            }
            return;
        // `T` is not a zero-sized type, so it's okay to divide by its size.
        } else if cmp::min(left, right) <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>()
        {
            // Algorithm 2
            // The `[T; 0]` here is to ensure this is appropriately aligned for T
            let mut rawarray = MaybeUninit::<(BufType, [T; 0])>::uninit();
            let buf = rawarray.as_mut_ptr() as *mut T;
            // SAFETY: `mid-left <= mid-left+right < mid+right`
            let dim = unsafe { mid.sub(left).add(right) };
            if left <= right {
                // SAFETY:
                //
                // 1) The `else if` condition about the sizes ensures `[mid-left; left]` will fit in
                //    `buf` without overflow and `buf` was created just above and so cannot be
                //    overlapped with any value of `[mid-left; left]`
                // 2) [mid-left, mid+right) are all valid for reading and writing and we don't care
                //    about overlaps here.
                // 3) The `if` condition about `left <= right` ensures writing `left` elements to
                //    `dim = mid-left+right` is valid because:
                //    - `buf` is valid and `left` elements were written in it in 1)
                //    - `dim+left = mid-left+right+left = mid+right` and we write `[dim, dim+left)`
                unsafe {
                    // 1)
                    ptr::copy_nonoverlapping(mid.sub(left), buf, left);
                    // 2)
                    ptr::copy(mid, mid.sub(left), right);
                    // 3)
                    ptr::copy_nonoverlapping(buf, dim, left);
                }
            } else {
                // SAFETY: same reasoning as above but with `left` and `right` reversed
                unsafe {
                    ptr::copy_nonoverlapping(mid, buf, right);
                    ptr::copy(mid.sub(left), dim, left);
                    ptr::copy_nonoverlapping(buf, mid.sub(left), right);
                }
            }
            return;
        } else if left >= right {
            // Algorithm 3
            //
            //           left = 9         mid    right = 6
            // [ 1  2  3  4  5  6  7  8  9,10 11 12 13 14 15]
            //            └──────────────┴──┬──────────────┐
            //    l = 3  mid                |    r = 6     |
            // [ 1  2  3,10 11 12 13 14 15  4  5  6  7  8  9]
            //
            // There is an alternate way of swapping that involves finding where the last swap
            // of this algorithm would be, and swapping using that last chunk instead of swapping
            // adjacent chunks like this algorithm is doing, but this way is still faster.
            loop {
                // SAFETY:
                // `left >= right` so `[mid-right, mid+right)` is valid for reading and writing
                // Subtracting `right` from `mid` each turn is counterbalanced by the addition and
                // check after it.
                unsafe {
                    ptr::swap_nonoverlapping(mid.sub(right), mid, right);
                    mid = mid.sub(right);
                }
                left -= right;
                if left < right {
                    break;
                }
            }
        } else {
            // Algorithm 3, `left < right`
            //
            //  left = 3 mid                    right = 6
            // [ 1  2  3,10 11 12 13 14 15 ,4  5  6  7  8  9]
            //   └─────┴──┬─────┐
            //   l = 3    |     | mid                r = 3
            // [10 11 12  1  2  3,13 14 15  4  5  6, 7  8  9]
            //   l = 3    └─────┴──┬─────┐ mid                r = 0
            // [10 11 12 13 14 15  1  2  3 ,4  5  6  7  8  9],
            loop {
                // SAFETY: `[mid-left, mid+left)` is valid for reading and writing because
                // `left < right` so `mid+left < mid+right`.
                // Adding `left` to `mid` each turn is counterbalanced by the subtraction and check
                // after it.
                unsafe {
                    ptr::swap_nonoverlapping(mid.sub(left), mid, left);
                    mid = mid.add(left);
                }
                right -= left;
                if right < left {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn div(s: usize, diff: usize) -> (usize, usize) {
        assert!(s >= diff);
        assert!(s % 2 == diff % 2);

        let r = s / 2 - diff / 2;

        (s - r, r)
    }

    fn seq(size: usize) -> Vec<usize> {
        let mut v = vec![0; size];
        for i in 0..size {
            v[i] = i + 1;
        }
        v
    }

    fn prepare(size: usize, diff: usize) -> (Vec<usize>, (usize, *mut usize, usize)) {
        let (l, r) = div(size, diff);
        let mut v = seq(size);

        unsafe {
            let p = &v[..].as_mut_ptr().add(l);
            (v, (l, p.clone(), r))
        }
    }

    fn case(
        rotate: unsafe fn(left: usize, mid: *mut usize, right: usize),
        size: usize,
        diff: usize,
    ) {
        let (vec, (l, p, r)) = prepare(size, diff);

        let mut s = seq(size);

        s.rotate_left(l);
        unsafe { rotate(l, p, r) };

        assert_eq!(vec, s);

        unsafe { rotate(r, p.sub(diff), l) };

        s.rotate_right(l);
        assert_eq!(vec, s);
    }

    fn test_correct(rotate_f: unsafe fn(left: usize, mid: *mut usize, right: usize)) {
        // --empty--
        case(rotate_f, 0, 0);

        // --empty--
        case(rotate_f, 2, 0);

        // 1  2  3  4  5  6 (7  8  9)10 11 12 13 14 15
        case(rotate_f, 15, 3);

        // 1  2  3  4  5 (6  7  8  9 10)11 12 13 14 15
        case(rotate_f, 15, 5);

        // 1  2  3  4  5  6  7 (8) 9 10 11 12 13 14 15
        case(rotate_f, 15, 1);

        // 1  2  3  4  5  6  7)(8  9 10 11 12 13 14
        case(rotate_f, 14, 0);

        // 1  2  3  4  5 (6  7  8  9 10)11 12 13 14 15
        case(rotate_f, 15, 5);

        // 1  2  3  4 (5  6  7  8  9 10 11)12 13 14 15
        case(rotate_f, 15, 7);

        // 1 (2  3  4  5  6  7  8  9 10 11 12 13 14)15
        case(rotate_f, 15, 13);

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        case(rotate_f, 15, 15);

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        case(rotate_f, 100_000, 0);
    }

    #[test]
    // default (stable) rust rotate
    fn ptr_rotate_correct() {
        test_correct(stable_ptr_rotate::<usize>);
    }

    // #[test]
    // fn ptr_harmony_rotate_correct() {
    //     test_correct(ptr_harmony_rotate::<usize>);
    // }

    #[test]
    fn ptr_edge_rotate_correct() {
        test_correct(ptr_edge_rotate::<usize>);
    }

    #[test]
    fn ptr_reversal_rotate_correct() {
        test_correct(ptr_reversal_rotate::<usize>);
    }

    #[test]
    fn ptr_block_reversal_rotate_correct() {
        test_correct(ptr_block_reversal_rotate::<usize>);
    }

    #[test]
    fn ptr_piston_rotate_rec_correct() {
        test_correct(ptr_piston_rotate_rec::<usize>);
    }

    #[test]
    fn ptr_piston_rotate_correct() {
        test_correct(ptr_piston_rotate::<usize>);
    }

    #[test]
    fn ptr_contrev_rotate_correct() {
        test_correct(ptr_contrev_rotate::<usize>);
    }

    #[test]
    fn ptr_gen_contrev_rotate_correct() {
        test_correct(ptr_block_contrev_rotate::<usize>);
    }

    #[test]
    fn ptr_direct_rotate_correct() {
        test_correct(ptr_direct_rotate::<usize>);
    }

    #[test]
    fn ptr_helix_rotate_correct() {
        test_correct(ptr_helix_rotate::<usize>);
    }
}
