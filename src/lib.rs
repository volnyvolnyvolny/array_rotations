#![doc = include_str!("../README.md")]
#![feature(sized_type_properties)]

//use std::cmp::Ordering;
use std::mem::MaybeUninit;
use std::mem::SizedTypeProperties;

use std::cmp;

//use std::mem::{MaybeUninit, SizedTypeProperties};
use std::ptr;
use std::slice;

use gcd::Gcd;

/// # Triple reversal rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
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
///        left = 9            mid  right = 6
/// [ 1  2  3  4  5  6 :7  8  9,10 11 12 13 14 15]
///   ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓                      reverse left
/// [ 9  8  7  6  5  4  3  2  1 10 11 12 13 14 15]
///                              ↓  ↓  ↓  ↓  ↓  ↓    reverse right
/// [ 9  8  7  6  5  4  3  2  1 15 14 13 12 11 10]
///   ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓    reverse all
/// [10 11 12 13 14 15 :1  2  3, 4  5  6  7  8  9]
/// ```
pub unsafe fn ptr_reversal_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    unsafe fn reverse_slice<T>(p: *mut T, size: usize) {
       let slice = unsafe{ slice::from_raw_parts_mut(p, size) };
       slice.reverse();
    }

    // SAFETY: `[mid - left, mid + right)` is valid for reading and writing.
    unsafe {
        reverse_slice(mid.sub(left), left);
        reverse_slice(mid,           right);
        reverse_slice(mid.sub(left), left + right);
    };
}

/// # Gries-Mills Rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "In some cases this rotation outperforms the classic triple reversal rotation
/// while making fewer moves. You swap the smallest array linearly towards its
/// proper location, since the blocks behind it are in the proper location you
/// can forget about them. What remains of the larger array is now the smallest
/// array, which you rotate in a similar manner, until the smallest side shrinks
/// to `0` elements. Its first known publication was in `1981` by David Gries and
/// Harlan Mills." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing. 
///
/// ## Example
///
/// ```text
///           left = 9         mid   right = 6
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\/┬──────────────┐
/// [ 1  ...3 10          ...15  4 ~~~~~~~~~~~~ 9]
///
///    l = 3        r = 6
/// [ 1...  3,10... 12...    15] 4 ~~~~~~~~~~~~ 9    swap
///   └─────┴/\┴─────┘
///   ┌─────┬\/┬─────┐
/// [10 ~~ 12  1...  3 13... 15] 4 ~~~~~~~~~~~~ 9
///
///             l = 3   r = 3
///  10 ~~ 12[ 1  ...3,13 ...15] 4 ~~~~~~~~~~~~ 9    swap
///            └─────┴/\┴─────┘
///            ┌─────┬\/┬─────┐
///  10 ~~ 12[13 ~~ 15  1 ~~~ 3] 4 ~~~~~~~~~~~~ 9
///
/// [10...          15: 1...  3  4...           9]
/// ```
pub unsafe fn ptr_griesmills_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    if (right == 0) || (left == 0) {
        return;
    }

    // SAFETY: all operations are made inside `[mid - left, mid + right)`
    unsafe {
        if left < right {
            ptr::swap_nonoverlapping(mid.sub(left), mid, left);
            ptr_griesmills_rotate(left, mid.add(left), right - left);
        } else {
            ptr::swap_nonoverlapping(mid, mid.sub(right), right);
            ptr_griesmills_rotate(left - right, mid.sub(right), right);
        }
    }
}

/// # Successive rotation (recursive variant)
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "First described by *Gries and Mills* in *1981*, this rotation is very similar to
/// the Gries-Mills rotation but performs non-linear swaps. It is implemented as
/// the *Piston Rotation* in the benchmark, named after a loop optimization that
/// removes up to `log n` branch mispredictions by performing both a left and
/// rightward rotation in each loop." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing. 
///
/// ## Example
///
/// ```text
///           left = 9         mid  right = 6
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
///   └──────────────┴/\/\/\/\/\/┴──────────────┘
///   ┌──────────────┬\/\/\/\/\/\┬──────────────┐
/// [10 ~~~~~~~~~~~ 15: 7...  9  1...           6]
///
///                      l = 3        r = 6
///  10 ~~~~~~~~~~~ 15[ 7  ...9, 1      : 4  ...6]   swap
///                     └─────┴/\/\/\/\/\/┴─────┘
///                     ┌─────┬\/\/\/\/\/\┬─────┐
///  10 ~~~~~~~~~~~ 15[ 4  ...6  1  ...3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10 ~~~~~~~~~~~ 15[ 4  ...6, 1  ...3] 7 ~~~ 9    swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬\/┬─────┐
///  10 ~~~~~~~~~~~ 15[ 1 ~~~~~~~~~~~~ 6] 7 ~~~ 9
///
/// [10...          15: 1...                    9]
/// ```
pub unsafe fn ptr_piston_rotate_rec<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    if (right == 0) || (left == 0) {
        return;
    }

    // SAFETY: all operations are made inside `[mid-left, mid+right)`
    unsafe {
        if left < right {
            ptr::swap_nonoverlapping(mid.sub(left), mid.add(right).sub(left), left);
            ptr_piston_rotate_rec(left, mid, right - left);
        } else {
            ptr::swap_nonoverlapping(mid, mid.sub(left), right);
            ptr_piston_rotate_rec(left - right, mid, right);
        }
    }
}

/// # Successive Rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// # Algorithm
///
/// "First described by *Gries and Mills* in *1981*, this rotation is very similar to
/// the *Gries-Mills rotation* but performs non-linear swaps. It is implemented as
/// the *Piston Rotation* in the benchmark, named after a loop optimization that
/// removes up to `log n` branch mispredictions by performing both a left and
/// rightward rotation in each loop." <<https://github.com/scandum/rotate>>
///
/// # Safety
///
/// The specified range must be valid for reading and writing. 
///
/// # Example
///
/// ```text
///           left = 9         mid  right = 6
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
///   └──────────────┴/\/\/\/\/\/┴──────────────┘
///   ┌──────────────┬\/\/\/\/\/\┬──────────────┐
/// [10 ~~~~~~~~~~~ 15: 7...  9  1...           6]
///
///                       l = 3        r = 6
///  10 ~~~~~~~~~~~ 15[ 7  ...9, 1      :    ...6]   swap
///                     └─────┴/\/\/\/\/\/┴─────┘
///                     ┌─────┬\/\/\/\/\/\┬─────┐
///  10 ~~~~~~~~~~~ 15[ 4  ...6  1  ...3  7 ~~~ 9]
///
///                       l = 3   r = 3
///  10 ~~~~~~~~~~~ 15[ 4  ...6; 1  ...3] 7 ~~~ 9    swap
///                     └─────┴/\┴─────┘
///                     ┌─────┬\/┬─────┐
///  10 ~~~~~~~~~~~ 15[ 1 ~~~~~~~~~~~~ 6] 7 ~~~ 9
///
/// [10...          15: 1...                    9]
/// ```
pub unsafe fn ptr_piston_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    let mut l = left as isize;
    let mut r = right as isize;

    // SAFETY: all operations are made inside `[mid-left, mid+right)`
    unsafe {
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
}

/// # Helix rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
/// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
/// right.
///
/// ## Algorithm
///
/// "The helix rotation has similarities with the Gries-Mills
/// rotation but has a distinct sequential movement pattern. It is
/// an improvement upon the Grail rotation by merging the two inner
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
///           left = 9         mid   right = 6
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   swap
///            └──────────────┴/\┴──────────────┘
///            ┌──────────────┬\/┬──────────────┐
/// [ 1  ...3 10          ...15  4 ~~~~~~~~~~~~ 9]
///
///    l = 3        r = 6
/// [ 1  ...3,10      :13 ...15] 4 ~~~~~~~~~~~~ 9    swap
///   └─────┴/\/\/\/\/\/┴─────┘
///   ┌─────┬\/\/\/\/\/\┬─────┐
/// [13 ...15 10 ...12  1 ~~~ 3] 4 ~~~~~~~~~~~~ 9
///
///    l = 3    r = 3
/// [13 ...15;10 ...12] 1 ~~~ 3  4 ~~~~~~~~~~~~ 9    swap
///   └─────┴/\┴─────┘
///   ┌─────┬\/┬─────┐
/// [10 ~~~~~~~~~~~ 15] 1 ~~~ 3  4 ~~~~~~~~~~~~ 9
///
/// [10          ...15: 1  ...3  4 ~~~~~~~~~~~~ 9]
/// ```
pub unsafe fn ptr_helix_rotate<T>(mut left: usize, mid: *mut T, mut right: usize) {
    if T::IS_ZST {
        return;
    }

    if (right == 0) || (left == 0) {
        return;
    }

    let start: usize = 0;
    let end = mid.add(right);

    // loop {
        // if (left > right) {
            // if (right <= 1) {
                // break;
            // }
// 
            // while (mid > start) {
                // mid = mid.sub(1);
                // swap = mid.read();
                // end -= 1;
                // mid.write(end.read());
                // end.write(swap);
            // }
// 
            // mid += (left %= right);
            // right = end - mid;
        // } else {
            // if (left <= 1) {
                // break;
            // }
// 
            // while (mid < end) {
                // swap = array[mid];
                // array[mid++] = array[start];
                // array[start++] = swap;
            // }
// 
            // mid -= (right %= left);
            // left = mid - start;
        // }
    // }

    // if (left && right) {
        // ptr_aux_rotate(left, mid.sub(left).add(start), right);
    // }
// 
    // // SAFETY: all operations are inside `[mid-left, mid+right)`
    // unsafe {
        // if left < right {
            // ptr::swap_nonoverlapping(mid.sub(left), mid.add(left), left);
            // ptr_helix_rotate(left, mid, right - left);
        // } else {
            // ptr::swap_nonoverlapping(mid, mid.sub(right), right);
            // ptr_helix_rotate(left - right, mid.sub(right), right);
        // }
    // }
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
///        left = 9    dim     mid   right = 6
/// [ 1  2  3  4  5  6 :7  8  9,            12-15]
///                                          └──┴───────┬─────┐
/// [              1-6 :7...  9  .  .  .  .  .  .]    [10... 15]
///                └────┬─────┴─────────────────┐                  move
/// [ .  .  .  .  .  . :1 ~~~~~~~~~~~~~~~~~~~~~ 9]    [10-15   ]
///   ┌──────────────┬──────────────────────────────────┴──┘
/// [10 ~~~~~~~~~~~ 15 :1 ~~~~~~~~~~~~~~~~~~~~~ 9]
/// ```
///
/// ```text
///                                  mid
///           left = 11              | right = 4
/// [ 1  2  3  4: 5  6  7  8  9 10 11,      12-15]
///                                          └──┴───────┬─────┐
/// [ 1...      :                  11  .  .  .  .]    [12... 15]
///   └───────────┬─────────────────┴───────────┐                  move
/// [ .  .  .  .: 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11]    [12-15   ]
///   ┌────────┬────────────────────────────────────────┴──┘
/// [12 ~~~~~ 15: 1...                         11]
/// ```
///
/// ```text
///
///    left = 4  mid         right = 11
/// [      12-15, 1  2  3  4  5  6  7: 8  9 10 11]
///         └──┴────────────────────────────────────────┬─────┐
/// [ .  .  .  .  1...               :         11]    [12... 15]
///   ┌───────────┴─────────────────┬───────────┘                  move
/// [ 1 ~~~~~~~~~~~~~~~~~~~~~~~~~~ 11: .  .  .  .]    [12-15   ]
///                                    ┌────────┬───────┴──┘
/// [ 1                         ...11:12 ~~~~~ 15]
/// ```
pub unsafe fn ptr_aux_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    if (right == 0) || (left == 0) {
        return;
    }

    let mut v = Vec::<T>::with_capacity(cmp::min(left, right));
    let buf = v.as_mut_ptr();

    // SAFETY: `mid-left <= mid-left+right < mid+right`
    let dim = unsafe { mid.sub(left).add(right) };

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
        if left <= right {
            // 1)
            ptr::copy_nonoverlapping(mid.sub(left), buf, left);
            // 2)
            ptr::copy(mid, mid.sub(left), right);
            // 3)
            ptr::copy_nonoverlapping(buf, dim, left);
        } else {
            // SAFETY: same reasoning as above but with `left` and `right` reversed
            ptr::copy_nonoverlapping(mid, buf, right);
            ptr::copy(mid.sub(left), dim, left);
            ptr::copy_nonoverlapping(buf, mid.sub(left), right);
        }
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
/// "This is a slightly more complex auxiliary rotation than
/// auxiliary rotation that reduces the maximum auxiliary memory
/// requirement from `50%` to `33.(3)%`. If the overlap between the
/// two halves is smaller than the halves themselves it copies
/// the overlap to swap memory instead. Its first known publication
/// was in *2021* by *Igor van den Hoven*." <<https://github.com/scandum/rotate>>
///
/// ## Safety
///
/// The specified range must be valid for reading and writing. 
///
/// # Example:
///
/// Case 1 (bridge (`s`) < left || right):
///
/// ```text
///          left = 9          mid  right = 6
/// [ 1  2  3  4  5  6:     7-9,10 11 12 13 14 15]
///                         └─┴────────────────────────┬─────┐
///   a-->              b-->    c-->                   |     |
/// [ 1           ...6: .  .  . 10...          15]    [7  8  9]
///   └─────────────────┐       |
///   ╭─ a ──────────── |─ b ───╯  c
/// [10  2        ...6: 1  .  .  . 11...       15]    [7...   ]
///      └─────────────────┐       |
///      ╭─ a ──────────── |─ b ───╯  c
/// [10 11  3     ...6: 1  2  .  .  . 12...    15]    [7...   ]
///         └─────────────────┐       |
///         ╭─ a ──────────── |─ b ───╯  c
/// [10 ~~ 12  4  ...6: 1 ~~~ 3  .  .  . 13... 15]    [7...   ]
///            └─────────────────┐       |
///            ╭─ a ──────────── |─ b ───╯  c
/// [10 ~~~~~ 13  5  6: 1 ~~~~~~ 4  .  .  . 14 15]    [7...   ]
///               └─────────────────┐       |
///               ╭─ a ──────────── |─ b ───╯  c
/// [10 ~~~~~~~~ 14  6: 1 ~~~~~~~~~ 5  .  .  . 15]    [7...   ]
///                  └─────────────────┐       |
///                  ╭──────────────── |─ b ───╯
/// [10 ~~~~~~~~~~~ 15: 1 ~~~~~~~~~~~~ 6  .  .  .]    [7-9    ]
///                                       ┌─────┬──────┴─┘
/// [10          ...15: 1           ...6  7  ~  9]
/// ```
///
/// ```text
///       left = 6    mid           right = 6
/// [10 11 12 13 14 15,     1-3: 4  5  6  7  8  9]
///                         └─┴────────────────────────┬─────┐
///                     b       c               d      |     |
/// [10          ...15  .  .  .: 4...           9]    [1  2  3]
///                  ╰─────────────────────────╮|
///                  b       c┌───────────── d |┘
/// [10...       14  .: .  .  9  4...        8 15]    [1...   ]
///               ╰─────────────────────────╮|
///               b       c┌───────────── d |┘
/// [10 11... 13  .  .: .  8  9  4...     7 14 15]    [1...   ]
///            ╰─────────────────────────╮|
///            b       c┌───────────── d |┘
/// [10 11 12  .  .  .: 7 ~~~ 9  4...  6 13 ~~ 15]    [1...   ]
///         ╰─────────────────────────╮|
///         b       c┌───────────── d |┘
/// [10 11  .  .  .  6:~~~~~~ 9  4  5 12 ~~~~~ 15]    [1...   ]
///      ╰─────────────────────────╮|
///      b       c┌───────────── d |┘
/// [10  .  .  .  5 ~~:~~~~~~ 9  4 11 ~~~~~~~~ 15]    [1-3    ]
///   ╰─────────────────────────╮|
///           c┌───────────── d |┘
/// [ .  .  .  4  5 ~~:~~~~~~ 9 10 ~~~~~~~~~~~ 15]    [1-3    ]
///   ┌─────┬──────────────────────────────────────────┴─┘
/// [ 1  ~  3  4...           9 10...          15]
/// ```
pub unsafe fn ptr_bridge_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    let mut v = Vec::<T>::with_capacity(cmp::min(left, right));
    let buf = v.as_mut_ptr();

    let bridge = left.abs_diff(right);

    if cmp::min(left, right) <= bridge {
        ptr_aux_rotate(left, mid, right); 
        return;
    }

    let mut a = mid.sub(left);
    let mut b = mid;
    let mut c = mid.sub(left).add(right);
    let mut d = mid.add(right);

    if left > right {
        // SAFETY: `[mid - left + right, mid - left + right + bridge)` is valid for reading
        unsafe {
            ptr::copy_nonoverlapping(c, buf, bridge);
        }

        // SAFETY: `[mid - left, mid + right)` is valid for reading and writing
        unsafe {
            for _ in 0..right {
                c.write(a.read());
                a.write(b.read());
                a = a.add(1);
                b = b.add(1);
                c = c.add(1);
            }
        }

        // SAFETY: `[mid + right - bridge, mid + right)` is valid for writing
        unsafe {
            ptr::copy_nonoverlapping(buf, d.sub(bridge), bridge);
        }
    } else if left < right {
        // SAFETY: `[mid, mid + bridge)` is valid for reading
        unsafe {
            ptr::copy_nonoverlapping(b, buf, bridge);
        }

        // SAFETY: `[mid - left, mid + right)` is valid for reading and writing
        unsafe {
            for _ in 0..left {
                b = b.sub(1);
                c = c.sub(1);
                d = d.sub(1);
                c.write(d.read());
                d.write(b.read());
            }
        }

        // SAFETY: `[mid - left, mid - left + bridge)` is valid for writing
        unsafe {
            ptr::copy_nonoverlapping(buf, a, bridge);
        }
    } else {
        // SAFETY:
        // `left == right` so `[mid-left, mid+right)` is valid for reading and writing
        unsafe {
            ptr::swap_nonoverlapping(mid.sub(left), mid, right);
        }
    }
}

// unsafe fn print<T: std::fmt::Debug>(label: &str, mut p: *const T, size: usize) {
    // print!("{} [", label);
    // for _ in 0..size {
        // print!("{:?} ", *p);
        // p = p.add(1);
    // }
    // println!("]");
// }

/// # Juggling rotation
///
/// Rotates the range `[mid-left, mid+right)` such that the element at
/// `mid` becomes the first element. Equivalently, rotates the range
/// `left` elements to the left or `right` elements to the right.
///
/// ## Algorithm
///
/// "Also known as the dolphin algorithm. This is a relatively complex
/// and inefficient way to rotate in-place, though it does so in the
/// minimal number of moves. Its first known publication was in *1966*.
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
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]
///   |        |        |        |        └─────────────────┐
///   |        |        |        └──────────────────┐       |
///   |        |        └─────────────────┐         |       |
///   |        └─────────────────┐        |         |       |
///   └─────────────────┐        |        |         |       |
/// ~──────────┐        |        |        |         |       |
/// ~─┐        |        |        |        |         |       |
/// [10  2  3 13  5  6: 1  8  9, 4 11 12  7 14 15][10  2  3 13...
///      |        |        |        |        └──────────────────┐
///      |        |        |        └──────────────────┐        |
///      |        |        └─────────────────┐         |        |
///      |        └─────────────────┐        |         |        |
///      └─────────────────┐        |        |         |        |
/// ~─────────────┐        |        |        |         |        |
/// ~────┐        |        |        |        |         |        |
/// [10 11  3 13 14  6: 1  2  9, 4  5 12  7  8 15][10 11  3 13 14...
///         |        |        |        |        └──────────────────┐
///         |        |        |        └──────────────────┐        |
///         |        |        └─────────────────┐         |        |
///         |        └─────────────────┐        |         |        |
///         └─────────────────┐        |        |         |        |
/// ~────────────────┐        |        |        |         |        |
/// ~───────┐        |        |        |        |         |        |
/// [   ...12    ...15:    ...3     ...6     ...9][   ...12    ...15:..
/// ```
pub unsafe fn ptr_juggling_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if left == 0 {
        return;
    }

    let len = left + right;

    let mut a;
    let mut b;
    let mut c = mid.sub(left);
    let d = mid.sub(left).add(left.gcd(len));

    let mut swap;

    while c < d {
        swap = c.read();
        a = c;

        loop {
            b = a.add(left);

            if b >= mid.add(right) {
                b = b.sub(len);

                if b == c {
                    break;
                }
            }
            a.replace(b.read());
            a = b;
        }

        a.replace(swap);
        c = c.add(1);
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
///  ls-->                <--le rs-->       <--re
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
///   ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮|    
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆╯
///   ┆ ls                le  |   |rs       re┆
/// [10  2...              8  1,15╯11...    14╰~9]   //(ls, le, re, rs)
///      ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮|
///      ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆╯
///      ┆ ls          le  |        | rs re┆
/// [10 11  3...        7  2  1,15 14 12 13╰~8  9]
///         ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ╮|
///         ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|╯
///         ┆ ls    le  |             re┆ 
/// [10 ~~ 12  4...  6  3  2  1,15 14 13╰~7 ~~~ 9]   //(ls, le, re)
///            ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
///            ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
///            ┆ ls  |             re┆
/// [10 ~~~~~ 13  5  4  3  2  1,15 14╰~6 ~~~~~~ 9]   //(ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
/// [10 ~~~~~ 13 14 15: 1  2  3, 4  5  6 ~~~~~~ 9]
/// ```
///
/// Case: `left < right`, `6 - 9`.
///
/// ```text
///  ls-->       <--le rs-->                <--re
/// [ 1  2  3  4  5  6, 7  8  9:10 11 12 13 14 15]   //(re -> rs -> ls -> le -> re)
///   | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
///   ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
///     ┆ls      le |   | rs                re  ┆
/// [ 7~╯2     ...5 ╰1,15  8             ...14  6]   //(re, rs, ls, le)
///      | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
///      ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
///        ls le|          | rs          re  ┆
/// [ 7  8~╯3  4╰~2  1,15 14  9       ...13  5  6]
///         |╭ ┆┈┈┈┈┈┈┈┈┈┈┈╯        ╭─────╯
///         ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
///          ┆ls              | rs    re  ┆
/// [ 7 ~~~ 9╯╰3  2  1,15 14 13 10 ...12  4 ~~~ 6]   //(re, rs, ls)
///            ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
///             ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
///             ┆ ls             | re  ┆
/// [ 7 ~~~~~ 10╯ 2  1,15 14 13 12 11  3 ~~~~~~ 6]   //(re, ls)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
/// [ 7 ~~~~~ 10 11 12,13 14 15: 1  2  3 ~~~~~~ 6]
/// ```
///
/// Case: `left > right`, `8 - 7`.
///
/// ```text
///  ls-->             <--le rs-->          <--re
/// [ 1  2  3  4  5  6  7: 8, 9 10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
///   ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|    
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭───────────── ┆╯
///   ┆ ls             le  |   |rs          re┆
/// [ 9  2...           7: 1,15╯10 11 12 13 14╰~8]   //(ls, le, re, rs)
///      ╰────────╮     ╰┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈┈╮|    
///      ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈╯ ╭─────── ┆╯
///      ┆ ls       le  |         |rs    re┆
/// [ 9 10  3...     6  2: 1,15 14╯11 12 13╰~7  8]   //(ls, le, re, rs)
///         ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮|    
///         ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆╯
///         ┆ ls le  |               |rs┆
/// [ 9 ~~ 11  4  5╮ 3  2: 1,15 14 13╯12╰~6 ~~~ 8]   //(ls, le, rs)
///            ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
///            ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆╯
///            | ls                re┆ 
/// [ 9 ~~~~~ 12  4  3  2: 1,15 14 13╰~5 ~~~~~~ 8]   //(ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯   
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮   
/// [ 9 ~~~~~ 12 13 14 15: 1, 2  3  4  5 ~~~~~~ 8]
/// ```
pub unsafe fn ptr_contrev_rotate<T>(left: usize, mid: *mut T, right: usize) {
    if T::IS_ZST {
        return;
    }

    if left == 0 || right == 0 {
        return;
    }

    // SAFETY:
    // `[ls, le) := [mid-left, mid-1]` is the left part;
    // `[rs, re) := [mid, mid+right-1]` is the right part.
    // `[mid-left, mid+right)` is valid for reading and writing
    let (mut ls, mut le) = unsafe{ (mid.sub(left), mid.sub(1)) };
    let (mut rs, mut re) = unsafe{ (mid, mid.add(right).sub(1)) };

    // SAFETY:
    //
    // All operations are within `[mid-left, mid+right)` which
    // is valid for reading and writing.
    unsafe {
        if left == right {
            ptr::swap_nonoverlapping(mid, mid.sub(left), right);
        } else {
            let half_min = cmp::min(left, right) / 2;
            let half_max = cmp::max(left, right) / 2;

            for _ in 0..half_min { // Permutation (ls, le, re, rs)
                ls.write(
                    rs.replace(
                        re.replace(
                            le.replace(ls.read())
                        )
                    )
                );

                ls = ls.add(1); le = le.sub(1);
                rs = rs.add(1); re = re.sub(1);
            }

            if left > right {
                for _ in 0..half_max-half_min { //(ls, le, re)
                    ls.write(
                        re.replace(
                            le.replace(ls.read())
                        )
                    );

                    ls = ls.add(1); le = le.sub(1);
                    re = re.sub(1);
                }
            } else {
                for _ in 0..half_max-half_min { //(rs, re, ls)
                    ls.write(
                        rs.replace(
                            re.replace(ls.read())
                        )
                    );

                    ls = ls.add(1);
                    rs = rs.add(1); re = re.sub(1);
                }
            }

            let center = slice::from_raw_parts_mut(ls, re.offset_from(ls).abs() as usize + 1);
            center.reverse();
        }
    }
}

/// # Trinity (Conjoined triple reversal + Bridge rotations)
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
/// "The trinity rotation (aka conjoined triple reversal) is derived from the triple reversal
/// rotation. Rather than three separate reversals it conjoins the three reversals, improving
/// locality and reducing the number of moves. Optionally, if the overlap is smaller than
/// `32 * size_of(usize)`, it skips the trinity rotation and performs an auxiliary
/// or bridge rotation on stack memory. Its first known publication was in 2021 by Igor van den Hoven."
/// <<https://github.com/scandum/rotate>>
///
/// ## Example
///
/// Case: `right > left`, `9 - 6`.
///
/// ```text
///  ls-->                <--le rs-->       <--re
/// [ 1  2  3  4  5  6: 7  8  9,10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
///   ╰───────────╮           ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈╮|    
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰───────────╮┈┈╯╭────────── ┆╯
///   ┆ ls                le  |   |rs       re┆
/// [10  2...              8  1,15╯11...    14╰~9]   //(ls, le, re, rs)
///      ╰────────╮        ╰┈┈┈┈┈╭┈┈╯ ┈┈┈┈┈╮|
///      ╭┈┈┈┈┈┈┈ ╰────────╮┈┈┈┈┈╯  ╭───── ┆╯
///      ┆ ls          le  |        | rs re┆
/// [10 11  3...        7  2  1,15 14 12 13╰~8  9]
///         ╰─────╮     ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ╮|
///         ╭┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭|╯
///         ┆ ls    le  |             re┆ 
/// [10 ~~ 12  4...  6  3  2  1,15 14 13╰~7 ~~~ 9]   //(ls, le, re)
///            ╰──╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭┈┈╯
///            ╭┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯┆
///            ┆ ls  |             re┆
/// [10 ~~~~~ 13  5  4  3  2  1,15 14╰~6 ~~~~~~ 9]   //(ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
/// [10 ~~~~~ 13 14 15: 1  2  3, 4  5  6 ~~~~~~ 9]
/// ```
///
/// Case: `left < right`, `6 - 9`.
///
/// ```text
///  ls-->       <--le rs-->                <--re
/// [ 1  2  3  4  5  6, 7  8  9:10 11 12 13 14 15]   //(re -> rs -> ls -> le -> re)
///   | ╭┈┈┈┈┈┈┈┈┈┈┈ ┆┈┈╯           ╭───────────╯
///   ╰─┆ ──────────╮╰┈┈╭───────────╯ ┈┈┈┈┈┈┈┈┈┈╮
///     ┆ls      le |   | rs                re  ┆
/// [ 7~╯2     ...5 ╰1,15  8             ...14  6]   //(re, rs, ls, le)
///      | ╭┈┈┈┈┈ ┆┈┈┈┈┈┈┈┈╯        ╭────────╯
///      ╰─┆ ───╮ ╰┈┈┈┈┈┈┈┈╭────────╯ ┈┈┈┈┈┈┈╮
///        ls le|          | rs          re  ┆
/// [ 7  8~╯3  4╰~2  1,15 14  9       ...13  5  6]
///         |╭ ┆┈┈┈┈┈┈┈┈┈┈┈╯        ╭─────╯
///         ╰┆╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭─────╯ ┈┈┈┈╮
///          ┆ls              | rs    re  ┆
/// [ 7 ~~~ 9╯╰3  2  1,15 14 13 10 ...12  4 ~~~ 6]   //(re, rs, ls)
///            ╰┈╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯  ╭──╯
///             ┆╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╭──╯ ─╮
///             ┆ ls             | re  ┆
/// [ 7 ~~~~~ 10╯ 2  1,15 14 13 12 11  3 ~~~~~~ 6]   //(re, ls)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮
/// [ 7 ~~~~~ 10 11 12,13 14 15: 1  2  3 ~~~~~~ 6]
/// ```
///
/// Case: `left > right`, `8 - 7`.
///
/// ```text
///  ls-->             <--le rs-->          <--re
/// [ 1  2  3  4  5  6  7: 8, 9 10 11 12 13 14 15]   //(ls -> le -> re -> rs -> ls)
///   ╰───────────╮        ╰┈┈┆ ┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|    
///   ╭┈┈┈┈┈┈┈┈┈┈ ╰────────╮┈┈╯╭───────────── ┆╯
///   ┆ ls             le  |   |rs          re┆
/// [ 9  2...           7: 1,15╯10 11 12 13 14╰~8]   //(ls, le, re, rs)
///      ╰────────╮     ╰┈┈┈┈┈┈┈┆ ┈┈┈┈┈┈┈┈┈╮|    
///      ╭┈┈┈┈┈┈┈ ╰─────╮┈┈┈┈┈┈┈╯ ╭─────── ┆╯
///      ┆ ls       le  |         |rs    re┆
/// [ 9 10  3...     6  2: 1,15 14╯11 12 13╰~7  8]   //(ls, le, re, rs)
///         ╰─────╮  ╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆ ┈┈╮|    
///         ╭┈┈┈┈ ╰──╮┈┈┈┈┈┈┈┈┈┈┈┈┈┈╯╭─ ┆╯
///         ┆ ls le  |               |rs┆
/// [ 9 ~~ 11  4  5╮ 3  2: 1,15 14 13╯12╰~6 ~~~ 8]   //(ls, le, rs)
///            ╰──╮╰┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈╮|
///            ╭┈ |┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆╯
///            | ls                re┆ 
/// [ 9 ~~~~~ 12  4  3  2: 1,15 14 13╰~5 ~~~~~~ 8]   //(ls, re)
///               ╰┈┈╰┈┈╰┈╮┆╭┈╯┈┈╯┈┈╯   
///               ╭┈┈╭┈┈╭┈╰┆┈┈╮┈┈╮┈┈╮   
/// [ 9 ~~~~~ 12 13 14 15: 1, 2  3  4  5 ~~~~~~ 8]
/// ```
pub unsafe fn ptr_trinity_rotate<T>(left: usize, mid: *mut T, right: usize) {
    type BufType = [usize; 32];

    if T::IS_ZST {
        return;
    }

    if cmp::min(left, right) <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>() {
        ptr_aux_rotate(left, mid, right);
        return;
    }

    let d = right.abs_diff(left);

    if d <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>() && d > 3 {
        ptr_bridge_rotate(left, mid, right);
        return;
    }

    // SAFETY:
    // `[ls, le) := [mid-left, mid-1]` is the left part;
    // `[rs, re) := [mid, mid+right-1]` is the right part.
    // `[mid-left, mid+right)` is valid for reading and writing
    let (mut ls, mut le) = unsafe{ (mid.sub(left), mid.sub(1)) };
    let (mut rs, mut re) = unsafe{ (mid, mid.add(right).sub(1)) };

    // SAFETY:
    //
    // All operations are within `[mid-left, mid+right)` which
    // is valid for reading and writing.
    unsafe {
        if left == right {
            ptr::swap_nonoverlapping(mid, mid.sub(left), right);
        } else {
            let half_min = cmp::min(left, right) / 2;
            let half_max = cmp::max(left, right) / 2;

            for _ in 0..half_min { // Permutation (ls, le, re, rs)
                ls.write(
                    rs.replace(
                        re.replace(
                            le.replace(ls.read())
                        )
                    )
                );

                ls = ls.add(1); le = le.sub(1);
                rs = rs.add(1); re = re.sub(1);
            }

            if left > right {
                for _ in 0..half_max-half_min { //(ls, le, re)
                    ls.write(
                        re.replace(
                            le.replace(ls.read())
                        )
                    );

                    ls = ls.add(1); le = le.sub(1);
                    re = re.sub(1);
                }
            } else {
                for _ in 0..half_max-half_min { //(rs, re, ls)
                    ls.write(
                        rs.replace(
                            re.replace(ls.read())
                        )
                    );

                    ls = ls.add(1);
                    rs = rs.add(1); re = re.sub(1);
                }
            }

            for _ in 0..re.offset_from(ls).abs() / 2 { //(re, ls)
                ls.write(
                    re.replace(ls.read())
                );

                ls = ls.add(1);
                re = re.sub(1);
            }


//            let center = slice::from_raw_parts_mut(ls, re.offset_from(ls).abs() as usize + 1);
//            center.reverse();
        }
    }
}

// /// # Combined rotation
// ///
// /// Rotates the range `[mid-left, mid+right)` such that the element at `mid` becomes the first
// /// element. Equivalently, rotates the range `left` elements to the left or `right` elements to the
// /// right.
// ///
// /// # Safety
// ///
// /// The specified range must be valid for reading and writing.
// ///
// /// # Algorithm
// ///
// /// This rotation combines four algorithms:
// ///
// /// 1. **Auxiliary rotation** — if left or right side fits in buffer (`32 * size_of(usize)` bytes);
// /// 2. **Bridge rotation** — if the overlap fits in buffer;
// /// 3. **Piston rotation** — otherwise.
// pub unsafe fn ptr_comb_rotate<T>(left: usize, mid: *mut T, right: usize) {
    // type BufType = [usize; 32];
// 
    // if T::IS_ZST {
        // return;
    // }
// 
    // if cmp::min(left, right) <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>() {
        // ptr_aux_rotate(left, mid, right);
        // return;
    // }
// 
    // let d = right.abs_diff(left);
// 
    // if d <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>() && d > 3 {
        // ptr_bridge_rotate(left, mid, right);
        // return;
    // }
// 
    // ptr_piston_rotate(left, mid, right);
// }

// // 2021 - Drill rotation by Igor van den Hoven (grail derived with piston and helix loops)
// 
// void drill_rotation(int *array, size_t left, size_t right)
// {
    // int swap;
    // size_t start = 0;
    // size_t end = left + right;
    // size_t mid = left;
    // size_t loop;
//
    // while (left > 1)
    // {
        // if (left <= right)
        // {
            // loop = end - mid - (right %= left);
//
            // do
            // {
                // swap = array[mid]; array[mid++] = array[start]; array[start++] = swap;
            // }
            // while (--loop);
        // }
// 
        // if (right <= 1)
        // {
            // break;
        // }
// 
        // loop = mid - start - (left %= right);
// 
        // do
        // {
            // swap = array[--mid]; array[mid] = array[--end]; array[end] = swap;
        // }
        // while (--loop);
    // }
// 
    // if (left && right)
    // {
        // stack_rotation(array + start, left, right);
    // }
// }


/// # Default rotation
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
/// left = 10, right = 6
/// the `^` indicates an element in its final place
/// 6 7 8 9 10 11 12 13 14 15 . 0 1 2 3 4 5
/// after using one step of the above algorithm (The X will be overwritten at the end of the round,
/// and 12 is stored in a temporary):
/// X 7 8 9 10 11 6 13 14 15 . 0 1 2 3 4 5
///               ^
/// after using another step (now 2 is in the temporary):
/// X 7 8 9 10 11 6 13 14 15 . 0 1 12 3 4 5
///               ^                 ^
/// after the third step (the steps wrap around, and 8 is in the temporary):
/// X 7 2 9 10 11 6 13 14 15 . 0 1 12 3 4 5
///     ^         ^                 ^
/// after 7 more steps, the round ends with the temporary 0 getting put in the X:
/// 0 7 2 9 4 11 6 13 8 15 . 10 1 12 3 14 5
/// ^   ^   ^    ^    ^       ^    ^    ^
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
/// [ 5  6  7  8  9 10 11:12 13 14 15 |1  2  3  4]
///                        ├────────┼──┼────────┤ swap
/// [ 5...             11: 1  2  3  4 12 13 14 15]
///                                    ^  ^  ^  ^
/// [ 5  6  7  1  2  3  4: 8  9 10 11 12...
///            ├────────┼──┼────────┤ swap
/// [ 5  6  7  8  9 10 11: 1  2  3  4 12 13 14 15
/// we cannot swap any more, but a smaller rotation problem is left to solve
/// ```
///
/// when `left < right` the swapping happens from the left instead.
pub unsafe fn stable_ptr_rotate<T>(mut left: usize, mut mid: *mut T, mut right: usize) {
    type BufType = [usize; 32];

    if T::IS_ZST {
        return;
    }

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
        } else if cmp::min(left, right) <= std::mem::size_of::<BufType>() / std::mem::size_of::<T>() {
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
        for i in 0..size { v[i] = i+1; }
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

    fn case(rotate: unsafe fn(left: usize, mid: *mut usize, right: usize), size: usize, diff: usize) {
        let (vec, (l, p, r)) = prepare(size, diff);

        let mut s = seq(size);

        s.rotate_left(l);
        unsafe{ rotate(l, p, r) };

        assert_eq!(vec, s);

        unsafe { rotate(r, p.sub(diff), l) };

        s.rotate_right(l);
        assert_eq!(vec, s);
    }

    fn test_correctness(rotate_f: unsafe fn(left: usize, mid: *mut usize, right: usize)) {
        // --empty--
        case(rotate_f,  0,  0);

        // 1  2  3  4  5  6 (7  8  9)10 11 12 13 14 15
        case(rotate_f, 15,  3);
 
        // 1  2  3  4  5  6  7 (8) 9 10 11 12 13 14 15
        case(rotate_f, 15,  1);
 
        // 1  2  3  4  5  6  7)(8  9 10 11 12 13 14
        case(rotate_f, 14,  0);

        // 1  2  3  4 (5  6  7  8  9 10 11)12 13 14 15
        case(rotate_f, 15,  7);

        // 1 (2  3  4  5  6  7  8  9 10 11 12 13 14)15
        case(rotate_f, 15, 13);

        //(1  2  3  4  5  6  7  8  9 10 11 12 13 14 15)
        case(rotate_f, 15, 15);
    }

    #[test]
    fn test_ptr_aux_rotate_correctness() {
        test_correctness(ptr_aux_rotate::<usize>);
    }

    #[test]
    //rust rotate
    fn test_ptr_rotate_correctness() {
       test_correctness(stable_ptr_rotate::<usize>);
    }

    #[test]
    fn test_ptr_bridge_rotate_correctness() {
       test_correctness(ptr_bridge_rotate::<usize>);
    }

    #[test]
    fn test_ptr_reversal_rotate_correctness() {
       test_correctness(ptr_reversal_rotate::<usize>);
    }

    #[test]
    fn test_ptr_griesmills_rotate_correctness() {
       test_correctness(ptr_griesmills_rotate::<usize>);
    }

    #[test]
    fn test_ptr_piston_rotate_rec_correctness() {
       test_correctness(ptr_piston_rotate_rec::<usize>);
    }

    #[test]
    fn test_ptr_piston_rotate_correctness() {
       test_correctness(ptr_piston_rotate::<usize>);
    }

    // #[test]
    // fn test_ptr_comb_rotate_correctness() {
       // test_correctness(ptr_comb_rotate::<usize>);
    // }

    #[test]
    fn test_ptr_contrev_rotate_correctness() {
       test_correctness(ptr_contrev_rotate::<usize>);
    }

    #[test]
    fn test_ptr_trinity_rotate_correctness() {
       test_correctness(ptr_trinity_rotate::<usize>);
    }

    #[test]
    fn test_ptr_juggling_rotate_correctness() {
       test_correctness(ptr_juggling_rotate::<usize>);
    }

    // #[test]
    // fn test_ptr_helix_rotate_correctness() {
       // test_correctness(ptr_helix_rotate::<usize>);
    // }

    // #[test]
    // fn test_ptr_grail_rotate_correctness() {
       // test_correctness(ptr_grail_rotate::<usize>);
    // }

    // #[test]
    // fn test_ptr_drill_rotate_correctness() {
       // test_correctness(ptr_drill_rotate::<usize>);
    // }
}
