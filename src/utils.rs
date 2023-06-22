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
use std::ptr::copy_nonoverlapping;
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

/// # Copy (may overlap)
///
/// Copy region `[src, src + count)` to `[dst, dst + count)` element by element.
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
///            dst      src   count = 7
/// [ 1  2  3: 4  5  6* 7  8  9 10 11 12 13 14 15]  // copy -->
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3: 7 ~~~~~~~~~~~~~~ 13 11  . 13 14 15]
/// ```
///
/// ```text
///            src      dst    count = 7
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15]  // copy <--
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3 *4  .  6 :4 ~~~~~~~~~~~~~~ 10 14 15]
/// ```
pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize) {
    #[inline(always)]
    unsafe fn _copy<T>(src: *const T, dst: *mut T, i: usize) {
        // SAFE: By precondition, `i` is in-bounds because it's below `count`
        let src = unsafe { src.add(i) };

        // SAFE: By precondition, `i` is in-bounds because it's below `count`
        let dst = unsafe { &mut *dst.add(i) };

        ptr::write(dst, ptr::read(src));
    }

    if src == dst {
        return;
    } else if src > dst {
        for i in 0..count {
            _copy(src, dst, i);
        }
    } else {
        for i in (0..count).rev() {
            _copy(src, dst, i);
        }
    }
}

/// # Copy (may overlap)
///
/// Copy region `[src, src + count)` to `[dst, dst + count)` by byte.
///
/// Regions could overlap.
///
/// ## Safety
///
/// The specified range must be valid for reading and writing.
pub unsafe fn byte_copy<T>(src: *const T, dst: *mut T, count: usize) {
    let src = src.cast::<u8>();
    let dst = dst.cast::<u8>();

    copy(src, dst, count * size_of::<T>());
}

/// # Copy (may overlap)
///
/// Copy region `[src, src + count)` to `[dst, dst + count)` block by block.
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
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15]  // copy
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3 *4  .  6 :4  5  6  4  5  6  4 14 15]
/// ```
///
/// ```text
///            src      dst    count = 13
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15 16 17 18 19 20]  // copy block(3)
///            └─────── | ─────────────────────────┘        |
///                     └───────────────────────────────────┘
/// [ 1  .  .  .  .  .  .  .  .  .  .  . 13 14  . 16 14  ~ 16 20]  // copy block
/// [ 1  .  .  .  .  .  .  .  .  . 11  . 13 11  ~ 13 14  . 16 20]  // copy block
/// [ 1  .  .  .  .  .  .  8  . 10  8  ~ 10 11  .  .  .  . 16 20]  // copy block
/// [ 1  .  .  .  .  .  .  5  ~  7  8  .  .  .  .  .  .  . 16 20]  // copy rem(1)
/// [ 1  .  3 *4  .  6 :4 ~~~~~~~~~~~~~~~~~~~~~~~ 16 14  . 16 20]
/// ```
///
/// ```text
///            src      dst    count = 7
/// [ 1  2  3 *4  5  6 :7  8  9 10 11 12 13 14 15]  // copy block(3)
///            └─────── | ───────┘        |
///                     └─────────────────┘
/// [ 1  .  .  .  .  .  7  8  . 10  8  ~ 10 14 15]  // copy block
/// [ 1  .  3  4  5  .  7  5  ~  7  8  . 10 14 15]  // copy rem(1)
/// [ 1  .  3 *4  .  6 :4 ~~~~~~~~~~~~~~ 10 14 15]
/// ```
///
/// ```text
///            dst      src    count = 7
/// [ 1  2  3: 4  5  6* 7  8  9 10 11 12 13 14 15]  // copy block(3)
///            └─────── |────────┘        |
///                     └─────────────────┘
/// [ 1  .  3  7  ~  9  7  .  9 10  .  .  .  . 15]  // copy block
/// [ 1  .  3  7  .  9 10  ~ 12 10  .  .  .  . 15]  // copy rem(1)
/// [ 1  .  3 *7 ~~~~~~~~~~~~~~ 13 11  .  .  . 15]
/// ```
pub unsafe fn block_copy<T>(src: *const T, dst: *mut T, count: usize) {
    let block_size = dst.offset_from(src).unsigned_abs();

    if src == dst {
        return;
    } else if block_size == 1 {
        copy(src, dst, count);
    } else if block_size > count {
        copy_nonoverlapping(src, dst, count);
    } else {
        let mut s = src;
        let mut d = dst;

        let rounds = count / block_size + 1;
        let rem = count % block_size;

        if src < dst {
            s = src.add(count);
            d = dst.add(count);

            for _ in 1..rounds {
                s = s.sub(block_size);
                d = d.sub(block_size);

                copy_nonoverlapping(s, d, block_size);
            }

            s = s.sub(rem);
            d = d.sub(rem);
            copy_nonoverlapping(s, d, rem);
        } else {
            for _ in 1..rounds {
                copy_nonoverlapping(s, d, block_size);

                s = s.add(block_size);
                d = d.add(block_size);
            }

            s = s.add(1);
            d = d.add(1);
            copy_nonoverlapping(s, d, rem);
        }
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
/// [ 1  2 :4 *5 ~~~~~~~~~~~ 10 10 11  .  .  . 15]
/// ```
pub unsafe fn shift_left<T>(arr: *mut T, count: usize) {
    if size_of::<T>() < 18 * size_of::<usize>() {
        copy(arr, arr.sub(1), count);
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
/// [ 1  2  3 *4 :4 ~~~~~~~~~~~~~~ 10 12  .  . 15]
/// ```
pub unsafe fn shift_right<T>(arr: *mut T, count: usize) {
    copy(arr, arr.add(1), count);
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
/// [ 1  2  3 :4  5  6 *7  8  9 10 11 12 13 14 15]  // swap -->
///            └─────── |───/\───┘        |
///                     └───\/────────────┘
/// [ 1  .  3 :7 ~~~~~~*~~~~~~~ 13  5  6  4 14 15]
/// ```
///
/// In details:
///
/// ```text
///            x-->     y-->
/// [ 1  2  3 :4  5  6* 7  8  9 10 11 12 13 14 15]  // swap -->
///            └────────┘
/// [ 1  .  3  7  5  6  4  8  .  .  .  .  .  . 15]  // 5 6 4
///               └────────┘
/// [ 1  .  3  7  8  6  4  5  9  .  .  .  .  . 15]  //   6 4 5
///                  └────────┘
/// [ 1  .  3  7  .  9  4  .  6 10  .  .  .  . 15]  //     4 5 6
///                     └────────┘
/// [ 1  .  3  7  .  . 10  5  6  4 11  .  .  . 15]  //       5 6 4
///                        └────────┘
/// [ 1  .  3  7  .  .  . 11  6  4  5 12  .  . 15]  //         6 4 5
///                           └────────┘
/// [ 1  .  3  7 ~~~~~~~~~~~ 12  4  5  6 13  . 15]  // 4-6 and 7-12 are swaped!
///                              └────────┘
/// [ 1  .  3 :7  .  9*10  .  . 13  5  6  4 14 15]  // and 5 6 4, again.
/// ```
pub unsafe fn swap_forward<T>(x: *mut T, y: *mut T, count: usize) {
    let x = x.cast::<MaybeUninit<T>>();
    let y = y.cast::<MaybeUninit<T>>();

    for i in 0..count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let x = unsafe { &mut *x.add(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let y = unsafe { &mut *y.add(i) };

        std::mem::swap(&mut *x, &mut *y);
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
/// [ 1  2  3 :4  5  6 *7  8  9 10 11 12 13 14 15]  // swap <--
///            |        └───/\───| ───────┘
///            └────────────\/───┘
/// [ 1  .  3:13 11 12* 4 ~~~~~~~~~~~~~~ 10 14 15]
/// ```
///
/// In details:
///
/// ```text
///                           <--x     <--y
/// [ 1  .  3: 4  5  6 *7  8  9 10 11 12 13 14 15]  // swap <--       11 12 13
///                              └────────┘
/// [ 1  .  .  .  .  .  .  8  9 13 11 12 10 14 15]  //             13 11 12
///                           └────────┘
/// [ 1  .  .  .  .  .  7  8 12 13 11  9 10 14 15]  //          12 13 11
///                        └────────┘
/// [ 1  .  .  .  .  6  7 11 12 13  8  . 10 14 15]  //       11 12 13
///                     └────────┘
/// [ 1  .  .  .  5  6 13 11 12  7  .  . 10 14 15]  //    13 11 12
///                  └────────┘
/// [ 1  .  .  4  5 12 13 11  6  .  .  . 10 14 15]  // 12 13 11
///               └────────┘
/// [ 1  .  3  4 11  . 13  5 ~~~~~~~~~~~ 10 14 15]  // 11-13 and 5-10 are swaped!
///            └────────┘
/// [ 1  .  3:13 11 12 *4 ~~~~~~~~~~~~~~ 10 14 15]  // and 13 11 12, again.
/// ```
pub unsafe fn swap_backward<T>(x: *mut T, y: *mut T, count: usize) {
    let x = x.add(count).cast::<MaybeUninit<T>>();
    let y = y.add(count).cast::<MaybeUninit<T>>();

    for i in 1..=count {
        // while i <= count {
        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let x = unsafe { &mut *x.sub(i) };

        // SAFETY: By precondition, `i` is in-bounds because it's below `count`
        let y = unsafe { &mut *y.sub(i) };

        std::mem::swap(&mut *x, &mut *y);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn seq_multi<const N: usize>(size: usize) -> Vec<[usize; N]> {
        let mut v = vec![[0; N]; size];
        for i in 0..size {
            v[i] = [i + 1; N];
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

    fn prepare(len: usize, x: usize, y: usize) -> (Vec<usize>, (*mut usize, *mut usize)) {
        let mut v = seq(len);

        unsafe {
            let x = &v[..].as_mut_ptr().add(x - 1);
            let y = &v[..].as_mut_ptr().add(y - 1);

            (v, (x.clone(), y.clone()))
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
    fn copy_correct() {
        let (v, (src, dst)) = prepare(15, 4, 7);

        unsafe { copy(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare(15, 7, 4);

        unsafe { copy(src, dst, 6) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 10, 11, 12, 13, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn block_copy_correct() {
        let (v, (src, dst)) = prepare(15, 4, 7);

        unsafe { block_copy(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare(15, 7, 4);

        unsafe { block_copy(src, dst, 6) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 10, 11, 12, 13, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn byte_copy_correct() {
        let (v, (src, dst)) = prepare(15, 4, 7);

        unsafe { byte_copy(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare(15, 7, 4);

        unsafe { byte_copy(src, dst, 6) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 10, 11, 12, 13, 14, 15];
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

        unsafe { shift_left(src, 7) };

        assert_eq!(v, vec![1, 2, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 15]);

        src = *unsafe { &v[..].as_mut_ptr().add(2) };

        unsafe { shift_left(src, 7) };

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
        let (v, (x, y)) = prepare(15, 4, 7);

        unsafe { swap_forward(x, y, 7) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 5, 6, 4, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn swap_backward_correct() {
        let (v, (x, y)) = prepare(15, 4, 7);

        unsafe { swap_backward(x, y, 7) };

        let s = vec![1, 2, 3, 13, 11, 12, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (x, y)) = prepare(15, 1, 7);

        unsafe { swap_backward(x, y, 9) };

        let s = vec![13, 14, 15, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(v, s);

        let (v, (x, y)) = prepare(15, 1, 8);

        unsafe { swap_backward(x, y, 8) };

        let s = vec![15, 9, 10, 11, 12, 13, 14, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(v, s);
    }
}
