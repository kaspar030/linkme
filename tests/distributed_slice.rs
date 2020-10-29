use linkme::distributed_slice;

#[distributed_slice]
static SHENANIGANS: [i32] = [..];

#[distributed_slice(SHENANIGANS)]
static N: i32 = 9;

#[distributed_slice(SHENANIGANS)]
static NN: i32 = 99;

#[distributed_slice(SHENANIGANS)]
static NNN: i32 = 999;

// creating a slice with sorted entries
// the integer values should end up sorted [0, 1, 2, 3)
// symbol names sorting would be M, MM, MMM, MMMM ([0, 2, 1, 3])
// MMMM has 11 which is alphabetically before 2, linkme should zero pad left the integer within the
// section name.
#[distributed_slice]
static SHENANIGANS_SORTED: [i32] = [..];

#[distributed_slice(SHENANIGANS_SORTED)]
static M: i32 = 0;

#[distributed_slice(SHENANIGANS_SORTED, 2)]
static MM: i32 = 2;

#[distributed_slice(SHENANIGANS_SORTED, 11)]
static MMMM: i32 = 3;

#[distributed_slice(SHENANIGANS_SORTED, 1)]
static MMM: i32 = 1;

#[test]
fn test() {
    assert_eq!(SHENANIGANS.len(), 3);

    let mut sum = 0;
    for n in SHENANIGANS {
        sum += n;
    }

    assert_eq!(sum, 9 + 99 + 999);
}

#[test]
fn test_sorted() {
    assert_eq!(SHENANIGANS_SORTED.len(), 4);

    let expected: [i32; 4] = [0, 1, 2, 3];
    for n in 0..SHENANIGANS_SORTED.len() {
        assert_eq!(expected[n], SHENANIGANS_SORTED[n]);
    }
}

#[test]
fn test_empty() {
    #[distributed_slice]
    static EMPTY: [i32] = [..];

    assert!(EMPTY.is_empty());
}

#[test]
fn test_non_copy() {
    struct NonCopy(i32);

    #[distributed_slice]
    static NONCOPY: [NonCopy] = [..];

    #[distributed_slice(NONCOPY)]
    static ELEMENT: NonCopy = NonCopy(9);

    assert!(!NONCOPY.is_empty());
}
