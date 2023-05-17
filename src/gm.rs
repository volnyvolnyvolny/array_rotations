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

use crate::swap_backward;
use crate::swap_forward;
use std::ptr;

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
/// Recursive variant of the *GM* rotation is slightly slower than simple iterative *GM*.
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
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap r-side and its shadow
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3 10  -  -  -  - 15  4 ~~~~~~~~~~~~ 9]
///
///    l = 3     𝑠ℎ. r = 6
/// [ 1  .  3,10  - 12 13  - 15] 4  .  .  .  .  9   // swap new l-side and its shadow
///   └─────┴/\┴─────┘
///   ┌─────┬~/┬─────┐
/// [10 ~~ 12  1  -  3 13  - 15] 4  .  .  .  .  9
///
///             l = 3   r = 3
///  10 ~~ 12[ 1  -  3;13  - 15] 4  .  .  .  .  9   // swap equal
///            └─────┴/\┴─────┘
///            ┌─────┬~~┬─────┐
///  10 ~~ 12[13 ~~ 15  1 ~~~ 3] 4  .  .  .  .  9
///
/// [10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_griesmills_rotate_rec<T>(left: usize, mid: *mut T, right: usize) {
    if (right == 0) || (left == 0) {
        return;
    }

    if left < right {
        let start = mid.sub(left);
        ptr::swap_nonoverlapping(start, mid, left);
        ptr_griesmills_rotate_rec(left, mid.add(left), right - left);
    } else {
        ptr::swap_nonoverlapping(mid, mid.sub(right), right);
        ptr_griesmills_rotate_rec(left - right, mid.sub(right), right);
    }
}

/// # Gries-Mills rotation
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
/// When the smallest side reaches a size of `1` element -- it is the worst case for the
/// *Gries-Mills rotation*.
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
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap r-side and its shadow
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3 10  -  -  -  - 15  4 ~~~~~~~~~~~~ 9]
///
///    l = 3     𝑠ℎ. r = 6
/// [ 1  .  3,10  - 12 13  - 15] 4  .  .  .  .  9   // swap new l-side and its shadow
///   └─────┴/\┴─────┘
///   ┌─────┬~/┬─────┐
/// [10 ~~ 12  1  -  3 13  - 15] 4  .  .  .  .  9
///
///             l = 3   r = 3
///  10  . 12[ 1  -  3;13  - 15] 4  .  .  .  .  9   // swap equal
///            └─────┴/\┴─────┘
///            ┌─────┬~~┬─────┐
///  10  . 12[13 ~~ 15  1 ~~~ 3] 4  .  .  .  .  9
///
/// [10 ~~~~~~~~~~~ 15: 1  .  3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_griesmills_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    loop {
        if (right == 0) || (left == 0) {
            return;
        }

        if left < right {
            let start = mid.sub(left);
            ptr::swap_nonoverlapping(start, mid, left);
            mid = mid.add(left);
            right -= left;
        } else {
            ptr::swap_nonoverlapping(mid, mid.sub(right), right);
            mid = mid.sub(right);
            left -= right;
        }
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
/// When the smallest side reaches a size of `1` element -- it is the worst case for the
/// *Gries-Mills rotation*.
/// <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Performance
///
/// *GM* outperforms *Grail*.
///
/// ## Examples
///
/// ```text
///                            mid
///           left = 9         |     right = 6
/// [ 1  2  3  4  5  6: 7  8  9*10 11 12 13 14 15]  // swap <--
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\~┬──────────────┐
/// [ 1  .  3;10  - 12 13  - 15] 4 ~~~~~~~~~~~~ 9   // swap <--
///   └─────┴/\┴─────┘
///   ┌─────┬~/┬─────┐
/// [10 ~~ 12  1  -  3 13  - 15] 4  .  .  .  .  9   // swap -->
///            └─────┴/\┴─────┘
///            ┌─────┬~~┬─────┐
///  10  . 12[13 ~~ 15  1 ~~~ 3] 4  .  .  .  .  9
///
/// [10 ~~~~~~~~~~~ 15: 1 ~~~ 3* 4  .  .  .  .  9]
/// ```
pub unsafe fn ptr_grail_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    loop {
        if (right == 0) || (left == 0) {
            return;
        }

        if left <= right {
            let start = mid.sub(left);
            swap_forward(start, mid, left); // !

            mid = mid.add(left);
            right -= left;
        } else {
            swap_backward(mid.sub(right), mid, right); // !

            mid = mid.sub(right);
            left -= right;
        }
    }
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
/// [ 1  .  3;10  -  -  -  - 15] 4 ~~~~~~~~~~~~ 9   // swap -->
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

            swap_forward(start, mid, s);
            mid = mid.add(s);
            start = start.add(s);
        }

        // <--
        if right < 1 {
            break;
        }

        let old_l = left;
        left %= right;

        s = old_l - left;

        swap_backward(mid.sub(s), end.sub(s), s);
        mid = mid.sub(s);
        end = end.sub(s);
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
    fn ptr_griesmills_rotate_rec_correct() {
        test_correct(ptr_griesmills_rotate_rec::<usize>);
    }

    #[test]
    fn ptr_griesmills_rotate_correct() {
        test_correct(ptr_griesmills_rotate::<usize>);
    }

    #[test]
    fn ptr_grail_rotate_correct() {
        test_correct(ptr_grail_rotate::<usize>);
    }

    #[test]
    fn ptr_drill_rotate_correct() {
        test_correct(ptr_drill_rotate::<usize>);
    }
}
