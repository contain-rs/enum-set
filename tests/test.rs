extern crate enum_set;
#[macro_use]
extern crate enum_set_derive;

use Foo::*;
use std::mem;

use enum_set::{EnumSet, CLike};

#[derive(Copy, Clone, CLike, PartialEq, Debug)]
#[repr(u32)]
enum Foo {
    A, B, C
}

#[test]
fn test_new() {
    let e: EnumSet<Foo> = EnumSet::new();
    assert!(e.is_empty());
}

#[test]
fn test_debug() {
    let mut e = EnumSet::new();
    assert_eq!("{}", format!("{:?}", e));
    e.insert(A);
    assert_eq!("{A}", format!("{:?}", e));
    e.insert(C);
    assert_eq!("{A, C}", format!("{:?}", e));
}

#[test]
fn test_len() {
    let mut e = EnumSet::new();
    assert_eq!(e.len(), 0);
    e.insert(A);
    e.insert(B);
    e.insert(C);
    assert_eq!(e.len(), 3);
    e.remove(&A);
    assert_eq!(e.len(), 2);
    e.clear();
    assert_eq!(e.len(), 0);
}

///////////////////////////////////////////////////////////////////////////
// intersect

#[test]
fn test_two_empties_do_not_intersect() {
    let e1: EnumSet<Foo> = EnumSet::new();
    let e2: EnumSet<Foo> = EnumSet::new();
    assert!(e1.is_disjoint(&e2));
}

#[test]
fn test_empty_does_not_intersect_with_full() {
    let e1: EnumSet<Foo> = EnumSet::new();

    let mut e2: EnumSet<Foo> = EnumSet::new();
    e2.insert(A);
    e2.insert(B);
    e2.insert(C);

    assert!(e1.is_disjoint(&e2));
}

#[test]
fn test_disjoint_intersects() {
    let mut e1: EnumSet<Foo> = EnumSet::new();
    e1.insert(A);

    let mut e2: EnumSet<Foo> = EnumSet::new();
    e2.insert(B);

    assert!(e1.is_disjoint(&e2));
}

#[test]
fn test_overlapping_intersects() {
    let mut e1: EnumSet<Foo> = EnumSet::new();
    e1.insert(A);

    let mut e2: EnumSet<Foo> = EnumSet::new();
    e2.insert(A);
    e2.insert(B);

    assert!(!e1.is_disjoint(&e2));
}

///////////////////////////////////////////////////////////////////////////
// contains and contains_elem

#[test]
fn test_superset() {
    let mut e1: EnumSet<Foo> = EnumSet::new();
    e1.insert(A);

    let mut e2: EnumSet<Foo> = EnumSet::new();
    e2.insert(A);
    e2.insert(B);

    let mut e3: EnumSet<Foo> = EnumSet::new();
    e3.insert(C);

    assert!(e1.is_subset(&e2));
    assert!(e2.is_superset(&e1));
    assert!(!e3.is_superset(&e2));
    assert!(!e2.is_superset(&e3));
}

#[test]
fn test_contains() {
    let mut e1: EnumSet<Foo> = EnumSet::new();
    e1.insert(A);
    assert!(e1.contains(&A));
    assert!(!e1.contains(&B));
    assert!(!e1.contains(&C));

    e1.insert(A);
    e1.insert(B);
    assert!(e1.contains(&A));
    assert!(e1.contains(&B));
    assert!(!e1.contains(&C));
}

///////////////////////////////////////////////////////////////////////////
// iter

#[test]
fn test_iterator() {
    let mut e1: EnumSet<Foo> = EnumSet::new();

    let elems: Vec<Foo> = e1.iter().collect();
    assert!(elems.is_empty());

    e1.insert(A);
    let elems: Vec<_> = e1.iter().collect();
    assert_eq!(vec![A], elems);

    e1.insert(C);
    let elems: Vec<_> = e1.iter().collect();
    assert_eq!(vec![A,C], elems);

    e1.insert(C);
    let elems: Vec<_> = e1.iter().collect();
    assert_eq!(vec![A,C], elems);

    e1.insert(B);
    let elems: Vec<_> = e1.iter().collect();
    assert_eq!(vec![A,B,C], elems);
}

#[test]
fn test_clone_iterator() {
    let mut e: EnumSet<Foo> = EnumSet::new();
    e.insert(A);
    e.insert(B);
    e.insert(C);

    let mut iter1 = e.iter();
    let first_elem = iter1.next();
    assert_eq!(Some(A), first_elem);

    let iter2 = iter1.clone();
    let elems1: Vec<_> = iter1.collect();
    assert_eq!(vec![B, C], elems1);

    let elems2: Vec<_> = iter2.collect();
    assert_eq!(vec![B, C], elems2);
}

///////////////////////////////////////////////////////////////////////////
// operators

#[test]
fn test_operators() {
    let mut e1: EnumSet<Foo> = EnumSet::new();
    e1.insert(A);
    e1.insert(C);

    let mut e2: EnumSet<Foo> = EnumSet::new();
    e2.insert(B);
    e2.insert(C);

    let e_union = e1 | e2;
    let elems: Vec<_> = e_union.iter().collect();
    assert_eq!(vec![A,B,C], elems);

    let e_intersection = e1 & e2;
    let elems: Vec<_> = e_intersection.iter().collect();
    assert_eq!(vec![C], elems);

    // Another way to express intersection
    let e_intersection = e1 - (e1 - e2);
    let elems: Vec<_> = e_intersection.iter().collect();
    assert_eq!(vec![C], elems);

    let e_subtract = e1 - e2;
    let elems: Vec<_> = e_subtract.iter().collect();
    assert_eq!(vec![A], elems);

    // Bitwise XOR of two sets, aka symmetric difference
    let e_symmetric_diff = e1 ^ e2;
    let elems: Vec<_> = e_symmetric_diff.iter().collect();
    assert_eq!(vec![A,B], elems);

    // Another way to express symmetric difference
    let e_symmetric_diff = (e1 - e2) | (e2 - e1);
    let elems: Vec<_> = e_symmetric_diff.iter().collect();
    assert_eq!(vec![A,B], elems);

    // Yet another way to express symmetric difference
    let e_symmetric_diff = (e1 | e2) - (e1 & e2);
    let elems: Vec<_> = e_symmetric_diff.iter().collect();
    assert_eq!(vec![A,B], elems);
}

#[test]
#[should_panic]
fn test_overflow() {
    #[allow(dead_code)]
    #[repr(u32)]
    #[derive(Clone, Copy)]
    enum Bar {
        V00, V01, V02, V03, V04, V05, V06, V07, V08, V09,
        V10, V11, V12, V13, V14, V15, V16, V17, V18, V19,
        V20, V21, V22, V23, V24, V25, V26, V27, V28, V29,
        V30, V31, V32, V33, V34, V35, V36, V37, V38, V39,
    }

    impl CLike for Bar {
        fn to_u32(&self) -> u32 {
            *self as u32
        }

        unsafe fn from_u32(v: u32) -> Bar {
            mem::transmute(v)
        }
    }

    let mut set = EnumSet::new();
    set.insert(Bar::V32);
}
