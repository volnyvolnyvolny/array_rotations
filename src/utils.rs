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

use std::mem::size_of;
use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

/// # Reverse slice
///
/// Reverse slice `[p, p+count)`.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                 count = 7
/// [ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]  // reverse slice
///            └─────────────────┘
/// [ 1  .  3 10  9  8  7  6  5  4 11  .  .  . 15]
/// ```
#[inline(always)]
pub unsafe fn reverse_slice<T>(p: *mut T, count: usize) {
    let slice = slice::from_raw_parts_mut(p, count);
    slice.reverse();
}

/// # Swap
///
/// Swap elements `p.add(x)` and `p.add(y)``.
///
/// ## Safety
///
/// The specified elements must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///            x                 y
/// [ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]  // swap
///            └─────────────────┘
/// [ 1  .  3 10  5  .  .  .  9 11  .  .  . 15]
/// ```
#[inline(always)]
pub unsafe fn swap<T>(p: *mut T, x: usize, y: usize) {
    let (x, y) = (p.add(x), p.add(y));

    let x_ref = unsafe { &mut *x.cast::<T>() };
    let y_ref = unsafe { &mut *y.cast::<T>() };

    std::mem::swap(x_ref, y_ref);
}

/// # Copy backward
///
/// Copy region `[src, src + count)` to `[dst, dst + count)` moving left.
///
/// Regions could overlap.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///            src      dst    count = 7
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15]  // copy backward
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3 *4  .  6 :4 ~~~~~~~~~~~~~~ 10 14 15]
/// ```
///
/// ```text
///            dst      src   count = 7
/// [ 1  2  3: 4  5  6* 7  8  9 10 11 12 13 14 15]  // copy backward
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3  4  .  6  7 11 ~~ 13 11  . 13 14 15]  // after 3 iterations
///
/// [ 1  .  3:13 11 ~~*13 11  . 13 11  .  .  . 15]  // 4 more iterations.
/// ```
pub unsafe fn copy_backward<T>(src: *const T, dst: *mut T, count: usize) {
    let src = src.cast::<MaybeUninit<T>>();
    let dst = dst.cast::<MaybeUninit<T>>();

    for i in (0..count).rev() {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let src = unsafe { src.add(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let dst = unsafe { &mut *dst.add(i) };

        ptr::write(dst, ptr::read(src));
    }
}

/// # Copy forward
///
/// Copy region `[src, src + count)` to `[dst, dst + count)` moving right.
///
/// Regions could overlap.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///            src      dst    count = 7
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15]  // copy forward
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3 *4  .  6 :4  5  6  4  5  6  4 14 15]
/// ```
///
/// ```text
///            dst      src   count = 7
/// [ 1  2  3: 4  5  6* 7  8  9 10 11 12 13 14 15]  // copy forward
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3: 7 ~~~~~~~~~~~~~~ 13 11  . 13 14 15]
/// ```
pub unsafe fn copy_forward<T>(src: *const T, dst: *mut T, count: usize) {
    let src = src.cast::<MaybeUninit<T>>();
    let dst = dst.cast::<MaybeUninit<T>>();

    for i in 0..count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let src = unsafe { src.add(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let dst = unsafe { &mut *dst.add(i) };

        ptr::write(dst, ptr::read(src));
    }
}

/// # Shift left (backward, naive)
///
/// Shift region `[src, src + count)` to `[src - 1, src - 1 + count)`, moving left-to-right.
///
/// ## Safety
///
/// * The region `[src - 1, src - 1 + count)` must be valid for writing;
/// * the region `[src    , src     + count)` must be valid for reading.
///
/// ## Example
///
/// ```text
///          <<src  count = 7
/// [ 1  2 :3 *4  5  6  7  8  9 10 11 12 13 14 15]
///            └─────────────────┘
/// [ 1  2 :4 *5  6  7  8  9 10 10 11  .  .  . 15]
/// ```
pub unsafe fn shift_left_naive<T>(arr: *mut T, count: usize) {
    let arr = arr.cast::<MaybeUninit<T>>();

    for i in 0..count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let src = unsafe { arr.add(i) };

        ptr::write(src.sub(1), ptr::read(src));
    }
}

/// # Shift left (backward)
///
/// Shift region `[src, src + count)` to `[src - 1, src - 1 + count)`, moving left-to-right.
///
/// ## Safety
///
/// * The region `[src - 1, src - 1 + count)` must be valid for writing;
/// * the region `[src    , src     + count)` must be valid for reading.
///
/// ## Example
///
/// ```text
///          <<src  count = 7
/// [ 1  2 :3 *4  5  6  7  8  9 10 11 12 13 14 15]
///            └─────────────────┘
/// [ 1  2 :4 *5  6  7  8  9 10 10 11  .  .  . 15]
/// ```
pub unsafe fn shift_left<T>(arr: *mut T, count: usize) {
    if size_of::<T>() < 18 * size_of::<usize>() {
        shift_left_naive(arr, count);
    } else {
        ptr::copy(arr, arr.sub(1), count);
    }
}

/// # Shift right (forward)
///
/// Shift region `[src, src + count)` to `[src + 1, src + 1 + count)`, moving right-to-left.
///
/// ## Safety
///
/// * The region `[src + 1, src + 1 + count)` must be valid for writing;
/// * the region `[src    , src     + count)` must be valid for reading.
///
/// ## Example
///
/// ```text
///            src>> count = 7
/// [ 1  2  3 *4 :5  6  7  8  9 10 11 12 13 14 15]
///            └─────────────────┘
/// [ 1  2  3 *4 :4  5  6  7  8  9 10 12  .  . 15]
/// ```
pub unsafe fn shift_right<T>(arr: *mut T, count: usize) {
    let arr = arr.cast::<MaybeUninit<T>>();

    for i in (0..count).rev() {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let src = unsafe { arr.add(i) };

        ptr::write(src.add(1), ptr::read(src));
    }
}

/// # Swap forward
///
/// Swaps regions `[x, x+count)` and `[y, y+count)` moving right,
/// element by element.
/// Regions could overlap.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///            x        y     count = 7
/// [ 1  2  3 :4  5  6 *7  8  9 10 11 12 13 14 15]  // swap forward
///            └─────── |───/\───┘        |
///                     └───\/────────────┘
/// [ 1  .  3 :7 ~~~~~~*~~~~~~~ 13  5  6  4 14 15]
/// ```
///
/// In details:
///
/// ```text
///            x-->     y-->
/// [ 1  2  3 :4  5  6 *7  8  9 10 11 12 13 14 15]  // swap forward
///            _  x-->  _  y-->
/// [ 1  .  3  7  5  6  4  8  .  .  .  .  .  . 15]  // 5 6 4
///               _  x-->  _  y-->
/// [ 1  .  3  7  8  6  4  5  9  .  .  .  .  . 15]  //   6 4 5
///                  _  x-->  _  y-->
/// [ 1  .  3  7  .  9  4  .  6 10  .  .  .  . 15]  //     4 5 6
///                     _  x-->  _  y-->
/// [ 1  .  3  7  .  . 10  5  6  4 11  .  .  . 15]  //       5 6 4
///                        _  x-->  _  y-->
/// [ 1  .  3  7  .  .  . 11  6  4  5 12  .  . 15]  //         6 4 5
///                           _  x-->  _  y-->
/// [ 1  .  3  7  .  .  . 11 12  4  5  6 13  . 15]  // 4-6 and 7-12 are swaped!
///                              _        _
/// [ 1  .  3 :7  .  . *.  .  . 13  5  6  4 14 15]  // and 5 6 4, again.
/// ```
pub unsafe fn swap_forward<T>(x: *mut T, y: *mut T, count: usize) {
    let x = x.cast::<MaybeUninit<T>>();
    let y = y.cast::<MaybeUninit<T>>();

    for i in 0..count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let x = unsafe { &mut *x.add(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let y = unsafe { &mut *y.add(i) };

        let a = ptr::read(x);
        let b = ptr::read(y);
        ptr::write(x, b);
        ptr::write(y, a);
    }
}

/// # Swap backward
///
/// Swaps regions `[x, x+count)` and `[y, y+count)` moving left,
/// element by element.
/// Regions could overlap.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
///
/// ## Example
///
/// ```text
///                              x        y
/// [ 1  2  3 :4  5  6 *7  8  9 10 11 12 13 14 15]  // swap backward
///            |        └───/\───| ───────┘
///            └────────────\/───┘
/// [ 1  .  3:13 11 12* 4 ~~~~~~~~~~~~~~ 10 14 15]
/// ```
///
/// In details:
///
/// ```text
///                           <--x     <--y
/// [ 1  .  3: 4  5  6 *7  8  9 10 11 12 13 14 15]  // swap backward  11 12 13
///                        <--x  _  <--y  _
/// [ 1  .  3  4  5  6  7  8  9 13 11 12 10 14 15]  //             13 11 12
///                     <--x  _  <--y  _
/// [ 1  .  3  4  5  6  7  8 12 13 11  9 10 14 15]  //          12 13 11
///                  <--x  _  <--y  _
/// [ 1  .  3  4  5  6  7 11 12 13  8  . 10 14 15]  //       11 12 13
///               <--x  _  <--y  _
/// [ 1  .  3  4  5  6 13 11 12  7  .  . 10 14 15]  //    13 11 12
///            <--x  _  <--y  _
/// [ 1  .  3  4  5 12 13 11  6  .  .  . 10 14 15]  // 12 13 11
///         <--x  _  <--y  _
/// [ 1  .  3  4 11  . 13  5 ~~~~~~~~~~~ 10 14 15]  // 11-13 and 5-10 are swaped!
///            _        _
/// [ 1  .  3:13 11 12 *4 ~~~~~~~~~~~~~~ 10 14 15]  // and 13 11 12, again.
/// ```
pub unsafe fn swap_backward<T>(x: *mut T, y: *mut T, count: usize) {
    let x = x.add(count); //.cast::<MaybeUninit<T>>();
    let y = y.add(count); //.cast::<MaybeUninit<T>>();

    for i in 1..=count {
        // while i <= count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let x = unsafe { &mut *x.sub(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let y = unsafe { &mut *y.sub(i) };

        let a = ptr::read(x);
        let b = ptr::read(y);
        ptr::write(x, b);
        ptr::write(y, a);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn seq_multi<const count: usize>(size: usize) -> Vec<[usize; count]> {
        let mut v = vec![[0; count]; size];
        for i in 0..size {
            v[i] = [i + 1; count];
        }
        v
    }

    fn seq(size: usize) -> Vec<usize> {
        let mut v = vec![0; size];
        for i in 0..size {
            v[i] = i + 1;
        }
        v
    }

    fn prepare_swap(len: usize, x: usize, y: usize) -> (Vec<usize>, (*mut usize, *mut usize)) {
        let mut v = seq(len);

        unsafe {
            let x = &v[..].as_mut_ptr().add(x - 1);
            let y = &v[..].as_mut_ptr().add(y - 1);

            (v, (x.clone(), y.clone()))
        }
    }

    fn prepare_copy(
        len: usize,
        src: usize,
        dst: usize,
    ) -> (Vec<usize>, (*const usize, *mut usize)) {
        let mut v = seq(len);

        unsafe {
            let src = &v[..].as_ptr().add(src - 1);
            let dst = &v[..].as_mut_ptr().add(dst - 1);

            (v, (src.clone(), dst.clone()))
        }
    }

    #[test]
    fn reverse_slice_correct() {
        let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        unsafe { reverse_slice(v.as_mut_ptr(), 15) };

        assert_eq!(v, vec![15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);

        v = vec![1, 2, 3];
        unsafe { reverse_slice(v.as_mut_ptr(), 3) };

        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn mem_swap_correct() {
        let mut v = vec![1, 2, 3];

        unsafe { swap(v.as_mut_ptr(), 0, 2) };

        assert_eq!(v, vec![3, 2, 1]);

        //
        v = vec![1, 2];

        unsafe { swap(v.as_mut_ptr(), 0, 1) };

        assert_eq!(v, vec![2, 1]);

        //
        v = vec![1, 2, 3, 4, 5, 6, 7, 8];

        unsafe { swap(v.as_mut_ptr(), 0, 7) };

        assert_eq!(v, vec![8, 2, 3, 4, 5, 6, 7, 1]);

        //
        unsafe { swap(v.as_mut_ptr(), 4, 3) };

        assert_eq!(v, vec![8, 2, 3, 5, 4, 6, 7, 1]);
    }

    #[test]
    fn copy_backward_correct() {
        let (v, (src, dst)) = prepare_copy(15, 4, 7);

        unsafe { copy_backward(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare_copy(15, 7, 4);

        unsafe { copy_backward(src, dst, 7) };

        let s = vec![1, 2, 3, 13, 11, 12, 13, 11, 12, 13, 11, 12, 13, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn copy_forward_correct() {
        let (v, (src, dst)) = prepare_copy(15, 4, 7);

        unsafe { copy_forward(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 4, 5, 6, 4, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare_copy(15, 7, 4);

        unsafe { copy_forward(src, dst, 7) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 11, 12, 13, 14, 15];
        assert_eq!(v, s);
    }

    // Shifts:

    #[test]
    fn shift_left_correct() {
        let mut v = seq(15);
        let mut src = *unsafe { &v[..].as_mut_ptr().add(3) };

        unsafe { shift_left(src, 7) };

        assert_eq!(v, vec![1, 2, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 15]);

        src = *unsafe { &v[..].as_mut_ptr().add(2) };

        unsafe { shift_left(src, 7) };

        assert_eq!(v, vec![1, 4, 5, 6, 7, 8, 9, 10, 10, 10, 11, 12, 13, 14, 15]);

        v = seq(15);
        let mut src = *unsafe { &v[..].as_mut_ptr().add(3) };

        unsafe { shift_left_naive(src, 7) };

        assert_eq!(v, vec![1, 2, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 15]);

        src = *unsafe { &v[..].as_mut_ptr().add(2) };

        unsafe { shift_left_naive(src, 7) };

        assert_eq!(v, vec![1, 4, 5, 6, 7, 8, 9, 10, 10, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn shift_right_correct() {
        let mut v = seq(15);
        let mut src = *unsafe { &v[..].as_mut_ptr().add(3) };

        unsafe { shift_right(src, 7) };

        assert_eq!(v, vec![1, 2, 3, 4, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15]);

        src = *unsafe { &v[..].as_mut_ptr().add(4) };

        unsafe { shift_right(src, 7) };

        assert_eq!(v, vec![1, 2, 3, 4, 4, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15]);
    }

    #[test]
    fn shift_correct() {
        let mut v = seq_multi::<20>(15);
        let mut src = *unsafe { &v[..].as_mut_ptr().add(1) };

        unsafe { shift_left(src, 14) };

        assert_eq!(v[0..13], seq_multi::<20>(14)[1..14]);

        v = seq_multi::<20>(15);
        src = *&v[..].as_mut_ptr();

        unsafe { shift_right(src, 14) };
        assert_eq!(v[1..14], seq_multi::<20>(14)[0..13]);
    }

    // Swaps:

    #[test]
    fn swap_forward_correct() {
        let (v, (x, y)) = prepare_swap(15, 4, 7);

        unsafe { swap_forward(x, y, 7) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 5, 6, 4, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn swap_backward_correct() {
        let (v, (x, y)) = prepare_swap(15, 4, 7);

        unsafe { swap_backward(x, y, 7) };

        let s = vec![1, 2, 3, 13, 11, 12, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (x, y)) = prepare_swap(15, 1, 7);

        unsafe { swap_backward(x, y, 9) };

        let s = vec![13, 14, 15, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(v, s);

        let (v, (x, y)) = prepare_swap(15, 1, 8);

        unsafe { swap_backward(x, y, 8) };

        let s = vec![15, 9, 10, 11, 12, 13, 14, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(v, s);
    }
}
