use graph::F32;

#[test]
fn test_compare_gt() {
    let a = F32(0.0);
    let b = F32(1.0);
    assert!(a < b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]
fn test_compare_lt() {
    let a = F32(0.0);
    let b = F32(1.0);
    assert!(b > a);
    assert_eq!(b.cmp(&a), std::cmp::Ordering::Greater);
}

#[test]
fn test_compare_eq() {
    let a = F32(0.0);
    let b = F32(0.0);
    assert!(a == b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
}

#[test]
fn test_compare_ne() {
    let a = F32(0.0);
    let b = F32(1.0);
    assert!(a != b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]

fn test_compare_ge() {
    let a = F32(0.0);
    let b = F32(1.0);
    assert!(b >= a);
    assert_eq!(b.cmp(&a), std::cmp::Ordering::Greater);
}

#[test]
fn test_compare_le() {
    let a = F32(0.0);
    let b = F32(1.0);
    assert!(a <= b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]
fn test_neq_inf() {
    let a = F32(f32::INFINITY);
    let b = F32(1.0);
    assert!(a != b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Greater);
}

#[test]
fn test_eq_inf() {
    let a = F32(f32::INFINITY);
    let b = F32(f32::INFINITY);
    assert!(a == b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
}

#[test]
fn test_eq_neg_inf() {
    let a = F32(f32::NEG_INFINITY);
    let b = F32(f32::NEG_INFINITY);
    assert!(a == b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
}

#[test]
fn test_neq_neg_inf() {
    let a = F32(f32::NEG_INFINITY);
    let b = F32(1.0);
    assert!(a != b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]
fn test_neq_nan() {
    let a = F32(f32::NAN);
    let b = F32(1.0);
    assert!(a != b);
}

#[test]
fn test_neq_nans() {
    let a = F32(f32::NAN);
    let b = F32(f32::NAN);
    assert!(a != b);
}

#[test]
#[should_panic]
fn test_cmp_nan() {
    let a = F32(f32::NAN);
    let b = F32(1.0);
    let _ = a.cmp(&b);
}

#[test]
#[should_panic]
fn test_cmp_nans() {
    let a = F32(f32::NAN);
    let b = F32(f32::NAN);
    let _ = a.cmp(&b);
}
