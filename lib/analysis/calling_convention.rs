//! Information about varying calling conventions.

use il;
use types::PartialBoolean;
use std::collections::HashSet;


/// Available type of calling conventions
pub enum CallingConventionType {
    MipsSystemV,
    MipselSystemV,
    Cdecl
}


/// The return type for a function.
pub enum ReturnAddressType {
    /// Functions return by loading an address from a register.
    Register(il::Scalar),
    /// Functions return by loading an address from the stack.
    ///
    /// The offset to the return address at function call/entry is given.
    Stack(usize)
}


/// The type of an argument.
pub enum ArgumentType {
    /// The argument is held in a register.
    Register(il::Scalar),

    /// The argument is held in a stack offset.
    ///
    /// The stack offset is given at function call/entry.
    Stack(usize)
}


/// Represents the calling convention of a particular platform.
pub struct CallingConvention {
    /// arguments passed in registers.
    argument_registers: Vec<il::Scalar>,
    
    /// These registers are preserved across function calls.
    preserved_registers: HashSet<il::Scalar>,

    /// These registers are not preserved across function calls.
    trashed_registers: HashSet<il::Scalar>,
    
    /// Offset from function start where first argument on stack is found.
    ///
    /// After register arguments are exhausted, analysis will begin looking
    /// here.
    stack_argument_offset: usize,

    /// Length of an argument on the stack in bytes.
    stack_argument_length: usize,

    /// The return address is given in the following type.
    return_address_type: ReturnAddressType,

    /// The register the returned value is given in.
    return_register: il::Scalar
}

/*
    Mips System V:
        $16-$23 and $29-$31 are saved. This is $s0-S8, $sp and $ra.
        Result is in $v0.
        Everything else is trashed.
*/


impl CallingConvention {
    /// Create a new `CallingConvention` based on the given
    /// `CallingConventionType`.
    pub fn new(typ: CallingConventionType) -> CallingConvention {
        match typ {
            CallingConventionType::MipsSystemV |
            CallingConventionType::MipselSystemV => {
                let argument_registers = vec![
                    il::scalar("$a0", 32), il::scalar("$a1", 32),
                    il::scalar("$a2", 32), il::scalar("$a3", 32)
                ];

                let mut preserved_registers = HashSet::new();
                preserved_registers.insert(il::scalar("$s0", 32));
                preserved_registers.insert(il::scalar("$s1", 32));
                preserved_registers.insert(il::scalar("$s2", 32));
                preserved_registers.insert(il::scalar("$s3", 32));
                preserved_registers.insert(il::scalar("$s4", 32));
                preserved_registers.insert(il::scalar("$s5", 32));
                preserved_registers.insert(il::scalar("$s6", 32));
                preserved_registers.insert(il::scalar("$s7", 32));
                preserved_registers.insert(il::scalar("$s8", 32));
                preserved_registers.insert(il::scalar("$sp", 32));
                preserved_registers.insert(il::scalar("$ra", 32));

                let mut trashed_registers = HashSet::new();
                trashed_registers.insert(il::scalar("$at", 32));
                trashed_registers.insert(il::scalar("$v0", 32));
                trashed_registers.insert(il::scalar("$v1", 32));
                trashed_registers.insert(il::scalar("$a0", 32));
                trashed_registers.insert(il::scalar("$a1", 32));
                trashed_registers.insert(il::scalar("$a2", 32));
                trashed_registers.insert(il::scalar("$a3", 32));
                trashed_registers.insert(il::scalar("$t0", 32));
                trashed_registers.insert(il::scalar("$t1", 32));
                trashed_registers.insert(il::scalar("$t2", 32));
                trashed_registers.insert(il::scalar("$t3", 32));
                trashed_registers.insert(il::scalar("$t4", 32));
                trashed_registers.insert(il::scalar("$t5", 32));
                trashed_registers.insert(il::scalar("$t6", 32));
                trashed_registers.insert(il::scalar("$t7", 32));
                trashed_registers.insert(il::scalar("$t8", 32));
                trashed_registers.insert(il::scalar("$t9", 32));

                let return_type = ReturnAddressType::Register(il::scalar("$ra", 32));

                CallingConvention {
                    argument_registers: argument_registers,
                    preserved_registers: preserved_registers,
                    trashed_registers: trashed_registers,
                    stack_argument_offset: 0,
                    stack_argument_length: 4,
                    return_address_type: return_type,
                    return_register: il::scalar("$v0", 32)
                }
            },
            CallingConventionType::Cdecl => {
                let mut preserved_registers = HashSet::new();
                preserved_registers.insert(il::scalar("ebx", 32));
                preserved_registers.insert(il::scalar("edi", 32));
                preserved_registers.insert(il::scalar("esi", 32));
                preserved_registers.insert(il::scalar("ebp", 32));
                preserved_registers.insert(il::scalar("esp", 32));

                let mut trashed_registers = HashSet::new();
                trashed_registers.insert(il::scalar("eax", 32));
                trashed_registers.insert(il::scalar("ecx", 32));
                trashed_registers.insert(il::scalar("edx", 32));

                let return_type = ReturnAddressType::Register(il::scalar("esp", 32));

                CallingConvention {
                    argument_registers: Vec::new(),
                    preserved_registers: preserved_registers,
                    trashed_registers: trashed_registers,
                    stack_argument_offset: 4,
                    stack_argument_length: 4,
                    return_address_type: return_type,
                    return_register: il::scalar("eax", 32)
                }
            },
        }
    }

    /// Get the registers the first n arguments are passed in.
    pub fn argument_registers(&self) -> &[il::Scalar] {
        &self.argument_registers
    }

    /// Get the registers preserved across function calls.
    pub fn preserved_registers(&self) -> &HashSet<il::Scalar> {
        &self.preserved_registers
    }

    /// Get the registers trashed across function calls.
    pub fn trashed_registers(&self) -> &HashSet<il::Scalar> {
        &self.trashed_registers
    }

    /// Get the length of an argument on the stack in _bytes, not bits_.
    ///
    /// We would expect this to be natural register-width of the architecture.
    pub fn stack_argument_length(&self) -> usize {
        self.stack_argument_length
    }

    /// Get the stack offset to the first argument passed on the stack in
    /// _bytes, not bits_.
    ///
    /// We would expect this to be immediately above the return address, if the
    /// return address is stored on the stack.
    pub fn stack_argument_offset(&self) -> usize {
        self.stack_argument_offset
    }

    /// How the return address is specified for function calls.
    pub fn return_address_type(&self) -> &ReturnAddressType {
        &self.return_address_type
    }

    /// The register returned values is given in.
    pub fn return_register(&self) -> &il::Scalar {
        &self.return_register
    }

    /// Get the type for the given argument, starting with 0 index.
    pub fn argument_type(&self, argument_number: usize) -> ArgumentType {
        if argument_number >= self.argument_registers.len() {
            let n = argument_number - self.argument_registers.len();
            let offset = self.stack_argument_offset + (self.stack_argument_length * n);
            ArgumentType::Stack(offset)
        }
        else {
            ArgumentType::Register(self.argument_registers[argument_number].clone())
        }
    }

    /// Is the given register preserved.
    pub fn is_preserved(&self, scalar: &il::Scalar) -> PartialBoolean {
        if self.preserved_registers.contains(scalar) {
            PartialBoolean::True
        }
        else if self.trashed_registers.contains(scalar) {
            PartialBoolean::False
        }
        else {
            PartialBoolean::Unknown
        }
    }

    /// Is the given register trashed.
    pub fn is_trashed(&self, scalar: &il::Scalar) -> PartialBoolean {
        if self.trashed_registers.contains(scalar) {
            PartialBoolean::True
        }
        else if self.preserved_registers.contains(scalar) {
            PartialBoolean::False
        }
        else {
            PartialBoolean::Unknown
        }
    }
}