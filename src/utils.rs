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
pub unsafe fn reverse_slice<T>(p: *mut T, count: usize) {
    let slice = slice::from_raw_parts_mut(p, count);
    slice.reverse();
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
    fn copy_backward_correctness() {
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
    fn copy_forward_correctness() {
        let (v, (src, dst)) = prepare_copy(15, 4, 7);

        unsafe { copy_forward(src, dst, 7) };

        let s = vec![1, 2, 3, 4, 5, 6, 4, 5, 6, 4, 5, 6, 4, 14, 15];
        assert_eq!(v, s);

        let (v, (src, dst)) = prepare_copy(15, 7, 4);

        unsafe { copy_forward(src, dst, 7) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 11, 12, 13, 14, 15];
        assert_eq!(v, s);
    }

    // Swaps:

    #[test]
    fn swap_forward_correctness() {
        let (v, (x, y)) = prepare_swap(15, 4, 7);

        unsafe { swap_forward(x, y, 7) };

        let s = vec![1, 2, 3, 7, 8, 9, 10, 11, 12, 13, 5, 6, 4, 14, 15];
        assert_eq!(v, s);
    }

    #[test]
    fn swap_backward_correctness() {
        let (v, (x, y)) = prepare_swap(15, 4, 7);

        unsafe { swap_backward(x, y, 7) };

        let s = vec![1, 2, 3, 13, 11, 12, 4, 5, 6, 7, 8, 9, 10, 14, 15];
        assert_eq!(v, s);

        let (v, (x, y)) = prepare_swap(15, 1, 7);

        unsafe { swap_backward(x, y, 9) };

        let s = vec![13, 14, 15, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(v, s);
    }
}
