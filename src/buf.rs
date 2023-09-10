/*
Copyright (C) 2023 Valentin Vasilev (3volny@gmail.com).
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

use crate::copy;
use crate::ptr_contrev_rotate;
use crate::ptr_edge_rotate;
use std::cmp;
use std::ptr;

/// # Auxiliary rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// This implementation uses `copy_forward` and `copy_backward` that are faster than `ptr:copy`.
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
/// [              1-6 :7  .  9  ✘  ✘  ✘  ✘  ✘  ✘]    [10 .. 15]  // move
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
    if right <= 2 || left <= 2 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    let start = mid.sub(left);
    let buf = buffer.as_mut_ptr();
    let dim = start.add(right);

    if left < right {
        ptr::copy_nonoverlapping(start, buf, left);
        copy(mid, start, right); // ! see 'ptr_naive_aux_rotate'
        ptr::copy_nonoverlapping(buf, dim, left);
    } else if right < left {
        ptr::copy_nonoverlapping(mid, buf, right);
        copy(start, dim, left); // !
        ptr::copy_nonoverlapping(buf, start, right);
    } else {
        ptr::swap_nonoverlapping(start, mid, left);
    }
}

/// # Auxiliary rotation (Naive)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// "Naive" implementation uses `ptr:copy` which is significantly slower than `copy_forward` and
/// `copy_backward`.
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
/// [              1-6 :7  .  9  ✘  ✘  ✘  ✘  ✘  ✘]    [10 .. 15]  // move
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
pub unsafe fn ptr_naive_aux_rotate<T>(left: usize, mid: *mut T, right: usize, buffer: &mut [T]) {
    if right <= 2 || left <= 2 {
        ptr_edge_rotate(left, mid, right);
        return;
    }

    let start = mid.sub(left);
    let buf = buffer.as_mut_ptr();
    let dim = start.add(right);

    if left < right {
        ptr::copy_nonoverlapping(start, buf, left);
        ptr::copy(mid, start, right);
        ptr::copy_nonoverlapping(buf, dim, left);
    } else if right < left {
        ptr::copy_nonoverlapping(mid, buf, right);
        ptr::copy(start, dim, left);
        ptr::copy_nonoverlapping(buf, start, right);
    } else {
        ptr::swap_nonoverlapping(start, mid, left);
    }
}

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
    if right <= 2 || left <= 2 {
        ptr_edge_rotate(left, mid, right);
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

    fn test_correct(
        rotate_f: unsafe fn(left: usize, mid: *mut usize, right: usize, buffer: &mut [usize]),
    ) {
        let mut buffer = Vec::<usize>::with_capacity(100_000);

        // --empty--
        case(rotate_f, 0, 0, buffer.as_mut_slice());

        // 1  2  3  4  5  6 (7  8  9)10 11 12 13 14 15
        case(rotate_f, 15, 3, buffer.as_mut_slice());

        // 1  2  3  4  5  6  7 (8) 9 10 11 12 13 14 15
        case(rotate_f, 15, 1, buffer.as_mut_slice());

        // 1  2  3  4  5 (6  7  8  9 10)11 12 13 14 15
        case(rotate_f, 15, 5, buffer.as_mut_slice());

        // 1  2  3  4  5  6  7)(8  9 10 11 12 13 14
        case(rotate_f, 14, 0, buffer.as_mut_slice());

        // 1  2  3  4 (5  6  7  8  9 10 11)12 13 14 15
        case(rotate_f, 15, 7, buffer.as_mut_slice());

        // 1 (2  3  4  5  6  7  8  9 10 11 12 13 14)15
        case(rotate_f, 15, 13, buffer.as_mut_slice());

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        case(rotate_f, 15, 15, buffer.as_mut_slice());

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        case(rotate_f, 100_000, 0, buffer.as_mut_slice());
    }

    #[test]
    fn ptr_aux_rotate_correct() {
        test_correct(ptr_aux_rotate::<usize>);
    }

    #[test]
    fn ptr_naive_aux_rotate_correct() {
        test_correct(ptr_naive_aux_rotate::<usize>);
    }

    #[test]
    fn ptr_bridge_rotate_correct() {
        test_correct(ptr_bridge_rotate::<usize>);
    }

    #[test]
    fn ptr_trinity_rotate_correct() {
        test_correct(ptr_trinity_rotate::<usize>);
    }
}
