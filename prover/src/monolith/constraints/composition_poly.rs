// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::StarkDomain;
use math::{
    fft,
    field::{FieldElement, StarkField},
    polynom,
};
use std::marker::PhantomData;
use utils::{iter, uninit_vector};

#[cfg(feature = "concurrent")]
use rayon::prelude::*;

// COMPOSITION POLYNOMIAL
// ================================================================================================
/// Represents a composition polynomial split into columns with each column being of length equal
/// to trace_length. Thus, for example, if the composition polynomial has degree 2N - 1, where N
/// is the trace length, it will be stored as two columns of size N (each of degree N - 1).
pub struct CompositionPoly<B: StarkField, E: FieldElement<BaseField = B>> {
    columns: Vec<Vec<E>>,
    _base_field: PhantomData<B>,
}

impl<B: StarkField, E: FieldElement<BaseField = B>> CompositionPoly<B, E> {
    /// Returns a new composition polynomial.
    pub fn new(coefficients: Vec<E>, trace_length: usize) -> Self {
        assert!(
            coefficients.len().is_power_of_two(),
            "size of composition polynomial must be a power of 2, but was {}",
            coefficients.len(),
        );
        assert!(
            trace_length.is_power_of_two(),
            "trace length must be a power of 2, but was {}",
            trace_length
        );
        assert!(
            trace_length < coefficients.len(),
            "trace length must be smaller than size of composition polynomial"
        );
        assert!(
            coefficients[coefficients.len() - 1] != E::ZERO,
            "expected composition polynomial of degree {}, but was {}",
            coefficients.len() - 1,
            polynom::degree_of(&coefficients)
        );

        let num_columns = coefficients.len() / trace_length;
        let polys = transpose(coefficients, num_columns);

        CompositionPoly {
            columns: polys,
            _base_field: PhantomData,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of individual column polynomials used to describe this composition
    /// polynomial.
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Returns the length of individual column polynomials; this is guaranteed to be a power of 2.
    pub fn column_len(&self) -> usize {
        self.columns[0].len()
    }

    /// Returns the degree of individual column polynomial.
    pub fn column_degree(&self) -> usize {
        self.column_len() - 1
    }

    // LOW-DEGREE EXTENSION
    // --------------------------------------------------------------------------------------------
    /// Evaluates the columns of the composition polynomial over the specified LDE domain and
    /// returns the result.
    pub fn evaluate(&self, domain: &StarkDomain<B>) -> Vec<Vec<E>>
    where
        B: StarkField,
        E: From<B>,
    {
        assert_eq!(
            self.column_len(),
            domain.trace_length(),
            "inconsistent trace domain size; expected {}, but received {}",
            self.column_len(),
            domain.trace_length()
        );

        iter!(self.columns)
            .map(|poly| {
                fft::evaluate_poly_with_offset(
                    poly,
                    domain.trace_twiddles(),
                    domain.offset(),
                    domain.trace_to_lde_blowup(),
                )
            })
            .collect()
    }

    /// Returns evaluations of all composition polynomial columns at point z^m, where m is
    /// the number of column polynomials.
    pub fn evaluate_at(&self, z: E) -> Vec<E> {
        let z_m = z.exp((self.columns.len() as u32).into());
        iter!(self.columns)
            .map(|poly| polynom::eval(poly, z_m))
            .collect()
    }

    /// Transforms this composition polynomial into a vector of individual column polynomials.
    pub fn into_columns(self) -> Vec<Vec<E>> {
        self.columns
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Splits polynomial coefficients into the specified number of columns. The coefficients are split
/// in such a way that each resulting column has the same degree. For example, a polynomial
/// a * x^3 + b * x^2 + c * x + d, can be rewritten as: (b * x^2 + d) + x * (a * x^2 + c), and then
/// the two columns will be: (b * x^2 + d) and (a * x^2 + c).
fn transpose<E: FieldElement>(coefficients: Vec<E>, num_columns: usize) -> Vec<Vec<E>> {
    let column_len = coefficients.len() / num_columns;

    let mut result = (0..num_columns)
        .map(|_| uninit_vector(column_len))
        .collect::<Vec<_>>();

    // TODO: implement multi-threaded version
    for (i, coeff) in coefficients.into_iter().enumerate() {
        let row_idx = i / num_columns;
        let col_idx = i % num_columns;
        result[col_idx][row_idx] = coeff;
    }

    result
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {

    use math::field::f128::BaseElement;

    #[test]
    fn transpose() {
        let values = (0u128..16).map(BaseElement::new).collect::<Vec<_>>();
        let actual = super::transpose(values, 4);

        #[rustfmt::skip]
        let expected = vec![
            vec![BaseElement::new(0), BaseElement::new(4), BaseElement::new(8), BaseElement::new(12)],
            vec![BaseElement::new(1), BaseElement::new(5), BaseElement::new(9), BaseElement::new(13)],
            vec![BaseElement::new(2), BaseElement::new(6), BaseElement::new(10), BaseElement::new(14)],
            vec![BaseElement::new(3), BaseElement::new(7), BaseElement::new(11), BaseElement::new(15)],
        ];

        assert_eq!(expected, actual)
    }
}
