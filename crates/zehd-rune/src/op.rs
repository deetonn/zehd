use std::fmt;

// ── Opcodes ────────────────────────────────────────────────────

/// Bytecode opcodes for the zehd stack-based VM.
///
/// Operands are encoded inline as big-endian u16 or u8 immediately
/// after the opcode byte. Type-directed arithmetic/comparison ops
/// are resolved at compile time using the TypeTable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Op {
    // ── Constants & Stack (0x00–0x0F) ────────────────────────────
    /// Push constant from pool. Operand: u16 index.
    Constant = 0x00,
    /// Push `true`.
    True = 0x01,
    /// Push `false`.
    False = 0x02,
    /// Push `Option::None`.
    None = 0x03,
    /// Push `()` unit value.
    Unit = 0x04,
    /// Discard top of stack.
    Pop = 0x05,
    /// Duplicate top of stack.
    Dup = 0x06,

    // ── Variables (0x10–0x1F) ────────────────────────────────────
    /// Push local variable. Operand: u16 slot.
    GetLocal = 0x10,
    /// Store into local variable. Operand: u16 slot.
    SetLocal = 0x11,
    /// Push global variable. Operand: u16 index.
    GetGlobal = 0x12,
    /// Store into global variable. Operand: u16 index.
    SetGlobal = 0x13,

    // ── Integer Arithmetic (0x20–0x2F) ──────────────────────────
    AddInt = 0x20,
    SubInt = 0x21,
    MulInt = 0x22,
    DivInt = 0x23,
    ModInt = 0x24,
    NegInt = 0x25,

    // ── Float Arithmetic (0x28–0x2F) ────────────────────────────
    AddFloat = 0x28,
    SubFloat = 0x29,
    MulFloat = 0x2A,
    DivFloat = 0x2B,
    NegFloat = 0x2C,

    // ── String Ops (0x2E–0x2F) ──────────────────────────────────
    AddStr = 0x2E,

    // ── Integer Comparison (0x30–0x3F) ──────────────────────────
    EqInt = 0x30,
    NeqInt = 0x31,
    LtInt = 0x32,
    GtInt = 0x33,
    LeqInt = 0x34,
    GeqInt = 0x35,

    // ── Float Comparison (0x38–0x3F) ────────────────────────────
    EqFloat = 0x38,
    NeqFloat = 0x39,
    LtFloat = 0x3A,
    GtFloat = 0x3B,
    LeqFloat = 0x3C,
    GeqFloat = 0x3D,

    // ── String Comparison (0x40–0x41) ───────────────────────────
    EqStr = 0x40,
    NeqStr = 0x41,

    // ── Bool Comparison (0x42–0x43) ─────────────────────────────
    EqBool = 0x42,
    NeqBool = 0x43,

    // ── Logical (0x48) ──────────────────────────────────────────
    Not = 0x48,

    // ── Control Flow (0x50–0x5F) ────────────────────────────────
    /// Unconditional forward jump. Operand: u16 offset.
    Jump = 0x50,
    /// Jump if top is false (consumes). Operand: u16 offset.
    JumpIfFalse = 0x51,
    /// Jump if top is true (consumes). Operand: u16 offset.
    JumpIfTrue = 0x52,
    /// Backward jump (loop). Operand: u16 offset.
    Loop = 0x53,

    // ── Functions (0x60–0x6F) ───────────────────────────────────
    /// Call function. Operand: u8 arg count.
    Call = 0x60,
    /// Return from function.
    Return = 0x61,
    /// Push closure. Operand: u16 function index.
    Closure = 0x62,
    /// Call a native (Rust) function. Operands: u16 native_fn_id + u8 arg_count.
    CallNative = 0x63,
    /// Call a user-defined module function. Operands: u16 module_fn_id + u8 arg_count.
    CallModule = 0x67,
    /// Call a built-in method on a value. Operands: u16 method_id + u8 arg_count.
    CallMethod = 0x64,

    // ── Data Structures (0x70–0x7F) ─────────────────────────────
    /// Create list from N stack values. Operand: u16 count.
    MakeList = 0x70,
    /// Create object from N key-value pairs. Operand: u16 pair count.
    MakeObject = 0x71,
    /// Get field from object. Operand: u16 name constant index.
    GetField = 0x72,
    /// Set field on object. Operand: u16 name constant index.
    SetField = 0x73,
    /// Get by index: `[obj idx -- value]`.
    GetIndex = 0x74,
    /// Set by index: `[obj idx val -- obj]`.
    SetIndex = 0x75,

    // ── Option/Result/Enum (0x80–0x8F) ──────────────────────────
    /// Wrap top in Some: `[val -- Some(val)]`.
    WrapSome = 0x80,
    /// Wrap top in Ok: `[val -- Ok(val)]`.
    WrapOk = 0x81,
    /// Wrap top in Err: `[val -- Err(val)]`.
    WrapErr = 0x82,
    /// Unwrap Option/Result (panics on None/Err).
    Unwrap = 0x83,
    /// Try operator (?): unwrap Ok or early-return Err.
    TryOp = 0x84,
    /// Construct enum variant. Operands: u16 type_idx, u16 variant_idx.
    MakeEnum = 0x85,

    // ── Pattern Matching (0x90–0x9F) ────────────────────────────
    /// Test if top is specific variant. Operands: u16 type_idx, u16 variant_idx.
    TestVariant = 0x90,
    /// Extract inner value from enum variant.
    UnwrapVariant = 0x91,
    /// Test equality for pattern matching.
    TestEqual = 0x92,

    // ── Strings (0xA0–0xAF) ─────────────────────────────────────
    /// Concatenate N string parts. Operand: u16 count.
    Concat = 0xA0,
    /// Convert top to string.
    ToString = 0xA1,

    // ── HTTP / DI (0xB0–0xBF) ──────────────────────────────────
    /// Push implicit `self` context in handlers.
    GetSelf = 0xB0,
    /// Store value in DI registry. Operand: u16 constant index (type name).
    /// Stack: [value] → [Unit]
    Provide = 0xB1,
    /// Load value from DI registry. Operand: u16 constant index (type name).
    /// Stack: [] → [value]
    Inject = 0xB2,
}

impl Op {
    /// Decode a single opcode from its byte representation.
    pub fn from_byte(byte: u8) -> Option<Op> {
        // Safety: we only match known values
        match byte {
            0x00 => Some(Op::Constant),
            0x01 => Some(Op::True),
            0x02 => Some(Op::False),
            0x03 => Some(Op::None),
            0x04 => Some(Op::Unit),
            0x05 => Some(Op::Pop),
            0x06 => Some(Op::Dup),

            0x10 => Some(Op::GetLocal),
            0x11 => Some(Op::SetLocal),
            0x12 => Some(Op::GetGlobal),
            0x13 => Some(Op::SetGlobal),

            0x20 => Some(Op::AddInt),
            0x21 => Some(Op::SubInt),
            0x22 => Some(Op::MulInt),
            0x23 => Some(Op::DivInt),
            0x24 => Some(Op::ModInt),
            0x25 => Some(Op::NegInt),

            0x28 => Some(Op::AddFloat),
            0x29 => Some(Op::SubFloat),
            0x2A => Some(Op::MulFloat),
            0x2B => Some(Op::DivFloat),
            0x2C => Some(Op::NegFloat),

            0x2E => Some(Op::AddStr),

            0x30 => Some(Op::EqInt),
            0x31 => Some(Op::NeqInt),
            0x32 => Some(Op::LtInt),
            0x33 => Some(Op::GtInt),
            0x34 => Some(Op::LeqInt),
            0x35 => Some(Op::GeqInt),

            0x38 => Some(Op::EqFloat),
            0x39 => Some(Op::NeqFloat),
            0x3A => Some(Op::LtFloat),
            0x3B => Some(Op::GtFloat),
            0x3C => Some(Op::LeqFloat),
            0x3D => Some(Op::GeqFloat),

            0x40 => Some(Op::EqStr),
            0x41 => Some(Op::NeqStr),

            0x42 => Some(Op::EqBool),
            0x43 => Some(Op::NeqBool),

            0x48 => Some(Op::Not),

            0x50 => Some(Op::Jump),
            0x51 => Some(Op::JumpIfFalse),
            0x52 => Some(Op::JumpIfTrue),
            0x53 => Some(Op::Loop),

            0x60 => Some(Op::Call),
            0x61 => Some(Op::Return),
            0x62 => Some(Op::Closure),
            0x63 => Some(Op::CallNative),
            0x64 => Some(Op::CallMethod),
            0x67 => Some(Op::CallModule),

            0x70 => Some(Op::MakeList),
            0x71 => Some(Op::MakeObject),
            0x72 => Some(Op::GetField),
            0x73 => Some(Op::SetField),
            0x74 => Some(Op::GetIndex),
            0x75 => Some(Op::SetIndex),

            0x80 => Some(Op::WrapSome),
            0x81 => Some(Op::WrapOk),
            0x82 => Some(Op::WrapErr),
            0x83 => Some(Op::Unwrap),
            0x84 => Some(Op::TryOp),
            0x85 => Some(Op::MakeEnum),

            0x90 => Some(Op::TestVariant),
            0x91 => Some(Op::UnwrapVariant),
            0x92 => Some(Op::TestEqual),

            0xA0 => Some(Op::Concat),
            0xA1 => Some(Op::ToString),

            0xB0 => Some(Op::GetSelf),
            0xB1 => Some(Op::Provide),
            0xB2 => Some(Op::Inject),

            _ => Option::None,
        }
    }

    /// Number of operand bytes following this opcode.
    pub fn operand_size(self) -> usize {
        match self {
            // u16 operand
            Op::Constant
            | Op::GetLocal
            | Op::SetLocal
            | Op::GetGlobal
            | Op::SetGlobal
            | Op::Jump
            | Op::JumpIfFalse
            | Op::JumpIfTrue
            | Op::Loop
            | Op::Closure
            | Op::MakeList
            | Op::MakeObject
            | Op::GetField
            | Op::SetField
            | Op::Concat
            | Op::Provide
            | Op::Inject => 2,

            // u8 operand
            Op::Call => 1,

            // u16 + u8 operand (native_fn_id + arg_count)
            Op::CallNative | Op::CallModule | Op::CallMethod => 3,

            // Two u16 operands (type_idx + variant_idx)
            Op::MakeEnum | Op::TestVariant => 4,

            // No operand
            _ => 0,
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Op::Constant => "Constant",
            Op::True => "True",
            Op::False => "False",
            Op::None => "None",
            Op::Unit => "Unit",
            Op::Pop => "Pop",
            Op::Dup => "Dup",
            Op::GetLocal => "GetLocal",
            Op::SetLocal => "SetLocal",
            Op::GetGlobal => "GetGlobal",
            Op::SetGlobal => "SetGlobal",
            Op::AddInt => "AddInt",
            Op::SubInt => "SubInt",
            Op::MulInt => "MulInt",
            Op::DivInt => "DivInt",
            Op::ModInt => "ModInt",
            Op::NegInt => "NegInt",
            Op::AddFloat => "AddFloat",
            Op::SubFloat => "SubFloat",
            Op::MulFloat => "MulFloat",
            Op::DivFloat => "DivFloat",
            Op::NegFloat => "NegFloat",
            Op::AddStr => "AddStr",
            Op::EqInt => "EqInt",
            Op::NeqInt => "NeqInt",
            Op::LtInt => "LtInt",
            Op::GtInt => "GtInt",
            Op::LeqInt => "LeqInt",
            Op::GeqInt => "GeqInt",
            Op::EqFloat => "EqFloat",
            Op::NeqFloat => "NeqFloat",
            Op::LtFloat => "LtFloat",
            Op::GtFloat => "GtFloat",
            Op::LeqFloat => "LeqFloat",
            Op::GeqFloat => "GeqFloat",
            Op::EqStr => "EqStr",
            Op::NeqStr => "NeqStr",
            Op::EqBool => "EqBool",
            Op::NeqBool => "NeqBool",
            Op::Not => "Not",
            Op::Jump => "Jump",
            Op::JumpIfFalse => "JumpIfFalse",
            Op::JumpIfTrue => "JumpIfTrue",
            Op::Loop => "Loop",
            Op::Call => "Call",
            Op::Return => "Return",
            Op::Closure => "Closure",
            Op::CallNative => "CallNative",
            Op::CallMethod => "CallMethod",
            Op::CallModule => "CallModule",
            Op::MakeList => "MakeList",
            Op::MakeObject => "MakeObject",
            Op::GetField => "GetField",
            Op::SetField => "SetField",
            Op::GetIndex => "GetIndex",
            Op::SetIndex => "SetIndex",
            Op::WrapSome => "WrapSome",
            Op::WrapOk => "WrapOk",
            Op::WrapErr => "WrapErr",
            Op::Unwrap => "Unwrap",
            Op::TryOp => "TryOp",
            Op::MakeEnum => "MakeEnum",
            Op::TestVariant => "TestVariant",
            Op::UnwrapVariant => "UnwrapVariant",
            Op::TestEqual => "TestEqual",
            Op::Concat => "Concat",
            Op::ToString => "ToString",
            Op::GetSelf => "GetSelf",
            Op::Provide => "Provide",
            Op::Inject => "Inject",
        };
        write!(f, "{name}")
    }
}

// ── Encoding helpers ───────────────────────────────────────────

/// Encode a u16 value as two big-endian bytes.
pub fn encode_u16(value: u16) -> [u8; 2] {
    value.to_be_bytes()
}

/// Decode a u16 from two big-endian bytes.
pub fn decode_u16(hi: u8, lo: u8) -> u16 {
    u16::from_be_bytes([hi, lo])
}

// ── Disassembler ───────────────────────────────────────────────

/// Decoded instruction for testing and debug printing.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Simple(Op),
    U16(Op, u16),
    U8(Op, u8),
    U16U16(Op, u16, u16),
    U16U8(Op, u16, u8),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Simple(op) => write!(f, "{op}"),
            Instruction::U16(op, val) => write!(f, "{op}({val})"),
            Instruction::U8(op, val) => write!(f, "{op}({val})"),
            Instruction::U16U16(op, a, b) => write!(f, "{op}({a}, {b})"),
            Instruction::U16U8(op, a, b) => write!(f, "{op}({a}, {b})"),
        }
    }
}

/// Decode a bytecode stream into a list of instructions.
pub fn decode_ops(code: &[u8]) -> Vec<Instruction> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < code.len() {
        let Some(op) = Op::from_byte(code[i]) else {
            panic!("unknown opcode 0x{:02X} at offset {i}", code[i]);
        };
        i += 1;
        match op.operand_size() {
            0 => result.push(Instruction::Simple(op)),
            1 => {
                let val = code[i];
                i += 1;
                result.push(Instruction::U8(op, val));
            }
            2 => {
                let val = decode_u16(code[i], code[i + 1]);
                i += 2;
                result.push(Instruction::U16(op, val));
            }
            3 => {
                let a = decode_u16(code[i], code[i + 1]);
                let b = code[i + 2];
                i += 3;
                result.push(Instruction::U16U8(op, a, b));
            }
            4 => {
                let a = decode_u16(code[i], code[i + 1]);
                let b = decode_u16(code[i + 2], code[i + 3]);
                i += 4;
                result.push(Instruction::U16U16(op, a, b));
            }
            _ => unreachable!(),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_module_opcode_roundtrip() {
        assert_eq!(Op::from_byte(0x67), Some(Op::CallModule));
        assert_eq!(Op::CallModule.operand_size(), 3);
    }

    #[test]
    fn call_module_encode_decode() {
        let mut code = vec![0x67]; // CallModule
        code.extend_from_slice(&encode_u16(42)); // fn_id = 42
        code.push(3); // argc = 3

        let instructions = decode_ops(&code);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::U16U8(Op::CallModule, 42, 3));
    }

    #[test]
    fn call_module_display() {
        assert_eq!(format!("{}", Op::CallModule), "CallModule");
    }
}
