//! Helper functions for common quantum Gates

use crate::common::types::{Gate, QubitRef};
use num_complex::Complex64;
use std::f64::consts::PI;

macro_rules! c {
    ($re:expr, $im:expr) => {
        Complex64::new($re, $im);
    };
    ($re:expr) => {
        Complex64::new($re, 0.)
    };
}

/// Returns an I gate.
pub fn i_gate(target: QubitRef) -> Gate {
    Gate::unitary(vec![target], vec![], vec![c!(1.), c!(0.), c!(0.), c!(1.)])
}

/// Returns an arbitrary X rotation gate.
/// Theta is the rotation angle in radians.
pub fn rx_gate(target: QubitRef, theta: f64) -> Gate {
    let a = c!((0.5 * theta).cos());
    let b = c!(0., -1.) * (0.5 * theta).sin();
    Gate::unitary(vec![target], vec![], vec![a, b, b, a])
}

/// Returns an arbitrary Y rotation gate.
/// Theta is the rotation angle in radians.
pub fn ry_gate(target: QubitRef, theta: f64) -> Gate {
    let a = c!((0.5 * theta).cos());
    let b = c!((0.5 * theta).sin());
    Gate::unitary(vec![target], vec![], vec![a, -b, b, a])
}

/// Returns an arbitrary Y rotation gate.
/// Theta is the rotation angle in radians.
pub fn rz_gate(target: QubitRef, theta: f64) -> Gate {
    let a = c!(0., -0.5 * theta).exp();
    let b = c!(0., 0.5 * theta).exp();
    Gate::unitary(vec![target], vec![], vec![a, c!(0.), c!(0.), b])
}

// TODO(mb): r_gate

/// Returns a swap gate on provided target qubits a and b.
pub fn swap_gate(a: QubitRef, b: QubitRef) -> Gate {
    Gate::unitary(
        vec![a, b],
        vec![],
        vec![
            c!(1.),
            c!(0.),
            c!(0.),
            c!(0.),
            //
            c!(0.),
            c!(0.),
            c!(1.),
            c!(0.),
            //
            c!(0.),
            c!(1.),
            c!(0.),
            c!(0.),
            //
            c!(0.),
            c!(0.),
            c!(0.),
            c!(1.),
        ],
    )
}

/// Returns a square-root-of-swap gate on provided target qubits a and b.
pub fn sqswap_gate(a: QubitRef, b: QubitRef) -> Gate {
    Gate::unitary(
        vec![a, b],
        vec![],
        vec![
            c!(1.),
            c!(0.),
            c!(0.),
            c!(0.),
            //
            c!(0.),
            c!(0.5, 0.5),
            c!(0.5, -0.5),
            c!(0.),
            //
            c!(0.),
            c!(0.5, -0.5),
            c!(0.5, 0.5),
            c!(0.),
            //
            c!(0.),
            c!(0.),
            c!(0.),
            c!(1.),
        ],
    )
}

/// Returns an X gate.
pub fn x_gate(target: QubitRef) -> Gate {
    rx_gate(target, PI)
}

/// Returns a 90-degree X gate.
pub fn x90_gate(target: QubitRef) -> Gate {
    rx_gate(target, 0.5 * PI)
}

/// Returns a negative 90-degree X gate.
pub fn mx90_gate(target: QubitRef) -> Gate {
    rx_gate(target, -0.5 * PI)
}

/// Returns a Y gate.
pub fn y_gate(target: QubitRef) -> Gate {
    ry_gate(target, PI)
}

/// Returns a 90-degree Y gate.
pub fn y90_gate(target: QubitRef) -> Gate {
    ry_gate(target, 0.5 * PI)
}

/// Returns a negative 90-degree Y gate.
pub fn my90_gate(target: QubitRef) -> Gate {
    ry_gate(target, -0.5 * PI)
}

/// Returns a Z gate.
pub fn z_gate(target: QubitRef) -> Gate {
    rz_gate(target, PI)
}

/// Returns a 90-degree Z gate.
pub fn z90_gate(target: QubitRef) -> Gate {
    rz_gate(target, 0.5 * PI)
}

/// Returns a negative 90-degree Z gate.
pub fn mz90_gate(target: QubitRef) -> Gate {
    rz_gate(target, -0.5 * PI)
}

/// Returns an S gate.
pub fn s_gate(target: QubitRef) -> Gate {
    z90_gate(target)
}

/// Returns an S-dagger gate.
pub fn sdag_gate(target: QubitRef) -> Gate {
    mz90_gate(target)
}

/// Returns a T gate.
pub fn t_gate(target: QubitRef) -> Gate {
    rz_gate(target, 0.25 * PI)
}

/// Returns a T-dagger gate.
pub fn tdag_gate(target: QubitRef) -> Gate {
    rz_gate(target, -0.25 * PI)
}

/// Returns a Hadamard gate.
pub fn h_gate(target: QubitRef) -> Gate {
    let x = c!(1. / 2f64.sqrt());
    Gate::unitary(vec![target], vec![], vec![x, x, x, -x])
}

/// Returns a CNOT gate.
pub fn cnot_gate(control: QubitRef, target: QubitRef) -> Gate {
    Gate::unitary(
        vec![target],
        vec![control],
        vec![c!(0.), c!(1.), c!(1.), c!(0.)],
    )
}

/// Returns a Toffoli gate.
pub fn toffoli_gate(c1: QubitRef, c2: QubitRef, target: QubitRef) -> Gate {
    Gate::unitary(
        vec![target],
        vec![c1, c2],
        vec![c!(0.), c!(1.), c!(1.), c!(0.)],
    )
}

/// Returns a Fredking gate.
pub fn fredkin_gate(control: QubitRef, a: QubitRef, b: QubitRef) -> Gate {
    Gate::unitary(
        vec![a, b],
        vec![control],
        vec![
            c!(1.),
            c!(0.),
            c!(0.),
            c!(0.),
            //
            c!(0.),
            c!(0.),
            c!(1.),
            c!(0.),
            //
            c!(0.),
            c!(1.),
            c!(0.),
            c!(0.),
            //
            c!(0.),
            c!(0.),
            c!(0.),
            c!(1.),
        ],
    )
}
