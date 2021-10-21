// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{AsBytes, BaseElement, DeserializationError, FieldElement, Serializable, StarkField};
use crate::field::{CubeExtension, QuadExtension};
use core::convert::TryFrom;
use num_bigint::BigUint;
use proptest::prelude::*;
use rand_utils::rand_value;

// MANUAL TESTS
// ================================================================================================

#[test]
fn add() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(r, r + BaseElement::ZERO);

    // test addition within bounds
    assert_eq!(
        BaseElement::from(5u8),
        BaseElement::from(2u8) + BaseElement::from(3u8)
    );

    // test overflow
    let t = BaseElement::from(BaseElement::MODULUS - 1);
    assert_eq!(BaseElement::ZERO, t + BaseElement::ONE);
    assert_eq!(BaseElement::ONE, t + BaseElement::from(2u8));
}

#[test]
fn sub() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(r, r - BaseElement::ZERO);

    // test subtraction within bounds
    assert_eq!(
        BaseElement::from(2u8),
        BaseElement::from(5u8) - BaseElement::from(3u8)
    );

    // test underflow
    let expected = BaseElement::from(BaseElement::MODULUS - 2);
    assert_eq!(expected, BaseElement::from(3u8) - BaseElement::from(5u8));
}

#[test]
fn neg() {
    assert_eq!(BaseElement::ZERO, -BaseElement::ZERO);
    assert_eq!(BaseElement::from(super::M - 1), -BaseElement::ONE);

    let r: BaseElement = rand_value();
    assert_eq!(r, -(-r));
}

#[test]
fn mul() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(BaseElement::ZERO, r * BaseElement::ZERO);
    assert_eq!(r, r * BaseElement::ONE);

    // test multiplication within bounds
    assert_eq!(
        BaseElement::from(15u8),
        BaseElement::from(5u8) * BaseElement::from(3u8)
    );

    // test overflow
    let m = BaseElement::MODULUS;
    let t = BaseElement::from(m - 1);
    assert_eq!(BaseElement::ONE, t * t);
    assert_eq!(BaseElement::from(m - 2), t * BaseElement::from(2u8));
    assert_eq!(BaseElement::from(m - 4), t * BaseElement::from(4u8));

    let t = (m + 1) / 2;
    assert_eq!(
        BaseElement::ONE,
        BaseElement::from(t) * BaseElement::from(2u8)
    );
}

#[test]
fn exp() {
    let a = BaseElement::ZERO;
    assert_eq!(a.exp(0), BaseElement::ONE);
    assert_eq!(a.exp(1), BaseElement::ZERO);

    let a = BaseElement::ONE;
    assert_eq!(a.exp(0), BaseElement::ONE);
    assert_eq!(a.exp(1), BaseElement::ONE);
    assert_eq!(a.exp(3), BaseElement::ONE);

    let a: BaseElement = rand_value();
    assert_eq!(a.exp(3), a * a * a);
}

#[test]
fn inv() {
    // identity
    assert_eq!(BaseElement::ONE, BaseElement::inv(BaseElement::ONE));
    assert_eq!(BaseElement::ZERO, BaseElement::inv(BaseElement::ZERO));
}

#[test]
fn element_as_int() {
    let v = u64::MAX;
    let e = BaseElement::new(v);
    assert_eq!(v % super::M, e.as_int());
}

#[test]
fn equals() {
    let a = BaseElement::ONE;
    let b = BaseElement::new(super::M - 1) * BaseElement::new(super::M - 1);

    // elements are equal
    assert_eq!(a, b);
    assert_eq!(a.as_int(), b.as_int());
    assert_eq!(a.to_bytes(), b.to_bytes());

    // but their internal representation is not
    assert_ne!(a.0, b.0);
    assert_ne!(a.as_bytes(), b.as_bytes());
}

// ROOTS OF UNITY
// ------------------------------------------------------------------------------------------------

#[test]
fn get_root_of_unity() {
    let root_32 = BaseElement::get_root_of_unity(32);
    assert_eq!(BaseElement::TWO_ADIC_ROOT_OF_UNITY, root_32);
    assert_eq!(BaseElement::ONE, root_32.exp(1u64 << 32));

    let root_31 = BaseElement::get_root_of_unity(31);
    let expected = root_32.exp(2);
    assert_eq!(expected, root_31);
    assert_eq!(BaseElement::ONE, root_31.exp(1u64 << 31));
}

// SERIALIZATION AND DESERIALIZATION
// ------------------------------------------------------------------------------------------------

#[test]
fn from_u128() {
    let v = u128::MAX;
    let e = BaseElement::from(v);
    assert_eq!((v % super::M as u128) as u64, e.as_int());
}

#[test]
fn try_from_slice() {
    let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_ok());
    assert_eq!(1, result.unwrap().as_int());

    let bytes = vec![1, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());

    let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());

    let bytes = vec![255, 255, 255, 255, 255, 255, 255, 255];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());
}

#[test]
fn elements_as_bytes() {
    let source = vec![
        BaseElement::new(1),
        BaseElement::new(2),
        BaseElement::new(3),
        BaseElement::new(4),
    ];

    let mut expected = vec![];
    expected.extend_from_slice(&source[0].0.to_le_bytes());
    expected.extend_from_slice(&source[1].0.to_le_bytes());
    expected.extend_from_slice(&source[2].0.to_le_bytes());
    expected.extend_from_slice(&source[3].0.to_le_bytes());

    assert_eq!(expected, BaseElement::elements_as_bytes(&source));
}

#[test]
fn bytes_as_elements() {
    let elements = vec![
        BaseElement::new(1),
        BaseElement::new(2),
        BaseElement::new(3),
        BaseElement::new(4),
    ];

    let mut bytes = vec![];
    bytes.extend_from_slice(&elements[0].0.to_le_bytes());
    bytes.extend_from_slice(&elements[1].0.to_le_bytes());
    bytes.extend_from_slice(&elements[2].0.to_le_bytes());
    bytes.extend_from_slice(&elements[3].0.to_le_bytes());
    bytes.extend_from_slice(&BaseElement::new(5).0.to_le_bytes());

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[..32]) };
    assert!(result.is_ok());
    assert_eq!(elements, result.unwrap());

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[..33]) };
    assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[1..33]) };
    assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));
}

// INITIALIZATION
// ------------------------------------------------------------------------------------------------

#[test]
fn zeroed_vector() {
    let result = BaseElement::zeroed_vector(4);
    assert_eq!(4, result.len());
    for element in result.into_iter() {
        assert_eq!(BaseElement::ZERO, element);
    }
}

// QUADRATIC EXTENSION
// ------------------------------------------------------------------------------------------------
#[test]
fn quad_mul() {
    // identity
    let r: QuadExtension<BaseElement> = rand_value();
    assert_eq!(
        <QuadExtension<BaseElement>>::ZERO,
        r * <QuadExtension<BaseElement>>::ZERO
    );
    assert_eq!(r, r * <QuadExtension<BaseElement>>::ONE);

    // test multiplication within bounds
    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(3), BaseElement::ONE);
    let b = <QuadExtension<BaseElement>>::new(BaseElement::new(4), BaseElement::new(2));
    let expected = <QuadExtension<BaseElement>>::new(BaseElement::new(8), BaseElement::new(12));
    assert_eq!(expected, a * b);

    // test multiplication with overflow
    let m = BaseElement::MODULUS;
    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(3), BaseElement::new(m - 1));
    let b = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 3), BaseElement::new(5));
    let expected = <QuadExtension<BaseElement>>::new(BaseElement::ONE, BaseElement::new(13));
    assert_eq!(expected, a * b);

    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(3), BaseElement::new(m - 1));
    let b = <QuadExtension<BaseElement>>::new(BaseElement::new(10), BaseElement::new(m - 2));
    let expected = <QuadExtension<BaseElement>>::new(
        BaseElement::new(26),
        BaseElement::new(18446744069414584307),
    );
    assert_eq!(expected, a * b);
}

#[test]
fn quad_conjugate() {
    let m = BaseElement::MODULUS;

    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 1), BaseElement::new(3));
    let expected = <QuadExtension<BaseElement>>::new(
        BaseElement::new(2),
        BaseElement::new(18446744069414584318),
    );
    assert_eq!(expected, a.conjugate());

    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 3), BaseElement::new(m - 2));
    let expected = <QuadExtension<BaseElement>>::new(
        BaseElement::new(18446744069414584316),
        BaseElement::new(2),
    );
    assert_eq!(expected, a.conjugate());

    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(4), BaseElement::new(7));
    let expected = <QuadExtension<BaseElement>>::new(
        BaseElement::new(11),
        BaseElement::new(18446744069414584314),
    );
    assert_eq!(expected, a.conjugate());
}

// CUBIC EXTENSION
// ------------------------------------------------------------------------------------------------
#[test]
fn cube_mul() {
    // identity
    let r: CubeExtension<BaseElement> = rand_value();
    assert_eq!(
        <CubeExtension<BaseElement>>::ZERO,
        r * <CubeExtension<BaseElement>>::ZERO
    );
    assert_eq!(r, r * <CubeExtension<BaseElement>>::ONE);

    // test multiplication within bounds
    let a = <CubeExtension<BaseElement>>::new(
        BaseElement::new(3),
        BaseElement::new(5),
        BaseElement::new(2),
    );
    let b = <CubeExtension<BaseElement>>::new(
        BaseElement::new(320),
        BaseElement::new(68),
        BaseElement::new(3),
    );
    let expected = <CubeExtension<BaseElement>>::new(
        BaseElement::new(1111),
        BaseElement::new(1961),
        BaseElement::new(995),
    );
    assert_eq!(expected, a * b);

    // test multiplication with overflow
    let a = <CubeExtension<BaseElement>>::new(
        BaseElement::new(18446744069414584267),
        BaseElement::new(18446744069414584309),
        BaseElement::new(9223372034707292160),
    );
    let b = <CubeExtension<BaseElement>>::new(
        BaseElement::new(18446744069414584101),
        BaseElement::new(420),
        BaseElement::new(18446744069414584121),
    );
    let expected = <CubeExtension<BaseElement>>::new(
        BaseElement::new(14070),
        BaseElement::new(18446744069414566571),
        BaseElement::new(5970),
    );
    assert_eq!(expected, a * b);

    let a = <CubeExtension<BaseElement>>::new(
        BaseElement::new(18446744069414584266),
        BaseElement::new(18446744069412558094),
        BaseElement::new(5268562),
    );
    let b = <CubeExtension<BaseElement>>::new(
        BaseElement::new(18446744069414583589),
        BaseElement::new(1226),
        BaseElement::new(5346),
    );
    let expected = <CubeExtension<BaseElement>>::new(
        BaseElement::new(18446744065041672051),
        BaseElement::new(25275910656),
        BaseElement::new(21824696736),
    );
    assert_eq!(expected, a * b);
}

// RANDOMIZED TESTS
// ================================================================================================

proptest! {

    #[test]
    fn add_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 + v2;

        let expected = (((a as u128) + (b as u128)) % (super::M as u128)) as u64;
        prop_assert_eq!(expected, result.as_int());
    }

    #[test]
    fn sub_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 - v2;

        let a = a % super::M;
        let b = b % super::M;
        let expected = if a < b { super::M - b + a } else { a - b };

        prop_assert_eq!(expected, result.as_int());
    }

    #[test]
    fn neg_proptest(a in any::<u64>()) {
        let v = BaseElement::from(a);
        let expected = super::M - (a % super::M);

        prop_assert_eq!(expected, (-v).as_int());
    }

    #[test]
    fn mul_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 * v2;

        let expected = (((a as u128) * (b as u128)) % super::M as u128) as u64;
        prop_assert_eq!(expected, result.as_int());
    }

    #[test]
    fn double_proptest(x in any::<u64>()) {
        let v = BaseElement::from(x);
        let result = v.double();

        let expected = (((x as u128) * 2) % super::M as u128) as u64;
        prop_assert_eq!(expected, result.as_int());
    }

    #[test]
    fn exp_proptest(a in any::<u64>(), b in any::<u64>()) {
        let result = BaseElement::from(a).exp(b);

        let b = BigUint::from(b);
        let m = BigUint::from(super::M);
        let expected = BigUint::from(a).modpow(&b, &m).to_u64_digits()[0];
        prop_assert_eq!(expected, result.as_int());
    }

    #[test]
    fn inv_proptest(a in any::<u64>()) {
        let a = BaseElement::from(a);
        let b = a.inv();

        let expected = if a == BaseElement::ZERO { BaseElement::ZERO } else { BaseElement::ONE };
        prop_assert_eq!(expected, a * b);
    }

    #[test]
    fn element_as_int_proptest(a in any::<u64>()) {
        let e = BaseElement::new(a);
        prop_assert_eq!(a % super::M, e.as_int());
    }

    #[test]
    fn from_u128_proptest(v in any::<u128>()) {
        let e = BaseElement::from(v);
        assert_eq!((v % super::M as u128) as u64, e.as_int());
    }

    // QUADRATIC EXTENSION
    // --------------------------------------------------------------------------------------------
    #[test]
    fn quad_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>()) {
        let a = QuadExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1));
        let b = a.inv();

        let expected = if a == QuadExtension::<BaseElement>::ZERO {
            QuadExtension::<BaseElement>::ZERO
        } else {
            QuadExtension::<BaseElement>::ONE
        };
        prop_assert_eq!(expected, a * b);
    }

    // CUBIC EXTENSION
    // --------------------------------------------------------------------------------------------
    #[test]
    fn cube_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>(), a2 in any::<u64>()) {
        let a = CubeExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1), BaseElement::from(a2));
        let b = a.inv();

        let expected = if a == CubeExtension::<BaseElement>::ZERO {
            CubeExtension::<BaseElement>::ZERO
        } else {
            CubeExtension::<BaseElement>::ONE
        };
        prop_assert_eq!(expected, a * b);
    }
}
