//! Some `const fn` helpers to manipulate `str`s, to greatly improve the error message
//! upon providing an incorrect `path_to_this_very_module`.
//!
//! Used in conjunction with `validate_module_path.rs`.

/// Whether `a == b` under `|s| s.bytes().filter(|&b| b != b' ')`.
pub
const
fn eq_modulo_whitespace(
    a: &str,
    b: &str,
) -> bool
{
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut i @ mut j = 0;
    while i < a.len() || j < b.len() {
        while i < a.len() && a[i] == b' ' {
            i += 1;
        }
        while j < b.len() && b[j] == b' ' {
            j += 1;
        }
        match (i < a.len(), j < b.len()) {
            (true, true) if a[i] == b[j] => {},
            (false, false) => {},
            _ => return false,
        }
        i += 1;
        j += 1;
    }
    true
}

/// Return a subslice starting at the first occurrence of `b` (or empty string otherwise).
pub
const
fn find_subslice(s: &str, b: u8) -> &str {
    let mut s = s.as_bytes();
    loop {
        match *s {
            [hd, ref rest @ ..] if hd != b => s = rest,
            _ => match ::core::str::from_utf8(s) {
                Ok(s) => return s,
                _ => panic!("non-UTF8"),
            },
        }
    }
}

pub
const
fn constcat<const N: usize, const NUM_INPUTS: usize>(
    ss: [&str; NUM_INPUTS],
) -> [u8; N]
{
    let ss = {
        let mut xss = [b"" as &[u8]; NUM_INPUTS];
        let mut sum = 0;
        let mut i = 0;
        while i < ss.len() {
            xss[i] = ss[i].as_bytes();
            sum += xss[i].len();
            i += 1;
        }
        if sum != N {
            panic!("Invalid `constcat` length");
        }
        xss
    };
    let mut ret = [b'?'; N];

    let mut i = 0;
    let mut j = 0;
    while j < ss.len() {
        let s = ss[(j, j += 1).0];
        let mut j = 0;
        while j < s.len() {
            ret[(i, i += 1).0] = s[(j, j += 1).0];
        }
    }

    if let Err(_) = ::core::str::from_utf8(&ret) {
        panic!("unreachable: non-UTF8 concat output");
    }

    ret
}
