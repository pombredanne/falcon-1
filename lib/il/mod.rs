//! Falcon Intermediate Language.
//!
//! # An Introduction
//!
//! Falcon IL is a simple, expression-based, well-defined, semantically-accurate
//! intermediate language for the analysis of Binary Programs.
//!
//! * **Simple** - Falcon IL has 21 expression types (including terminals), and 5
//! operation types, minimizing the work required to implement analyses.
//! * **Expression-based** - Falcon IL operates over expression, as opposed to a
//! [three-address form](https://en.wikipedia.org/wiki/Three-address_code) like
//! REIL/RREIL.
//! * **Well-defined** - Falcon IL is specified with rust's enumerated types, leaving
//! no ambiguity in the IL.
//! * **Semantically accurate** - Falcon IL accurately captures the semantics of
//! underlying architectures. This is mainly a function of the lifters.  A
//! divergence in the lifted semantics and the target architecture is a bug.
//! This makes Falcon IL suitable for analyses which require precision in the
//! semantics.
//!
//! ## Limitations
//!
//! * Falcon IL does not support operations over values > 64 bits in width.
//! * Falcon IL does not support floating point operations.
//!
//! While Falcon IL allows for analyses that find real bugs, due to these
//! limitations it cannot completely analyze programs which require floating
//! point or wide-register instructions.
//! 
//! ## Position and Semantics
//! 
//! You should think of components of the Falcon IL as belonging to two groups:
//!
//! * Components which provide program semantics
//! * Components which provide location within a binary.
//! 
//! The following components provide semantics:
//!
//! * `Array`
//! * `Block`
//! * `Constant `
//! * `ControlFlowGraph`
//! * `Edge`
//! * `Expression`
//! * `Operation`
//! * `Scalar`
//! * `Variable`
//! 
//! The following components provide location within a binary:
//!
//! * `Function`
//! * `Instruction`
//! * `ProgramLocation` / `FunctionLocation`
//! * `RefProgramLocation` / `RefFunctionLocation`
//! * `Program`
//!
//! While this construct may seem verbose at first, in practice it is not. There
//! are several convenience functions which allow for quickly gathering the
//! necessary information from different components of the IL, and the
//! IL is easily iterable.
//!
//! # Components of the IL
//! 
//! ## `Array`, `Constant`, `Scalar`, and `Variable`.
//! 
//! The terminals in Falcon IL are `Array`, `Constant` and `Scalar`.
//!
//! `Variable` is a trait implemented by `Array` and `Scalar`, and provides
//! convenience functions applicable to both types. You most
//! likely will not encounter it in your analyses.
//! 
//! ## `Expression`
//! 
//! Expressions implement basic arithemtic operations, comparison
//! operations, and bit extension/truncation operations over the terminals
//! `Scalar` and `Constant`
//!
//! * Terminals: `Scalar`, `Constant`.
//! * Arithmetic: `Add`, `Sub`, `Mul`, `Divu`, `Modu`, `Divs`, `Mods`, `And`,
//! `Or`, `Xor`, `Shl`, `Shr`.
//! * Comparison: `Cmpeq`, `Cmpneq`, `Cmplts`, `Cmpltu`.
//! * Extension: `Zext`, `Sext`, `Trun`.
//!
//! Comparison expressions evaluate to a 1-bit expression with the value `1`
//! representing `True`, and the value `0` representing `False`.
//!
//! It is an error to create an expresison which operates over expressions of
//! differing bitness. This is checked dynamically at runtime, and a `Sort`
//! error wil be emitted if expressions have operands of differing bitness. It
//! is a bug if a lifter generates an expression with operands of differing
//! bitness. `Zext`, `Sext`, and `Trun` should be used to ensure expressions
//! are of the same bitness.
//! 
//! ## `Operation`
//! 
//! An `Operation` applies a transformation over some state. There are five
//! types of `Operation` in Falcon:
//!
//! * `Assign`: Assigns an `Expression` to a `Scalar`.
//! * `Store`: Stores an `Expression` into an `Array`,
//! indexed by an `Expression`. The size of the store will be determined by the
//! size of the expression being stored.
//! * `Load`: Loads an `Expression` from an `Array`, indexed by an `Expression`,
//! and places the result into a `Scalar`. The size of the load will be
//! determined by the size of the Scalar being loaded into.
//! * `Brc`: Short for, "Branch Conditional." When the conditional expression,
//! which is a 1-bit value, is equal to `1`, the program should branch to the
//! given target address.
//! * `Raise`: The raise operation takes a single `Expression`, which is
//! architecture/lifter-dependent, and allows for implementation of semantics
//! which cannot be captured by Falcon IL, for example a system call.
//!
//! When lifting, direct conditional branches such as X86 `je` or MIPS `be` do
//! not result in an `Operation::Brc`. Instead, the instruction will be omitted
//! and edges will be emitted in the `ControlFlowGraph` with expressions which
//! guard those edges. `Brc` will only be emitted for indirect branches.
//!
//! `Load` and `Store` are obviously dependent on the endianness of the target
//! architecture. Endianness is specified in the memory model, not in the IL.
//!
//! ## `Instruction`
//! 
//! An instruction provides position to an `Operation` within a `Block`, and
//! also carries an optional `address`. When an `Instruction` is lifted from a
//! program, this `address` field will be filled in.
//!
//! You should not create an `Instruction` explicitly, but instead call the
//! various methods over `Block` corresponding to the `Operation` you wish to
//! emit, and this will create the `Instruction` implicitly.
//!
//! ## `Block`
//!
//! A `Block` is a basic block, or a sequence of `Instruction`.
//!
//! A `Block` does not carry an `address` field like `Instruction`. Instead, a
//! `Block` has an `index` field, which is an arbitray location within a
//! `ControlFlowGraph`.
//!
//! ## `Edge`
//! 
//! An `Edge` connects two `Block`s in a `ControlFlowGraph`. An edge has an
//! optional field `condition`, which guards the edge. When direct conditional
//! branches are lifted from a program, this field will be filled in with the
//! condition that guards traversal of the edge.
//!
//! We create a new `Edge` by calling the `conditional_edge` and
//! `unconditional_edge` methods on a `ControlFlowGraph`.
//!
//! ## `ControlFlowGraph`
//!
//! A `ControlFlowGraph` is a directed graph with vertices of type `Block` and
//! edges of type `Edge`.
//!
//! A perhaps interesting property of `ControlFlowGraph` is the
//! optional `entry` and `exit` index. Falcon lifters lift individual
//! instructions to `ControlFlowGraph`, allowing them to capture semantics of
//! instructions which loop, such as X86's `repne scasb` instruction.
//! Since all instructions have clearly defined entry and exit points, we can
//! use these to append their lifted graphs together. This is how Falcon's
//! lifters construct basic blocks.
//!
//! ## `Function`
//! 
//! A function holds an address, and a `ControlFlowGraph`, applying location to
//! the `ControlFlowGraph`. A function also has an optional `name`, which will
//! be filled in by a `Loader` when a corresponding symbol is available, as well
//! as an optional `index` for when this function belongs to an `il::Program`.
//!
//! ## `Program`
//!
//! A program holds multiple instances of `Function`.
//!
//! # That's it!
//!
//! Falcon IL may seem verbose, because of the many components, but in practice
//! it is relatively simple and straight forward, with a minimal set of
//! expressions and operations. Influenced by RREIL and Binary Ninja's IL,
//! Falcon IL is my 3rd IL for binary program analysis, and strikes a good
//! balance between simplicity and readability.
//!
//! Unless writing a lifter, you should never have to create elements of the IL
//! yourself, and can use the accessor methods to gather the information
//! required.

use error::*;
use graph;

mod array;
mod block;
mod constant;
mod control_flow_graph;
mod edge;
mod expression;
mod function;
mod instruction;
mod location;
mod operation;
mod scalar;
mod program;
mod variable;

pub use self::array::*;
pub use self::block::*;
pub use self::constant::*;
pub use self::control_flow_graph::*;
pub use self::edge::*;
pub use self::expression::*;
pub use self::function::*;
pub use self::instruction::*;
pub use self::location::*;
pub use self::operation::*;
pub use self::scalar::*;
pub use self::program::*;
pub use self::variable::*;

/// A convenience function to create a new constant.
///
/// This is the preferred way to create a `Constant`.
pub fn const_(value: u64, bits: usize) -> Constant {
    Constant::new(value, bits)
}


/// A convenience function to create a new constant expression.
///
/// This is the preferred way to create an `Expression::Constant`.
pub fn expr_const(value: u64, bits: usize) -> Expression {
    Expression::constant(Constant::new(value, bits))
}


/// A convenience function to create a new scalar.
///
/// This is the preferred way to create a `Scalar`.
pub fn scalar<S>(name: S, bits: usize) -> Scalar where S: Into<String> {
    Scalar::new(name, bits)
}


/// A convenience function to create a new scalar expression.
///
/// This is the preferred way to create an `Expression::Scalar`.
pub fn expr_scalar<S>(name: S, bits: usize) -> Expression where S: Into<String> {
    Expression::scalar(Scalar::new(name, bits))
}


/// A convenience function to create a new array
///
/// This is the preferred way to create an `Array`.
pub fn array<S>(name: S, size: u64) -> Array where S: Into<String> {
    Array::new(name, size)
}