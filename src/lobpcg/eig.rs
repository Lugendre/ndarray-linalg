///! Implements truncated eigenvalue decomposition
///

use ndarray::prelude::*;
use ndarray::stack;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use num_traits::{Float, NumCast};
use crate::{Scalar, Lapack};
use super::lobpcg::{lobpcg, EigResult, Order};

pub struct TruncatedEig<A: Scalar> {
    order: Order,
    problem: Array2<A>,
    pub constraints: Option<Array2<A>>,
    precision: A::Real,
    maxiter: usize
}

impl<A: Scalar + Lapack + PartialOrd + Default> TruncatedEig<A> {
    pub fn new(problem: Array2<A>, order: Order) -> TruncatedEig<A> {
        TruncatedEig {
            precision: NumCast::from(1e-5).unwrap(),
            maxiter: problem.len_of(Axis(0)) * 2,
            constraints: None,
            order, 
            problem
        }
    }

    pub fn precision(mut self, precision: A::Real) -> Self {
        self.precision = precision;

        self
    }

    pub fn maxiter(mut self, maxiter: usize) -> Self {
        self.maxiter = maxiter;

        self

    }

    pub fn constraints(mut self, constraints: Array2<A>) -> Self {
        self.constraints = Some(constraints);

        self
    }

    pub fn once(&self, num: usize) -> EigResult<A> {
        let x = Array2::random((self.problem.len_of(Axis(0)), num), Uniform::new(0.0, 1.0))
            .mapv(|x| NumCast::from(x).unwrap());

        lobpcg(|y| self.problem.dot(&y), x, None, self.constraints.clone(), self.precision, self.maxiter, self.order.clone())
    }
}

impl<A: Float + Scalar + Lapack + PartialOrd + Default> IntoIterator for TruncatedEig<A> {
    type Item = (Array1<A>, Array2<A>);
    type IntoIter = TruncatedEigIterator<A>;

    fn into_iter(self) -> TruncatedEigIterator<A>{
        TruncatedEigIterator {
            step_size: 1,
            eig: self
        }
    }
}

pub struct TruncatedEigIterator<A: Scalar> {
    step_size: usize,
    eig: TruncatedEig<A>
}

impl<A: Float + Scalar + Lapack + PartialOrd + Default> Iterator for TruncatedEigIterator<A> {
    type Item = (Array1<A>, Array2<A>);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.eig.once(self.step_size);
        dbg!(&res);

        match res {
            EigResult::Ok(vals, vecs, norms) | EigResult::Err(vals, vecs, norms, _) => {
                // abort if any eigenproblem did not converge
                for r_norm in norms {
                    if r_norm > NumCast::from(0.1).unwrap() {
                        return None;
                    }
                }

                let new_constraints = if let Some(ref constraints) = self.eig.constraints {
                    let eigvecs_arr = constraints.gencolumns().into_iter()
                        .chain(vecs.gencolumns().into_iter())
                        .map(|x| x.insert_axis(Axis(1)))
                        .collect::<Vec<_>>();

                    stack(Axis(1), &eigvecs_arr).unwrap()
                } else {
                    vecs.clone()
                };

                dbg!(&new_constraints);

                self.eig.constraints = Some(new_constraints);

                Some((vals, vecs))
            },
            EigResult::NoResult(_) => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TruncatedEig;
    use super::Order;
    use ndarray::Array2;
    use ndarray_rand::rand_distr::Uniform;
    use ndarray_rand::RandomExt;

    #[test]
    fn test_truncated_eig() {
        let a = Array2::random((50, 50), Uniform::new(0., 1.0));
        let a = a.t().dot(&a);

        let teig = TruncatedEig::new(a, Order::Largest)
            .precision(1e-5)
            .maxiter(500);
        
        let res = teig.into_iter().take(3).flat_map(|x| x.0.to_vec()).collect::<Vec<_>>();
        dbg!(&res);
        panic!("");
    }
}
