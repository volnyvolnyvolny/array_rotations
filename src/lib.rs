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
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#![doc = include_str!("../README.md")]
//#![feature(sized_type_properties)]

use std::mem::MaybeUninit;
//use std::mem::SizedTypeProperties;

use std::cmp;

use std::ptr;
use std::slice;

use gcd::Gcd;

mod utils;
pub use utils::*;

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
    // if T::IS_ZST {
    // return;
    // }

    unsafe fn reverse_slice<T>(p: *mut T, size: usize) {
        let slice = slice::from_raw_parts_mut(p, size);
        slice.reverse();
    }

    reverse_slice(mid.sub(left), left);
    reverse_slice(mid, right);
    reverse_slice(mid.sub(left), left + right);
}

/// # Gries-Mills rotation (recursive variant)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes
/// the first element. Equivalently, rotates the range `left` elements to the left
/// or `right` elements to the right.
///
/// ## Algorithm
///
/// 1. Swap the shadow to its place;
/// 2. rotate smaller array.
///
/// "You swap the smallest array linearly towards its proper location,
/// since the blocks behind it are in the proper location you can forget about them.
/// What remains of the larger array is now the smallest array, which you rotate in
/// a similar manner, until the smallest side shrinks to `0` elements. Its first known
/// publication was in *1981* by *David Gries* and *Harlan Mills*."
/// <<https://github.com/scandum/rotate>>
///
/// ## Performance
///
/// "In some cases this rotation outperforms the classic *Triple reversal rotation*
/// while making fewer moves." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                  𝑠ℎ𝑎𝑑𝑜𝑤    mid
///           left = 9         |     right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap r-side and shadow
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3 10  .  .  .  . 15  4 ~~~~~~~~~~~~ 9]
///
///    l = 3     𝑠ℎ. r = 6
/// [ 1  .  3,10  . 12:13  . 15] 4  .  .  .  .  9   // swap new l-side and new shadow
///   └─────┴/\┴─────┘
///   ┌─────┬~/┬─────┐
/// [10 ~~ 12  1  .  3 13  . 15] 4  .  .  .  .  9
///
///             l = 3   r = 3
///  10 ~~ 12[ 1  .  3;13  . 15] 4  .  .  .  .  9   // swap equal
///            └─────┴/\┴─────┘
///            ┌─────┬~~┬─────┐
///  10 ~~ 12[13 ~~ 15  1 ~~~ 3] 4  .  .  .  .  9
///
/// [10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
/// ```
pub unsafe fn ptr_griesmills_rotate_rec<T>(left: usize, mid: *mut T, right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    if (right == 0) || (left == 0) {
        return;
    }

    if left < right {
        ptr::swap_nonoverlapping(mid.sub(left), mid, left);
        ptr_griesmills_rotate_rec(left, mid.add(left), right - left);
    } else {
        ptr::swap_nonoverlapping(mid, mid.sub(right), right);
        ptr_griesmills_rotate_rec(left - right, mid.sub(right), right);
    }
}

/// # Grail rotation (Gries-Mills rotation + *swap_backward*)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "The *Grail rotation* from the *Holy Grail Sort Project* is *Gries-Mills* derived
/// and tries to improve locality by shifting memory either left or right depending on which
/// side it's swapped from.
///
/// In addition it performs an auxiliary rotation on stack memory when the smallest side reaches
/// a size of `1` element, which is the worst case for the *Gries-Mills rotation*. The flow diagram
/// is identical to that of *Gries-Mills*, but due to memory being shifted from the right the
/// visualization differs."
/// <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Examples
///
/// ```text
///                            mid
///           left = 9         |     right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap <--
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3;10  . 12 13  . 15] 4 ~~~~~~~~~~~~ 9   // swap <--
///   └─────┴/\┴─────┘
///   ┌─────┬~/┬─────┐
/// [10 ~~ 12  1  .  3 13  . 15] 4  .  .  .  .  9   // swap -->
///            └─────┴/\┴─────┘
///            ┌─────┬~~┬─────┐
///  10  . 12[13 ~~ 15  1 ~~~ 3] 4  .  .  .  .  9
///
/// [10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_grail_rotate<T>(mut left: usize, mid: *mut T, mut right: usize) {
    let mut min = cmp::min(left, right);
    let mut start = mid.sub(left);

    while min > 0 {
        if left <= right {
            loop {
                swap_forward(start, start.add(left), left);

                start = start.add(left);
                right -= left;

                if left > right {
                    break;
                }
            }

            min = right;
        } else {
            loop {
                swap_backward(start.add(left - right), start.add(left), right);

                left -= right;

                if right > left {
                    break;
                }
            }

            min = left;
        }
    }

    // if min > 0 { // min = 0, 1
    //     ptr_aux_rotate(left, start.add(left), right);
    // }
}

/// # Drill rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "The drill rotation is a grail variant that utilizes a piston main loop
/// and a helix inner loop. Performance is similar to the helix rotation.
/// The flow diagram and visualization are identical to the grail rotation."
///
/// *2021* - *Drill rotation* by *Igor van den Hoven* (*Grail* derived with *Piston*
/// and *Helix* loops)
/// <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Examples
///
/// ```text
///                            mid
///           left = 9         |     right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap <--
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3;10  .  .  .  . 15] 4 ~~~~~~~~~~~~ 9   // swap -->
///   └─────┴/\┴─────────────┘
///    ┌─────────────┬~~┬─────┐
/// [ 10 ~~~~~~~~~~ 15  1 ~~~ 3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_drill_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    let mut start = mid.sub(left);
    let mut end = mid.add(right);
    let mut s;

    while left > 0 {
        if left <= right {
            // -->
            let old_r = right;
            right %= left;

            s = old_r - right;

            //            swap_forward(start, mid, s);

            for _ in 0..s {
                mid.swap(start);

                mid = mid.add(1);
                start = start.add(1);
            }
        }

        // <--
        if right < 1 {
            break;
        }

        let old_l = left;
        left %= right;

        s = old_l - left;
        // swap_backward(end, mid, s);

        for _ in 0..s {
            mid = mid.sub(1);
            end = end.sub(1);

            mid.swap(end);
        }
    }

    // if left > 0 && right > 0 {
    //     ptr_aux_rotate(left, mid, right);
    // }
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
/// 2. repeat with smaller array.
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
/// [10 ~~~~~~~~~~~ 15: 7  .  9  1  .  .  .  .  6]
///
///                      l = 3        r = 6
///  10  .  .  .  . 15[ 7  .  9* 1  .  3: 4  .  6]  // swap
///                     └─────┴────/\─────┴─────┘
///                     ┌─────┬────\~─────┬─────┐
///  10  .  .  .  . 15[ 4  .  6  1  .  3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10  .  .  .  . 15[ 4  .  6; 1  .  3] 7 ~~~ 9   // swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬~~┬─────┐
///  10  .  .  .  . 15[ 1 ~~~ 3  4 ~~~ 6] 7 ~~~ 9
///
/// [10  .  .  .  . 15: 1 ~~~ 3* 4 ~~~~~~~~~~~~ 9]
/// ```
pub unsafe fn ptr_piston_rotate_rec<T>(left: usize, mid: *mut T, right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    if (right == 0) || (left == 0) {
        return;
    }

    if left < right {
        ptr::swap_nonoverlapping(mid.sub(left), mid.add(right).sub(left), left);
        ptr_piston_rotate_rec(left, mid, right - left);
    } else {
        ptr::swap_nonoverlapping(mid, mid.sub(left), right);
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
/// 2. repeat with smaller array.
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
/// [ 1  2  3  4  5  6: 7  8  9"10 11 12 13 14 15]  // swap
///   └──────────────┴─────/\────┴──────────────┘
///   ┌──────────────┬─────~/────┬──────────────┐
/// [10 ~~~~~~~~~~~ 15: 7  .  9  1  .  .  .  .  6]
///
///                       l = 3        r = 6
///  10  .  .  .  . 15[ 7  .  9" 1  .  3: 4  .  6]  // swap
///                     └─────┴─────/\────┴─────┘
///                     ┌─────┬─────\~────┬─────┐
///  10  .  .  .  . 15[ 4  .  6  1  .  3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10  .  .  .  . 15[ 4  .  6; 1  .  3] 7 ~~~ 9   // swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬~~┬─────┐
///  10  .  .  .  . 15[ 1 ~~~ 3  4 ~~~ 6] 7 ~~~ 9
///
/// [10  .  .  .  . 15: 1 ~~~ 3" 4 ~~~~~~~~~~~~ 9]
/// ```
pub unsafe fn ptr_piston_rotate<T>(left: usize, mid: *mut T, right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    let mut l = left as isize;
    let mut r = right as isize;

    loop {
        if l <= 0 {
            return;
        }

        while l <= r {
            ptr::swap_nonoverlapping(mid.offset(-l), mid.offset(r - l), l as usize);
            r -= l;
        }

        if r <= 0 {
            return;
        }

        while l >= r {
            ptr::swap_nonoverlapping(mid, mid.offset(-l), r as usize);
            l -= r;
        }
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
/// [ 1  .  3 10  . 12 13  . 15  4 ~~~~~~~~~~~~ 9]  // swap
///   └─────┴─────/\────┴─────┘
///   ┌─────┬─────\~────┬─────┐
/// [13  . 15;10  . 12] 1 ~~~ 3  4  .  .  .  .  9   // swap
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
/// [ 1  9  .  .  .  .  . 15  2 ~~~~~~~~~~~~~~~ 8]  // swap
///   └────────/\──────────┘
///   ┌────────\~──────────┐
/// [15; 9  .  .  .  . 14] 1 ~~~~~~~~~~~~~~~~~~ 8]  // aux rotate
///
/// [ 9 ~~~~~~~~~~~~~~ 15: 1* 2  .  .  .  .  .  8]
/// ```
pub unsafe fn ptr_helix_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    if (right == 0) || (left == 0) {
        return;
    }

    let mut start = mid.sub(left);
    let mut end = mid.add(right);

    loop {
        if left > right {
            if right <= 1 {
                break;
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

    if left > 0 && right > 0 {
        // left = 0, 1; right = 0, 1
        ptr_direct_rotate(left, mid, right);
    }
}

/// # Auxiliary rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "This is an easy and fast way to rotate, but since it requires
/// auxiliary memory it is of little interest to in-place algorithms.
/// It's a good strategy for array sizes of `1000` elements or less.
/// The smaller half is copied to swap memory, the larger half is moved,
/// and the swap memory is copied back to the main array." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                            mid
///        left = 9    dim     |        right = 6
/// [ 1  2  3  4  5  6 :7  8  9*            10-15]                // move
///                                          └──┴───────┬─────┐
/// [              1-6 :7 ... 9  ✘  ✘  ✘  ✘  ✘  ✘]    [10 .. 15]  // move
///                └────┬─────┴─────────────────┐
/// [ ✘  ✘  ✘  ✘  ✘  ✘ :1 ~~~~~~~~~~~~~~~~~~~~~ 9]    [10-15   ]  // move
///   ┌──────────────┬──────────────────────────────────┴──┘
/// [10 ~~~~~~~~~~~ 15 :1  .  3* 4  .  .  .  .  9]
/// ```
///
/// ```text
///                                  mid
///           left = 11              | right = 4
/// [ 1  2  3  4: 5  6  7  8  9 10 11*      12-15]                // move
///                                          └──┴───────┬─────┐
/// [ 1  .  .  .  .  .  .  .  .  . 11  ✘  ✘  ✘  ✘]    [12 .. 15]  // move
///   └───────────┬─────────────────┴───────────┐
/// [ ✘  ✘  ✘  ✘  1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11]    [12-15   ]  // move
///   ┌────────┬────────────────────────────────────────┴──┘
/// [12 ~~~~~ 15: 1  .  .  .  .  .  7* 8  .  . 11]
/// ```
///
/// ```text
///             mid
///    left = 4 |           right = 11
/// [      12-15* 1  2  3  4  5  6  7: 8  9 10 11]                // move
///         └──┴────────────────────────────────────────┬─────┐
/// [ ✘  ✘  ✘  ✘  1  .  .  .  .  .  .  .  .  . 11]    [12 .. 15]  // move
///   ┌───────────┴─────────────────┬───────────┘
/// [ 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11  .  .  .  .]    [12-15   ]  // move
///                                    ┌────────┬───────┴──┘
/// [ 1  .  .  4* 5  .  .  .  .  . 11:12 ~~~~~ 15]
/// ```
pub unsafe fn ptr_aux_rotate<T>(left: usize, mid: *mut T, right: usize, buffer: &mut [T]) {
    // if T::IS_ZST {
    // return;
    // }

    if (right == 0) || (left == 0) {
        return;
    }

    let start = mid.sub(left);
    let buf = buffer.as_mut_ptr();
    let dim = start.add(right);

    if left < right {
        // if right == left * 2 {
        //     ptr::swap_nonoverlapping(start, mid, left);
        //     ptr::swap_nonoverlapping(start, mid.add(left), left);
        // } else {
        ptr::copy_nonoverlapping(mid, buf, right);
        ptr::copy(start, dim, left);
        ptr::copy_nonoverlapping(buf, start, right);
        // }
    } else if right < left {
        // if left == right * 2 {
        //     ptr::swap_nonoverlapping(start.add(right), mid, right);
        //     ptr::swap_nonoverlapping(start, start.add(right), right);
        // } else {
        ptr::copy_nonoverlapping(start, buf, left);
        ptr::copy(mid, start, right);
        ptr::copy_nonoverlapping(buf, dim, left);
        // }
    } else {
        ptr::swap_nonoverlapping(start, mid, left);
    }
}

/// # Raft rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 1. Move first *buffer size* elements from bridge to buffer, creating the vacant space;
/// 2.a Let *a* be the first position from the l-side to be moved;
/// 2.b let *b* be the first vacant position;
/// 2.c let *c* be the first pos. from the r-side to be moved.
/// 3. *a* goes to *b*, *c* goes to *a*;
/// 4. increment *a*, *b*, *c* and repeat step **2** while *b* is vacant;
/// 5. Fill the vacant positions with buffer elements.
/// 6. Let *new bridge* be the elements *[b, c)*, repeat **1**.
///
/// 2023 -- by Valentin Vasilev
///
/// ## Properties
///
/// 1. If bridge = buffer -- it's a classical *Bridge* rotation.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// # Examples:
///
/// Buffer size 3 (bridge = 5):
///
/// ```text
///                dim            mid
///     left = 10  |    bridge    |   right = 5
/// [ 1  2  3  4  5: 6-8      9 10*11 12 13 14 15]                // 1 round
///                  └─┴───────────────────────────────┬─────┐
///   a-->           b-->           c-->               |     |
/// [ 1  .  .  .  5: ✘  ✘  ✘  9 10*11  .  .  . 15]    [6  7  8]
///   └──────────────┐              |
///   ╭───────────── ┆──────────────╯
///   ↓  a           ↓  b              c
/// [11  2  .  .  5  1  ✘  ✘  9 10  ✘ 12  .  . 15]    [6  .  8]
///      └──────────────┐              |
///      ╭───────────── ┆──────────────╯
///   _  ↓  a        _  ↓  b              c
/// [11 12  3  .  5  1  2  ✘  9 10  ✘  ✘ 13  . 15]    [6  .  8]
///         └──────────────┐              |
///         ╭───────────── ┆──────────────╯
///   _  _  ↓        _  _  ↓
/// [11  . 13  4  5  1  .  3  9 10  ✘  ✘  ✘ 14 15]    [6-8    ]
///   _  _  _        _  _  _        ┌─────┬────────────┴─┘
/// [11  . 13  4  5  1  .  3  9 10  6  7  8 14 15]    [✘  ✘  ✘]   // 2 round
///                           " new bridge"
///                           └──┴─────────────────────┬──┐
///   _  _  _  a-->  _  _  _  b-->  _  _  _  c-->      |  |
/// [11  . 13  4  5  1  .  3  ✘  ✘  6  .  8 14 15]    [9 10]
///            └──────────────┐              |
///            ╭───────────── ┆──────────────╯
///   _  _  _  ↓  a  _  _  _  ↓  b  _  _  _     c
/// [11  .  . 14  5  1  .  .  4  ✘  6  .  8  ✘ 15]    [9 10]
///               └──────────────┐              |
///               ╭───────────── ┆──────────────╯
///   _  _  _  _  ↓  _  _  _  _  ↓  _  _  _
/// [11  .  .  . 15  1  .  .  .  5  .  .  8  ✘  ✘]    [9 10]
///   _  _  _  _  _  _  _  _  _  _  _  _  _  ┌──┬──────┴──┘
/// [11  .  .  . 15: 1  .  .  .  .  .  .  8  9 10]
/// ```
///
/// and backward:
///
/// ```text
///                mid            dim
///     left = 5   |    bridge    |   right = 10
/// [11 12 13 14 15* 1  2      3-5: 6  7  8  9 10]                // 1 round
///                            └─┴─────────────────────┬─────┐
///   a-->           b-->           c-->               |     |
/// [11  .  .  . 15* 1  2  ✘  ✘  ✘: 6  .  .  . 10]    [3  4  5]
///               |              ┌──────────────┘
///               ╰──────────────┆ ─────────────╮
///                              ↓              ↓
/// [11  .  . 14  ✘  1  2  ✘  ✘ 10  6  .  .  9 15]    [3  4  5]
///            |              ┌──────────────┘
///            ╰──────────────┆ ─────────────╮
///                           ↓  _           ↓  _
/// [11  . 13  ✘  ✘  1  2  ✘  9 10  6  .  8 14 15]    [3  4  5]
///         |              ┌──────────────┘
///         ╰──────────────┆ ─────────────╮
///                        ↓  _  _        ↓  _  _
/// [11 12  ✘  ✘  ✘  1  2  8  9 10  6  7 13 14 15]    [3-5    ]
///         ┌─────┬────────────────────────────────────┴─┘
/// [11 12  3  4  5  1  2  8  9 10  6  7 13 14 15]    [✘  ✘  ✘]   // 2 round
///        "  new bridge "
///                  └──┴──────────────────────────────┬──┐
///         _  a-->        _  b-->        _  c-->      |  |
/// [11 12  3  4  5  ✘  ✘  8  9 10  6  7 13 14 15]    [1  2]
///      |              ┌──────────────┘
///      ╰──────────────┆ ─────────────╮
///         _  _  _     ↓  _  _  _     ↓  _  _  _
/// [11  ✘  3  4  5  ✘  7  8  9 10  6 12 13 14 15]    [1  2]
///   |              ┌──────────────┘
///   ╰──────────────┆ ─────────────╮
///         _  _  _  ↓  _  _  _  _  ↓  _  _  _  _
/// [ ✘  ✘  3  4  5  6  7  8  9 10 11 12 13 14 15]    [1  2]
///   ┌──┬─────────────────────────────────────────────┴──┘
/// [ 1  2  .  .  5* .  .  .  . 10:11  .  .  . 15]
/// ```
///
/// Same situation, buffer size = 2 (bridge = 5):
///
/// ```text
///                dim            mid
///     left = 10  |    bridge    |   right = 5
/// [ 1  2  3  4  5: 6  7  8  9 10*11 12 13 14 15]                // 1 round
///                  └──┴──────────────────────────────┬──┐
///   a-->           b-->           c-->               |  |
/// [ 1  .  .  .  5: ✘  ✘  8  .  .*11  .  .  . 15]    [6  7]
///   └──────────────┐              |
///   ╭───────────── ┆──────────────╯
///   ↓  a           ↓  b              c
/// [11  2  .  .  5  1  ✘  8  . 10  ✘ 12  .  . 15]    [6  7]
///      └──────────────┐              |
///      ╭───────────── ┆──────────────╯
///   _  ↓           _  ↓
/// [11 12  3  .  5  1  2  8  . 10  ✘  ✘ 13  . 15]    [6  7]
///   _  _           _  _           ┌──┬───────────────┴──┘
/// [11 12  3  .  5  1  2  8  9 10  6  7 13  . 15]    [✘  ✘]      // 2 round
///                        └──┴────────────────────────┬──┐
///   _  _  a-->     _  _  b-->     _  _  c-->         |  |
/// [11 12  3  .  5  1  2  ✘  ✘ 10  6  7 13  . 15]    [8  9]
///         └──────────────┐              |
///         ╭───────────── ┆──────────────╯
///   _  _  ↓  a     _  _  ↓  b     _  _     c
/// [11  . 13  4  5  1  .  3  ✘ 10  6  7  ✘ 14 15]    [8  9]
///            └──────────────┐              |
///            ╭───────────── ┆──────────────╯
///   _  _  _  ↓     _  _  _  ↓     _  _
/// [11  .  . 14  5  1  .  .  4 10  6  7  ✘  ✘ 15]    [8  9]
///   _  _  _  _     _  _  _  _     _     ┌──┬─────────┴──┘
/// [11  .  . 14  5  1  .  .  4 10  6  7  8  9 15]    [✘  ✘]      // 3 round
///                              └──────────────────────┐
///   _  _  _  _ a-->_  _  _  _ b-->_  _  _  _  c-->    |
/// [11  .  . 14  5  1  .  .  4  ✘  6  .  .  9 15]    [10]
///               └──────────────┐              |
///               ╭───────────── ┆──────────────╯
///   _  _  _  _  ↓  _  _  _  _  ↓  _  _  _  _
/// [11  .  .  . 15  1  .  .  .  5  .  .  .  9  ✘]    [10]
///   _  _  _  _  _  _  _  _  _  _  _  _  _  _  ┌───────┘
/// [11  .  .  . 15  1  .  .  .  .  .  .  .  9 10]
/// ```
///
/// Same situation, buffer size = 2 (bridge = 9):
///
/// ```text
///          dim                        mid
/// left = 12|           bridge         |right = 3
/// [ 1  2  3: 4  5  6  7  8  9 10 11 12*13 14 15]
///            └──┴────────────────────────────────────┬──┐
///   a-->     b-->                       c-->         |  |
/// [ 1  .  3: ✘  ✘  6  .  .  .  .  . 12*13  . 15]    [4  5]
///   └────────┐                          |
///   ╭─────── ┆──────────────────────────╯
///   ↓  a     ↓  b                          c
/// [13  2  3  1  ✘  6  .  .  .  .  . 12  ✘ 14 15]    [4  5]
///      └────────┐                          |
///      ╭─────── ┆──────────────────────────╯
///   _  ↓     _  ↓
/// [13 14  3  1  2  6  .  .  .  .  . 12  ✘  ✘ 15]    [4  5]

///   _  _     _  _                       ┌──┬─────────┴──┘
/// [13 14  3  1  2  6  7  .  .  .  . 12  4  5 15]    [✘  ✘]
///                  └──┴──────────────────────────────┬──┐
///   _  _  a-->     b-->                       c-->   |  |
/// [13 14  3  1  2  ✘  ✘  8  .  .  . 12  4  5 15]    [6  7]
///         └────────┐                          |
///         ╭─────── ┆──────────────────────────╯
///   _  _  ↓  _  _  ↓
/// [13 14 15  1  2  3  ✘  8  .  .  . 12  4  5  ✘]    [6  7]

///                     └──┴─────────────/\────────────┴──┘
///                     ┌──┬─────────────~~────────────┬──┐
///   _  _  a-->  _  b-->  |                    c-->   |  |
/// [13 14  3  1  2  6  4  5  .  .  . 12  ✘  ✘ 15]    [7  8]

///   _  _           _  _           ┌──┬───────────────┴──┘

/// [13 14  3  .  5  1  2  8  9 10  6  7 13  . 15]    [✘  ✘]
///                        └──┴────────────────────────┬──┐
///   _  _  a-->     _  _  b-->     _  _  c-->         |  |
/// [13 14  3  .  5  1  2  ✘  ✘ 10  6  7 13  . 15]    [8  9]
///         └──────────────┐              |
///         ╭───────────── ┆──────────────╯
///   _  _  ↓  a     _  _  ↓  b     _  _     c
/// [13  . 13  4  5  1  .  3  ✘ 10  6  7  ✘ 14 15]    [8  9]
///            └──────────────┐              |
///            ╭───────────── ┆──────────────╯
///   _  _  _  ↓     _  _  _  ↓     _  _
/// [13  .  . 14  5  1  .  .  4 10  6  7  ✘  ✘ 15]    [8  9]
///   _  _  _  _     _  _  _  _     _     ┌──┬─────────┴──┘
/// [13  .  . 14  5  1  .  .  4 10  6  7  8  9 15]    [✘  ✘]
///                              └──────────────────────┐
///   _  _  _  _ a-->_  _  _  _ b-->_  _  _  _  c-->    |
/// [13  .  . 14  5  1  .  .  4  ✘  6  .  .  9 15]    [10]
///               └──────────────┐              |
///               ╭───────────── ┆──────────────╯
///   _  _  _  _  ↓  _  _  _  _  ↓  _  _  _  _
/// [13  .  .  . 15  1  .  .  .  5  .  .  .  9  ✘]    [10]
///   _  _  _  _  _  _  _  _  _  _  _  _  _  _  ┌───────┘
/// [13  .  .  . 15  1  .  .  .  .  .  .  .  9 10]
/// ```
// unsafe fn ptr_raft_rotate<T>(left: usize, mid: *mut T, right: usize, mut buffer: Vec<T>) {
//     // // if T::IS_ZST {
//     //     // return;
//     // // }

//     // if (right == 0) || (left == 0) {
//     //     return;
//     // }

//     // let buffer = buffer.as_mut_ptr();
//     // let bridge = left.abs_diff(right);

//     // if bridge == 0 {
//     //     ptr::swap_nonoverlapping(mid.sub(left), mid, right);
//     // }

//     // let s = cmp::min(bridge, buffer.capacity());

//     // // if cmp::min(left, right) <= bridge {
//     //     // ptr_aux_rotate(left, mid, right);
//     //     // return;
//     // // }

//     // let mut a = mid.sub(left);
//     // let mut b = mid.sub(bridge);
//     // let mut c = mid;

//     // for _ in 0..xzcd {
//     //     ptr::copy_nonoverlapping(c, buffer, bridge);

//     //     for _ in 0..right {
//     //         c.write(a.read());
//     //         a.write(b.read());
//     //         a = a.add(1);
//     //         b = b.add(1);
//     //         c = c.add(1);
//     //     }

//     //     ptr::copy_nonoverlapping(buf, d.sub(bridge), bridge);
//     // }
// }

/// # Bridge rotation (without Auxilary)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "This is a slightly more complex auxiliary rotation than
/// auxiliary rotation that reduces the maximum auxiliary memory
/// requirement from `50%` to `33.(3)%`. Its first known publication
/// was in *2021* by *Igor van den Hoven*." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// 1. The specified range must be valid for reading and writing;
/// 2. The `buffer` length must be larger than `|right - left|`.
///
/// # Example:
///
/// ```text
///                            mid
///          left = 9          |   right = 6
/// [ 1  2  3  4  5  6: 7-9    *10 11 12 13 14 15]
///                     └─┴────────────────────────────┬─────┐
///   a-->              b-->     c-->                  |     |
/// [ 1  .  .  .  .  6: ✘  ✘  ✘*10  .  .  .  . 15]    [7  8  9]
///   └─────────────────┐       |
///   ╭──────────────── ┆───────╯
///   ↓  a              ↓  b        c
/// [10  2  .  .  .  6  1  ✘  ✘  ✘ 11  .  .  . 15]    [7  .  9]
///      └─────────────────┐       |
///      ╭──────────────── ┆───────╯
///      ↓  a              ↓  b        c
/// [10 11  3  .  .  6  1  2  ✘  ✘  ✘ 12  .  . 15]    [7  .  9]
///         └─────────────────┐       |
///         ╭──────────────── ┆───────╯
///         ↓  a              ↓  b        c
/// [10  . 12  4  .  6  1  .  3  ✘  ✘  ✘ 13  . 15]    [7  .  9]
///            └─────────────────┐       |
///            ╭──────────────── ┆───────╯
///            ↓  a              ↓  b        c
/// [10  .  . 13  5  6  1  .  .  4  ✘  ✘  ✘ 14 15]    [7  .  9]
///               └─────────────────┐       |
///               ╭──────────────── ┆───────╯
///               ↓  a              ↓  b        c
/// [10  .  .  . 14  6  1  .  .  .  5  ✘  ✘  ✘ 15]    [7  .  9]
///                  └─────────────────┐       |
///                  ╭──────────────── ┆───────╯
///                  ↓                 ↓  b
/// [10 ~~~~~~~~~~~ 15  1 ~~~~~~~~~~~~ 6  ✘  ✘  ✘]    [7-9    ]
///                                       ┌─────┬──────┴─┘
/// [10  .  .  .  . 15: 1  .  3* 4  .  6  7 ~~~ 9]
/// ```
///
/// ```text
///                   mid
///       left = 6    |           right = 9
/// [10 11 12 13 14 15*     1-3: 4  5  6  7  8  9]
///                         └─┴────────────────────────┬─────┐
///                     b        c              d      |     |
/// [10  .  .  .  . 15* ✘  ✘  ✘: 4  .  .  .  .  9]    [1  2  3]
///                  ╰─────────────────────────╮|
///                           ┌─────────────── ┆┘
///                  b       c↓                ↓d
/// [10  .  .  . 14  ✘  ✘  ✘  9  4  .  .  .  8 15]    [1  .  3]
///               ╰─────────────────────────╮|
///                        ┌─────────────── ┆┘
///               b       c↓                ↓d
/// [10  .  . 13  ✘  ✘  ✘  8  9  4  .  .  7 14 15]    [1  .  3]
///            ╰─────────────────────────╮|
///                     ┌─────────────── ┆┘
///            b       c↓                ↓d
/// [10  . 12  ✘  ✘  ✘  7  .  9  4  .  6 13  . 15]    [1  .  3]
///         ╰─────────────────────────╮|
///                  ┌─────────────── ┆┘
///         b       c↓                ↓d
/// [10 11  ✘  ✘  ✘  6  .  .  9  4  5 12  .  . 15]    [1  .  3]
///      ╰─────────────────────────╮|
///               ┌─────────────── ┆┘
///      b       c↓                ↓d
/// [10  ✘  ✘  ✘  5  .  .  .  9  4 11  .  .  . 15]    [1  .  3]
///   ╰─────────────────────────╮|
///            ┌─────────────── ┆┘
///           c↓               d↓
/// [ ✘  ✘  ✘  4 ~~~~~~~~~~~~ 9 10 ~~~~~~~~~~~ 15]    [1-3    ]
///   ┌─────┬──────────────────────────────────────────┴─┘
/// [ 1 ~~~ 3  4  .  6* 7  .  9:10  .  .  .  . 15]
/// ```
unsafe fn ptr_bridge_rotate_simple<T>(left: usize, mid: *mut T, right: usize, buffer: &mut [T]) {
    // if T::IS_ZST {
    // return;
    // }

    if (right == 0) || (left == 0) {
        return;
    }

    // type BufType = [usize; 32];
    // let mut rawarray = MaybeUninit::<(BufType, [T; 0])>::uninit();
    // let buf = rawarray.as_mut_ptr() as *mut T;

    let buf = buffer.as_mut_ptr();
    let bridge = left.abs_diff(right);

    // if cmp::min(left, right) <= bridge {
    // ptr_aux_rotate(left, mid, right);
    // return;
    // }

    let a = mid.sub(left);
    let b = mid;
    let c = mid.sub(left).add(right);
    let d = mid.add(right);

    if left > right {
        ptr::copy_nonoverlapping(c, buf, bridge);

        for i in 0..right {
            c.add(i).write(a.add(i).read());
            a.add(i).write(b.add(i).read());
        }

        ptr::copy_nonoverlapping(buf, d.sub(bridge), bridge);
    } else if left < right {
        ptr::copy_nonoverlapping(b, buf, bridge);

        for i in 1..=left {
            c.sub(i).write(d.sub(i).read());
            d.sub(i).write(b.sub(i).read());
        }

        ptr::copy_nonoverlapping(buf, a, bridge);
    } else {
        ptr::swap_nonoverlapping(mid.sub(left), mid, right);
    }
}

/// # Bridge rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// 1. If bridge > minimal side -- fallback to *Auxiliary rotation*;
/// 2. Otherwise:
///
///    2.1. move bridge to buffer, creating the vacant space;
///    2.2. copy elements to the left-side of the vacant space;
///    2.3. increase the vacant space moving the element after the vacant space;
///    2.4. put elements from the buffer space back.
///
/// "This is a slightly more complex auxiliary rotation than
/// auxiliary rotation that reduces the maximum auxiliary memory
/// requirement from `50%` to `33.(3)%`. If the overlap between the
/// two halves is smaller than the halves themselves it copies
/// the overlap to swap memory instead. Its first known publication
/// was in *2021* by *Igor van den Hoven*." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// 1. The specified range must be valid for reading and writing;
/// 2. The `buffer` length must be larger than `min(|right - left|, left, right)`.
///
/// # Example:
///
/// ```text
///                            mid
///          left = 9          |   right = 6
/// [ 1  2  3  4  5  6: 7-9    *10 11 12 13 14 15]
///                     └─┴────────────────────────────┬─────┐
///   a-->              b-->     c-->                  |     |
/// [ 1  .  .  .  .  6: ✘  ✘  ✘*10  .  .  .  . 15]    [7  8  9]
///   └─────────────────┐       |
///   ╭──────────────── ┆───────╯
///   ↓  a              ↓  b        c
/// [10  2  .  .  .  6  1  ✘  ✘  ✘ 11  .  .  . 15]    [7  .  9]
///      └─────────────────┐       |
///      ╭──────────────── ┆───────╯
///      ↓  a              ↓  b        c
/// [10 11  3  .  .  6  1  2  ✘  ✘  ✘ 12  .  . 15]    [7  .  9]
///         └─────────────────┐       |
///         ╭──────────────── ┆───────╯
///         ↓  a              ↓  b        c
/// [10  . 12  4  .  6  1  .  3  ✘  ✘  ✘ 13  . 15]    [7  .  9]
///            └─────────────────┐       |
///            ╭──────────────── ┆───────╯
///            ↓  a              ↓  b        c
/// [10  .  . 13  5  6  1  .  .  4  ✘  ✘  ✘ 14 15]    [7  .  9]
///               └─────────────────┐       |
///               ╭──────────────── ┆───────╯
///               ↓  a              ↓  b        c
/// [10  .  .  . 14  6  1  .  .  .  5  ✘  ✘  ✘ 15]    [7  .  9]
///                  └─────────────────┐       |
///                  ╭──────────────── ┆───────╯
///                  ↓                 ↓  b
/// [10 ~~~~~~~~~~~ 15  1 ~~~~~~~~~~~~ 6  ✘  ✘  ✘]    [7-9    ]
///                                       ┌─────┬──────┴─┘
/// [10  .  .  .  . 15: 1  .  3* 4  .  6  7 ~~~ 9]
/// ```
///
/// ```text
///                   mid
///       left = 6    |           right = 9
/// [10 11 12 13 14 15*     1-3: 4  5  6  7  8  9]
///                         └─┴────────────────────────┬─────┐
///                     b        c              d      |     |
/// [10  .  .  .  . 15* ✘  ✘  ✘: 4  .  .  .  .  9]    [1  2  3]
///                  ╰─────────────────────────╮|
///                           ┌─────────────── ┆┘
///                  b       c↓                ↓d
/// [10  .  .  . 14  ✘  ✘  ✘  9  4  .  .  .  8 15]    [1  .  3]
///               ╰─────────────────────────╮|
///                        ┌─────────────── ┆┘
///               b       c↓                ↓d
/// [10  .  . 13  ✘  ✘  ✘  8  9  4  .  .  7 14 15]    [1  .  3]
///            ╰─────────────────────────╮|
///                     ┌─────────────── ┆┘
///            b       c↓                ↓d
/// [10  . 12  ✘  ✘  ✘  7  .  9  4  .  6 13  . 15]    [1  .  3]
///         ╰─────────────────────────╮|
///                  ┌─────────────── ┆┘
///         b       c↓                ↓d
/// [10 11  ✘  ✘  ✘  6  .  .  9  4  5 12  .  . 15]    [1  .  3]
///      ╰─────────────────────────╮|
///               ┌─────────────── ┆┘
///      b       c↓                ↓d
/// [10  ✘  ✘  ✘  5  .  .  .  9  4 11  .  .  . 15]    [1  .  3]
///   ╰─────────────────────────╮|
///            ┌─────────────── ┆┘
///           c↓               d↓
/// [ ✘  ✘  ✘  4 ~~~~~~~~~~~~ 9 10 ~~~~~~~~~~~ 15]    [1-3    ]
///   ┌─────┬──────────────────────────────────────────┴─┘
/// [ 1 ~~~ 3  4  .  6* 7  .  9:10  .  .  .  . 15]
/// ```
pub unsafe fn ptr_bridge_rotate<T>(left: usize, mid: *mut T, right: usize, buffer: &mut [T]) {
    let bridge = left.abs_diff(right);

    if cmp::min(left, right) <= bridge {
        ptr_aux_rotate(left, mid, right, buffer);
        return;
    }

    ptr_bridge_rotate_simple(left, mid, right, buffer);
}

// unsafe fn print<T: std::fmt::Debug>(label: &str, mut p: *const T, size: usize) {
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

/// # Direct aka Juggling aka Dolphin rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at
/// `mid` becomes the first element. Equivalently, rotates the range
/// `left` elements to the left or `right` elements to the right.
///
/// ## Algorithm
///
/// "This is a relatively complex and inefficient way to rotate in-place,
///  though it does so in the minimal number of moves.
///
///  Its first known publication was in *1966*.
///
/// It computes the greatest common divisor and uses a loop to create
/// a chain of consecutive swaps." <<https://github.com/scandum/rotate>>
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
pub unsafe fn ptr_direct_rotate<T>(left: usize, mid: *mut T, right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    loop {
        // N.B. the below algorithms can fail if these cases are not checked
        if (right == 0) || (left == 0) {
            return;
        }

        let start = mid.sub(left);

        // `gcd` can be found before hand by calculating `gcd(left + right, right)`,
        // but it is faster to do one loop which calculates the gcd as a side effect, then
        // doing the rest of the chunk
        let mut gcd = right;

        // beginning of first round
        let mut tmp: T = start.read();
        let mut i = 0;
        let mut next = i + right;

        loop {
            if next == left + right {
                // end of first round
                start.write(tmp);
                break;
            } else if i > left {
                i -= left;

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

        return;
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
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // (ls -> le -> re -> rs -> ls)
///   ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮|
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆╯
///   ↓  ls               le  |   |rs       re┆
/// [10  2  .  .  .  .  .  8  1 15╯11 ..... 14╰>9]  // (ls, le, re, rs)
///      ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮|
///      ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆╯
///      ↓  ls         le  |        | rs re┆
/// [10 11  3  .  .  .  7  2  1 15 14 12 13╰>8  9]
///         ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ╮|
///         ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|╯
///         ↓  ls   le  |             re┆
/// [10  . 12  4  .  6  3  2  1 15 14 13╰>7  .  9]  // (ls, le, re)
///            ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
///            ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
///            ↓  ls |             re┆
/// [10 ~~~~~ 13  5  4  3  2  1 15 14╰>6 ~~~~~~ 9]  // (ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [10  .  .  . 14 15: 1  2  3* 4  5  .  .  .  9]
/// ```
///
/// Case: `left < right`, `6 - 9`.
///
/// ```text
///                   mid
///   ls-->      <--le|rs-->                <--re
/// [ 1  2  3  4  5  6* 7  8  9:10 11 12 13 14 15]  // (re -> rs -> ls -> le -> re)
///   | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
///   ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
///     ┆ls      le |   | rs                re  ↓
/// [ 7<╯2  .  .  5 ╰1 15  8  .  .  .  .  . 14  6]  // (re, rs, ls, le)
///      | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
///      ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
///        ┆lsle|          | rs          re  ↓
/// [ 7  8<╯3  4╰~2  1 15 14  9  .  .  . 13  5  6]
///         |╭ ┆┈┈┈┈┈┈┈┈┈┈┈╯        ╭─────╯
///         ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
///          ┆┆ls             | rs    re  ↓
/// [ 7  .  9╯╰3  2  1 15 14 13 10 11 12  4  .  6]  // (re, rs, ls)
///            ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
///             ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
///             ┆ ls             | re  ↓
/// [ 7 ~~~~~ 10╯ 2  1 15 14 13 12 11  3 ~~~~~~ 6]  // (re, ls)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ 7  .  .  . 11 12*13 14 15: 1  2  .  .  .  6]
/// ```
///
/// Case: `left > right`, `8 - 7`.
///
/// ```text
///   ls-->            <--le rs-->          <--re
/// [ 1  2  3  4  5  6  7: 8* 9 10 11 12 13 14 15]  // (ls -> le -> re -> rs -> ls)
///   ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭───────────── ┆╯
///   ↓  ls             le |   |rs          re┆
/// [ 9  2  .  .  .  .  7  1 15╯10 11 12 13 14╰>8]  // (ls, le, re, rs)
///      ╰────────╮     ╰┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈┈╮|
///      ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈╯ ╭─────── ┆╯
///      ↓  ls      le  |         |rs    re┆
/// [ 9 10  3  .  .  6  2  1 15 14╯11 12 13╰>7  8]  // (ls, le, re, rs)
///         ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮|
///         ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆╯
///         ↓  lsle  |               |rs┆
/// [ 9  . 11  4  5╮ 3  2  1 15 14 13╯12╰>6  .  8]  // (ls, le, rs)
///            ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
///            ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆╯
///            ↓  ls               re┆
/// [ 9 ~~~~~ 12  4  3  2  1 15 14 13╰>5 ~~~~~~ 8]  // (ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
///               ↓  ↓  ↓  ↓  ↓  ↓  ↓
/// [ 9  .  .  . 13 14 15: 1* 2  3  4  .  .  .  8]
/// ```
pub unsafe fn ptr_contrev_rotate<T>(left: usize, mid: *mut T, right: usize) {
    // if T::IS_ZST {
    // return;
    // }

    if left == 0 || right == 0 {
        return;
    }

    let (mut ls, mut le) = (mid.sub(left), mid.sub(1));
    let (mut rs, mut re) = (mid, mid.add(right).sub(1));

    if left == right {
        ptr::swap_nonoverlapping(mid, mid.sub(left), right);
    } else {
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

        let center = slice::from_raw_parts_mut(ls, re.offset_from(ls).abs() as usize + 1);
        center.reverse();
    }
}

/// # Trinity (Conjoined triple reversal + Bridge) rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Safety
///
/// 1. The specified range must be valid for reading and writing;
/// 2. The `buffer` length must be larger than `min(|right - left|, left, right)`.
///
/// ## Algorithm
///
/// "The trinity rotation (aka conjoined triple reversal) is derived from the triple reversal
/// rotation. Rather than three separate reversals it conjoins the three reversals, improving
/// locality and reducing the number of moves. Optionally, if the overlap is smaller than
/// `32 * size_of(usize)`, it skips the trinity rotation and performs an auxiliary
/// or bridge rotation on stack memory. Its first known publication was in 2021 by Igor van den Hoven."
/// <<https://github.com/scandum/rotate>>
pub unsafe fn ptr_trinity_rotate<T>(left: usize, mid: *mut T, right: usize, buffer: &mut [T]) {
    // if T::IS_ZST {
    // return;
    // }

    if cmp::min(left, right) <= buffer.len() {
        ptr_aux_rotate(left, mid, right, buffer);
        return;
    }

    let d = right.abs_diff(left);

    if d <= buffer.len() && d > 3 {
        ptr_bridge_rotate(left, mid, right, buffer);
        return;
    }

    ptr_contrev_rotate(left, mid, right);
}

/// # Algo1 (juggler) rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// This algorithm is extracted from current rotation implementation in Rust.
///
/// "In Rust *Algo1* is used for small values of `left + right` or for large `T`. The elements are moved
/// into their final positions one at a time starting at `mid - left` and advancing by `right` steps
/// modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
/// `mid - left`. However, if `gcd(left + right, right)` is not 1, the above steps skipped over
/// elements."
///
/// "Fortunately, the number of skipped over elements between finalized elements is always equal, so
/// we can just offset our starting position and do more rounds (the total number of rounds is the
/// `gcd(left + right, right)` value). The end result is that all elements are finalized once and
/// only once."
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
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]                      // round
///   └─────────────────┐
/// [ ✘  2  .  .  .  6  1  8  .  .  .  .  .  . 15]   [ 7]
///                                       ┌────────────┘
///                     _                 ↓
/// [ ✘  2  .  .  .  6  1  8  .  .  . 12  7 14 15]   [13]
///            ┌───────────────────────────────────────┘
///            ↓        _                 _
/// [ ✘  2  3 13  5  6  1  8  .  .  . 12  7 14 15]   [ 4]
///                              ┌─────────────────────┘
///            _        _        ↓        _
/// [ ✘  2  3 13  5  6  1  8  9  4 11 12  7 14 15]   [10]
///   ┌────────────────────────────────────────────────┘
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
pub unsafe fn ptr_algo1_rotate<T>(left: usize, mid: *mut T, right: usize) {
    loop {
        // N.B. the below algorithms can fail if these cases are not checked
        if (right == 0) || (left == 0) {
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

        return;
    }
}

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
/// *Algorithm 1* is used for small values of `left + right` or for large `T`. The elements are moved
/// into their final positions one at a time starting at `mid - left` and advancing by `right` steps
/// modulo `left + right`, such that only one temporary is needed. Eventually, we arrive back at
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
/// *Algorithm 2* is used if `left + right` is large but `min(left, right)` is small enough to
/// fit onto a stack buffer. The `min(left, right)` elements are copied onto the buffer, `memmove`
/// is applied to the others, and the ones on the buffer are moved back into the hole on the
/// opposite side of where they originated.
///
/// Algorithms that can be vectorized outperform the above once `left + right` becomes large enough.
/// *Algorithm 1* can be vectorized by chunking and performing many rounds at once, but there are too
/// few rounds on average until `left + right` is enormous, and the worst case of a single
/// round is always there. Instead, *algorithm 3* utilizes repeated swapping of
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

    fn buf_case(
        buf_rotate: unsafe fn(left: usize, mid: *mut usize, right: usize, buffer: &mut [usize]),
        size: usize,
        diff: usize,
        buffer: &mut [usize],
    ) {
        let (vec, (l, p, r)) = prepare(size, diff);

        let mut s = seq(size);

        s.rotate_left(l);
        unsafe { buf_rotate(l, p, r, buffer) };

        assert_eq!(vec, s);

        unsafe { buf_rotate(r, p.sub(diff), l, buffer) };

        s.rotate_right(l);
        assert_eq!(vec, s);
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

    fn test_buf_correctness(
        rotate_f: unsafe fn(left: usize, mid: *mut usize, right: usize, buffer: &mut [usize]),
    ) {
        let mut buffer = Vec::<usize>::with_capacity(100_000);

        // --empty--
        buf_case(rotate_f, 0, 0, buffer.as_mut_slice());

        // 1  2  3  4  5  6 (7  8  9)10 11 12 13 14 15
        buf_case(rotate_f, 15, 3, buffer.as_mut_slice());

        // 1  2  3  4  5  6  7 (8) 9 10 11 12 13 14 15
        buf_case(rotate_f, 15, 1, buffer.as_mut_slice());

        // 1  2  3  4  5 (6  7  8  9 10)11 12 13 14 15
        buf_case(rotate_f, 15, 5, buffer.as_mut_slice());

        // 1  2  3  4  5  6  7)(8  9 10 11 12 13 14
        buf_case(rotate_f, 14, 0, buffer.as_mut_slice());

        // 1  2  3  4 (5  6  7  8  9 10 11)12 13 14 15
        buf_case(rotate_f, 15, 7, buffer.as_mut_slice());

        // 1 (2  3  4  5  6  7  8  9 10 11 12 13 14)15
        buf_case(rotate_f, 15, 13, buffer.as_mut_slice());

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        buf_case(rotate_f, 15, 15, buffer.as_mut_slice());

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        buf_case(rotate_f, 100_000, 0, buffer.as_mut_slice());
    }

    fn test_correctness(rotate_f: unsafe fn(left: usize, mid: *mut usize, right: usize)) {
        // --empty--
        case(rotate_f, 0, 0);

        // 1  2  3  4  5  6 (7  8  9)10 11 12 13 14 15
        case(rotate_f, 15, 3);

        // 1  2  3  4  5 (6  7  8  9 10)11 12 13 14 15
        case(rotate_f, 15, 5);

        // 1  2  3  4  5  6  7 (8) 9 10 11 12 13 14 15
        case(rotate_f, 15, 1);

        // 1  2  3  4  5  6  7)(8  9 10 11 12 13 14
        case(rotate_f, 14, 0);

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
    fn ptr_aux_rotate_correctness() {
        test_buf_correctness(ptr_aux_rotate::<usize>);
    }

    #[test]
    // default (stable) rust rotate
    fn ptr_rotate_correctness() {
        test_correctness(stable_ptr_rotate::<usize>);
    }

    // #[test]
    // fn ptr_bridge_rotate_simple_correctness() {
    // test_correctness(ptr_bridge_rotate_simple::<usize>);
    // }

    // #[test]
    // fn ptr_bridge_rotate_correctness() {
    //     test_buf_correctness(ptr_bridge_rotate::<usize>);
    // }

    #[test]
    fn ptr_reversal_rotate_correctness() {
        test_correctness(ptr_reversal_rotate::<usize>);
    }

    #[test]
    fn ptr_griesmills_rotate_rec_correctness() {
        test_correctness(ptr_griesmills_rotate_rec::<usize>);
    }

    #[test]
    fn ptr_piston_rotate_rec_correctness() {
        test_correctness(ptr_piston_rotate_rec::<usize>);
    }

    #[test]
    fn ptr_piston_rotate_correctness() {
        test_correctness(ptr_piston_rotate::<usize>);
    }

    #[test]
    fn ptr_contrev_rotate_correctness() {
        test_correctness(ptr_contrev_rotate::<usize>);
    }

    #[test]
    fn ptr_trinity_rotate_correctness() {
        test_buf_correctness(ptr_trinity_rotate::<usize>);
    }

    #[test]
    fn ptr_juggling_rotate_correctness() {
        test_correctness(ptr_direct_rotate::<usize>);
    }

    #[test]
    fn ptr_helix_rotate_correctness() {
        test_correctness(ptr_helix_rotate::<usize>);
    }

    #[test]
    fn ptr_grail_rotate_correctness() {
        test_correctness(ptr_grail_rotate::<usize>);
    }

    #[test]
    fn ptr_drill_rotate_correctness() {
        test_correctness(ptr_drill_rotate::<usize>);
    }

    #[test]
    fn ptr_algo1_rotate_correctness() {
        test_correctness(ptr_algo1_rotate::<usize>);
    }
}
