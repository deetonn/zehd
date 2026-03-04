use crate::error::{RuntimeError, RuntimeErrorCode};
use crate::frame::{CallFrame, ChunkSource};
use crate::{Context, VmBackend};
use zehd_rune::chunk::Chunk;
use zehd_rune::op::{decode_u16, Op};
use zehd_rune::value::Value;

const MAX_CALL_DEPTH: usize = 256;

// ── StackVm ────────────────────────────────────────────────────

pub struct StackVm {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Vec<Value>,
    current_self: Option<Value>,
}

impl Default for StackVm {
    fn default() -> Self {
        Self::new()
    }
}

impl StackVm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: Vec::new(),
            current_self: None,
        }
    }

    /// Execute a function from the module by index, passing arguments.
    pub fn call_function(
        &mut self,
        func_index: u16,
        args: Vec<Value>,
        context: &Context,
    ) -> Result<Value, RuntimeError> {
        let func = context
            .module
            .functions
            .get(func_index as usize)
            .ok_or_else(|| {
                RuntimeError::err(
                    RuntimeErrorCode::R152,
                    format!("function index {func_index} out of bounds"),
                )
                .build()
            })?;

        let arity = func.chunk.arity;
        let arg_count = args.len() as u8;
        if arg_count != arity {
            return Err(RuntimeError::err(
                RuntimeErrorCode::R150,
                format!(
                    "expected {arity} arguments but got {arg_count}"
                ),
            )
            .build());
        }

        // Push function value as callee slot, then arguments
        self.stack.push(Value::Function(func_index));
        let stack_base = self.stack.len();
        for arg in args {
            self.stack.push(arg);
        }

        // Reserve extra local slots beyond arity
        let local_count = func.chunk.local_count as usize;
        let extra = local_count.saturating_sub(arity as usize);
        for _ in 0..extra {
            self.stack.push(Value::Unit);
        }

        self.frames.push(CallFrame {
            source: ChunkSource::Function(func_index),
            ip: 0,
            stack_base,
        });

        self.run(context)
    }

    /// Execute a handler chunk (e.g. get/post block).
    pub fn execute_handler(
        &mut self,
        handler_index: usize,
        context: &Context,
        self_value: Value,
    ) -> Result<Value, RuntimeError> {
        let handler = context
            .module
            .handlers
            .get(handler_index)
            .ok_or_else(|| {
                RuntimeError::err(
                    RuntimeErrorCode::R190,
                    format!("handler index {handler_index} out of bounds"),
                )
                .build()
            })?;

        let local_count = handler.chunk.local_count as usize;

        // Set self context for GetSelf opcode
        self.current_self = Some(self_value);

        // No callee slot for handlers — push directly
        let stack_base = self.stack.len();
        for _ in 0..local_count {
            self.stack.push(Value::Unit);
        }

        self.frames.push(CallFrame {
            source: ChunkSource::Main,
            ip: 0,
            stack_base,
        });

        // We need to execute the handler's chunk, not the main chunk
        let result = self.run_chunk(&handler.chunk, context);

        // Clear self context after handler returns
        self.current_self = None;

        result
    }
}

impl VmBackend for StackVm {
    fn execute(
        &mut self,
        chunk: &Chunk,
        context: &Context,
    ) -> Result<Value, RuntimeError> {
        let local_count = chunk.local_count as usize;
        let stack_base = self.stack.len();

        // Reserve local variable slots
        for _ in 0..local_count {
            self.stack.push(Value::Unit);
        }

        self.frames.push(CallFrame {
            source: ChunkSource::Main,
            ip: 0,
            stack_base,
        });

        self.run_chunk(chunk, context)
    }
}

impl StackVm {
    /// Run the dispatch loop using the main chunk (for execute/execute_handler).
    fn run_chunk(
        &mut self,
        main_chunk: &Chunk,
        context: &Context,
    ) -> Result<Value, RuntimeError> {
        self.run_with_main(Some(main_chunk), context)
    }

    /// Run the dispatch loop using only function chunks from context.
    fn run(&mut self, context: &Context) -> Result<Value, RuntimeError> {
        self.run_with_main(None, context)
    }

    /// Core dispatch loop.
    fn run_with_main(
        &mut self,
        main_chunk: Option<&Chunk>,
        context: &Context,
    ) -> Result<Value, RuntimeError> {
        let entry_frame_depth = self.frames.len() - 1;

        loop {
            let frame = self.frames.last().ok_or_else(|| {
                RuntimeError::err(RuntimeErrorCode::R190, "no active call frame")
                    .build()
            })?;

            let chunk = match frame.source {
                ChunkSource::Main => main_chunk.ok_or_else(|| {
                    RuntimeError::err(
                        RuntimeErrorCode::R190,
                        "main chunk not available",
                    )
                    .build()
                })?,
                ChunkSource::Function(idx) => {
                    &context.module.functions[idx as usize].chunk
                }
            };

            let ip = frame.ip;
            let stack_base = frame.stack_base;

            // End of bytecode — return Unit
            if ip >= chunk.code.len() {
                let result = if self.stack.len() > stack_base {
                    self.stack.pop().unwrap_or(Value::Unit)
                } else {
                    Value::Unit
                };

                self.stack.truncate(stack_base.saturating_sub(1).max(0));
                self.frames.pop();

                if self.frames.len() < entry_frame_depth || self.frames.is_empty() {
                    return Ok(result);
                }
                self.stack.push(result);
                continue;
            }

            let byte = chunk.code[ip];
            let op = Op::from_byte(byte).ok_or_else(|| {
                RuntimeError::err(
                    RuntimeErrorCode::R140,
                    format!("unknown opcode 0x{byte:02X}"),
                )
                .span_from_chunk(chunk, ip)
                .build()
            })?;

            // Advance ip past opcode
            self.frames.last_mut().unwrap().ip = ip + 1;

            match op {
                // ── Constants & Stack ─────────────────────────────
                Op::Constant => {
                    let idx = self.read_u16(chunk)?;
                    let value = chunk
                        .constants
                        .get(idx as usize)
                        .cloned()
                        .ok_or_else(|| {
                            RuntimeError::err(
                                RuntimeErrorCode::R190,
                                format!("constant index {idx} out of bounds"),
                            )
                            .build()
                        })?;
                    self.stack.push(value);
                }
                Op::True => self.stack.push(Value::Bool(true)),
                Op::False => self.stack.push(Value::Bool(false)),
                Op::None => self.stack.push(Value::None),
                Op::Unit => self.stack.push(Value::Unit),
                Op::Pop => {
                    self.pop()?;
                }
                Op::Dup => {
                    let val = self.peek()?.clone();
                    self.stack.push(val);
                }

                // ── Variables ─────────────────────────────────────
                Op::GetLocal => {
                    let slot = self.read_u16(chunk)?;
                    let idx = stack_base + slot as usize;
                    let val = self.stack.get(idx).cloned().ok_or_else(|| {
                        RuntimeError::err(
                            RuntimeErrorCode::R110,
                            format!("local slot {slot} out of bounds"),
                        )
                        .build()
                    })?;
                    self.stack.push(val);
                }
                Op::SetLocal => {
                    let slot = self.read_u16(chunk)?;
                    let val = self.pop()?;
                    let idx = stack_base + slot as usize;
                    if idx >= self.stack.len() {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R110,
                            format!("local slot {slot} out of bounds"),
                        )
                        .build());
                    }
                    self.stack[idx] = val;
                }
                Op::GetGlobal => {
                    let slot = self.read_u16(chunk)?;
                    let val = self
                        .globals
                        .get(slot as usize)
                        .cloned()
                        .ok_or_else(|| {
                            RuntimeError::err(
                                RuntimeErrorCode::R130,
                                format!("undefined global at index {slot}"),
                            )
                            .build()
                        })?;
                    self.stack.push(val);
                }
                Op::SetGlobal => {
                    let slot = self.read_u16(chunk)?;
                    let val = self.pop()?;
                    let idx = slot as usize;
                    if idx >= self.globals.len() {
                        self.globals.resize(idx + 1, Value::Unit);
                    }
                    self.globals[idx] = val;
                }

                // ── Integer Arithmetic ───────────────────────────
                Op::AddInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Int(a.wrapping_add(b)));
                }
                Op::SubInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Int(a.wrapping_sub(b)));
                }
                Op::MulInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Int(a.wrapping_mul(b)));
                }
                Op::DivInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    if b == 0 {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R100,
                            "division by zero",
                        )
                        .span_from_chunk(chunk, ip)
                        .build());
                    }
                    self.stack.push(Value::Int(a / b));
                }
                Op::ModInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    if b == 0 {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R100,
                            "modulo by zero",
                        )
                        .span_from_chunk(chunk, ip)
                        .build());
                    }
                    self.stack.push(Value::Int(a % b));
                }
                Op::NegInt => {
                    let val = self.pop_int(chunk, ip)?;
                    self.stack.push(Value::Int(-val));
                }

                // ── Float Arithmetic ─────────────────────────────
                Op::AddFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Float(a + b));
                }
                Op::SubFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Float(a - b));
                }
                Op::MulFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Float(a * b));
                }
                Op::DivFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    if b == 0.0 {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R100,
                            "float division by zero",
                        )
                        .span_from_chunk(chunk, ip)
                        .build());
                    }
                    self.stack.push(Value::Float(a / b));
                }
                Op::NegFloat => {
                    let val = self.pop_float(chunk, ip)?;
                    self.stack.push(Value::Float(-val));
                }

                // ── String Ops ───────────────────────────────────
                Op::AddStr => {
                    let b = self.pop_string(chunk, ip)?;
                    let a = self.pop_string(chunk, ip)?;
                    self.stack.push(Value::String(a + &b));
                }

                // ── Integer Comparison ───────────────────────────
                Op::EqInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a == b));
                }
                Op::NeqInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a != b));
                }
                Op::LtInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a < b));
                }
                Op::GtInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a > b));
                }
                Op::LeqInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a <= b));
                }
                Op::GeqInt => {
                    let (a, b) = self.pop_two_ints(chunk, ip)?;
                    self.stack.push(Value::Bool(a >= b));
                }

                // ── Float Comparison ─────────────────────────────
                Op::EqFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a == b));
                }
                Op::NeqFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a != b));
                }
                Op::LtFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a < b));
                }
                Op::GtFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a > b));
                }
                Op::LeqFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a <= b));
                }
                Op::GeqFloat => {
                    let (a, b) = self.pop_two_floats(chunk, ip)?;
                    self.stack.push(Value::Bool(a >= b));
                }

                // ── String Comparison ────────────────────────────
                Op::EqStr => {
                    let b = self.pop_string(chunk, ip)?;
                    let a = self.pop_string(chunk, ip)?;
                    self.stack.push(Value::Bool(a == b));
                }
                Op::NeqStr => {
                    let b = self.pop_string(chunk, ip)?;
                    let a = self.pop_string(chunk, ip)?;
                    self.stack.push(Value::Bool(a != b));
                }

                // ── Bool Comparison ──────────────────────────────
                Op::EqBool => {
                    let b = self.pop_bool(chunk, ip)?;
                    let a = self.pop_bool(chunk, ip)?;
                    self.stack.push(Value::Bool(a == b));
                }
                Op::NeqBool => {
                    let b = self.pop_bool(chunk, ip)?;
                    let a = self.pop_bool(chunk, ip)?;
                    self.stack.push(Value::Bool(a != b));
                }

                // ── Logical ──────────────────────────────────────
                Op::Not => {
                    let val = self.pop_bool(chunk, ip)?;
                    self.stack.push(Value::Bool(!val));
                }

                // ── Control Flow ─────────────────────────────────
                Op::Jump => {
                    let offset = self.read_u16(chunk)?;
                    self.frames.last_mut().unwrap().ip += offset as usize;
                }
                Op::JumpIfFalse => {
                    let offset = self.read_u16(chunk)?;
                    let cond = self.pop_bool(chunk, ip)?;
                    if !cond {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                Op::JumpIfTrue => {
                    let offset = self.read_u16(chunk)?;
                    let cond = self.pop_bool(chunk, ip)?;
                    if cond {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                Op::Loop => {
                    let offset = self.read_u16(chunk)?;
                    self.frames.last_mut().unwrap().ip -= offset as usize;
                }

                // ── Functions ────────────────────────────────────
                Op::Closure => {
                    let func_idx = self.read_u16(chunk)?;
                    self.stack.push(Value::Function(func_idx));
                }
                Op::Call => {
                    let arg_count = self.read_u8(chunk)?;
                    self.call_value(arg_count, context, main_chunk)?;
                }
                Op::CallNative => {
                    let native_id = self.read_u16(chunk)?;
                    let arg_count = self.read_u8(chunk)?;

                    // Pop arguments from the stack.
                    let start = self.stack.len().saturating_sub(arg_count as usize);
                    let args: Vec<Value> = self.stack.drain(start..).collect();

                    let func = context
                        .native_fns
                        .get(native_id as usize)
                        .ok_or_else(|| {
                            RuntimeError::err(
                                RuntimeErrorCode::R152,
                                format!("native function id {native_id} out of bounds"),
                            )
                            .span_from_chunk(chunk, ip)
                            .build()
                        })?;

                    let result = func(&args)?;
                    self.stack.push(result);
                }
                Op::Return => {
                    let result = self.pop().unwrap_or(Value::Unit);
                    let frame = self.frames.pop().ok_or_else(|| {
                        RuntimeError::err(
                            RuntimeErrorCode::R190,
                            "return with no call frame",
                        )
                        .build()
                    })?;

                    // Truncate stack: remove locals + callee slot
                    self.stack
                        .truncate(frame.stack_base.saturating_sub(1));

                    if self.frames.len() <= entry_frame_depth
                        || self.frames.is_empty()
                    {
                        return Ok(result);
                    }
                    self.stack.push(result);
                }

                // ── Strings ──────────────────────────────────────
                Op::Concat => {
                    let count = self.read_u16(chunk)?;
                    let count = count as usize;
                    if self.stack.len() < count {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R110,
                            format!(
                                "stack underflow: need {count} values for Concat"
                            ),
                        )
                        .build());
                    }
                    let start = self.stack.len() - count;
                    let mut result = String::new();
                    for val in &self.stack[start..] {
                        result.push_str(&value_to_string(val));
                    }
                    self.stack.truncate(start);
                    self.stack.push(Value::String(result));
                }
                Op::ToString => {
                    let val = self.pop()?;
                    self.stack
                        .push(Value::String(value_to_string(&val)));
                }

                // ── HTTP Context ─────────────────────────────────
                Op::GetSelf => {
                    let val = self.current_self.clone().ok_or_else(|| {
                        RuntimeError::err(
                            RuntimeErrorCode::R190,
                            "self is not available outside of a handler context",
                        )
                        .span_from_chunk(chunk, ip)
                        .build()
                    })?;
                    self.stack.push(val);
                }

                // ── Data Structures ─────────────────────────────────
                Op::GetField => {
                    let name_idx = self.read_u16(chunk)?;
                    let name = match chunk.constants.get(name_idx as usize) {
                        Some(Value::String(s)) => s.clone(),
                        _ => {
                            return Err(RuntimeError::err(
                                RuntimeErrorCode::R190,
                                format!("field name constant {name_idx} is not a string"),
                            )
                            .span_from_chunk(chunk, ip)
                            .build());
                        }
                    };
                    let obj = self.pop()?;
                    match obj {
                        Value::Object(fields) => {
                            let val = fields
                                .iter()
                                .find(|(k, _)| k == &name)
                                .map(|(_, v)| v.clone())
                                .ok_or_else(|| {
                                    RuntimeError::err(
                                        RuntimeErrorCode::R120,
                                        format!("object has no field '{name}'"),
                                    )
                                    .span_from_chunk(chunk, ip)
                                    .build()
                                })?;
                            self.stack.push(val);
                        }
                        other => {
                            return Err(RuntimeError::err(
                                RuntimeErrorCode::R120,
                                format!(
                                    "cannot access field '{name}' on {}",
                                    type_name(&other)
                                ),
                            )
                            .span_from_chunk(chunk, ip)
                            .build());
                        }
                    }
                }
                Op::MakeObject => {
                    let count = self.read_u16(chunk)? as usize;
                    if self.stack.len() < count * 2 {
                        return Err(RuntimeError::err(
                            RuntimeErrorCode::R110,
                            format!(
                                "stack underflow: need {} values for MakeObject",
                                count * 2
                            ),
                        )
                        .build());
                    }
                    let start = self.stack.len() - count * 2;
                    let mut fields = Vec::with_capacity(count);
                    for i in 0..count {
                        let key_idx = start + i * 2;
                        let val_idx = start + i * 2 + 1;
                        let key = match &self.stack[key_idx] {
                            Value::String(s) => s.clone(),
                            other => {
                                return Err(RuntimeError::err(
                                    RuntimeErrorCode::R120,
                                    format!(
                                        "object key must be String, got {}",
                                        type_name(other)
                                    ),
                                )
                                .span_from_chunk(chunk, ip)
                                .build());
                            }
                        };
                        let val = self.stack[val_idx].clone();
                        fields.push((key, val));
                    }
                    self.stack.truncate(start);
                    self.stack.push(Value::Object(fields));
                }

                // ── Unimplemented (Session 2+) ───────────────────
                Op::MakeList
                | Op::SetField
                | Op::GetIndex
                | Op::SetIndex
                | Op::WrapSome
                | Op::WrapOk
                | Op::WrapErr
                | Op::Unwrap
                | Op::TryOp
                | Op::MakeEnum
                | Op::TestVariant
                | Op::UnwrapVariant
                | Op::TestEqual => {
                    return Err(RuntimeError::err(
                        RuntimeErrorCode::R140,
                        format!("unimplemented opcode: {op}"),
                    )
                    .span_from_chunk(chunk, ip)
                    .build());
                }
            }
        }
    }

    // ── Stack helpers ────────────────────────────────────────────

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or_else(|| {
            RuntimeError::err(RuntimeErrorCode::R110, "stack underflow")
                .build()
        })
    }

    fn peek(&self) -> Result<&Value, RuntimeError> {
        self.stack.last().ok_or_else(|| {
            RuntimeError::err(RuntimeErrorCode::R110, "stack underflow")
                .build()
        })
    }

    fn pop_int(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<i64, RuntimeError> {
        match self.pop()? {
            Value::Int(n) => Ok(n),
            other => Err(RuntimeError::err(
                RuntimeErrorCode::R120,
                format!("expected Int, got {}", type_name(&other)),
            )
            .span_from_chunk(chunk, op_ip)
            .build()),
        }
    }

    fn pop_two_ints(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<(i64, i64), RuntimeError> {
        let b = self.pop_int(chunk, op_ip)?;
        let a = self.pop_int(chunk, op_ip)?;
        Ok((a, b))
    }

    fn pop_float(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<f64, RuntimeError> {
        match self.pop()? {
            Value::Float(n) => Ok(n),
            other => Err(RuntimeError::err(
                RuntimeErrorCode::R120,
                format!("expected Float, got {}", type_name(&other)),
            )
            .span_from_chunk(chunk, op_ip)
            .build()),
        }
    }

    fn pop_two_floats(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<(f64, f64), RuntimeError> {
        let b = self.pop_float(chunk, op_ip)?;
        let a = self.pop_float(chunk, op_ip)?;
        Ok((a, b))
    }

    fn pop_string(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<String, RuntimeError> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            other => Err(RuntimeError::err(
                RuntimeErrorCode::R120,
                format!("expected String, got {}", type_name(&other)),
            )
            .span_from_chunk(chunk, op_ip)
            .build()),
        }
    }

    fn pop_bool(
        &mut self,
        chunk: &Chunk,
        op_ip: usize,
    ) -> Result<bool, RuntimeError> {
        match self.pop()? {
            Value::Bool(b) => Ok(b),
            other => Err(RuntimeError::err(
                RuntimeErrorCode::R120,
                format!("expected Bool, got {}", type_name(&other)),
            )
            .span_from_chunk(chunk, op_ip)
            .build()),
        }
    }

    // ── Instruction reading ──────────────────────────────────────

    fn read_u16(&mut self, chunk: &Chunk) -> Result<u16, RuntimeError> {
        let frame = self.frames.last_mut().unwrap();
        let ip = frame.ip;
        if ip + 1 >= chunk.code.len() {
            return Err(RuntimeError::err(
                RuntimeErrorCode::R190,
                "unexpected end of bytecode reading u16",
            )
            .build());
        }

        // Re-resolve chunk for reading
        let hi = chunk.code[ip];
        let lo = chunk.code[ip + 1];
        frame.ip += 2;
        Ok(decode_u16(hi, lo))
    }

    fn read_u8(&mut self, chunk: &Chunk) -> Result<u8, RuntimeError> {
        let frame = self.frames.last_mut().unwrap();
        let ip = frame.ip;
        if ip >= chunk.code.len() {
            return Err(RuntimeError::err(
                RuntimeErrorCode::R190,
                "unexpected end of bytecode reading u8",
            )
            .build());
        }
        let val = chunk.code[ip];
        frame.ip += 1;
        Ok(val)
    }

    // ── Function calls ───────────────────────────────────────────

    fn call_value(
        &mut self,
        arg_count: u8,
        context: &Context,
        _main_chunk: Option<&Chunk>,
    ) -> Result<(), RuntimeError> {
        if self.frames.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::err(
                RuntimeErrorCode::R151,
                format!(
                    "call stack overflow (max {MAX_CALL_DEPTH} frames)"
                ),
            )
            .build());
        }

        // The callee sits below the arguments
        let callee_idx = self.stack.len() - 1 - arg_count as usize;
        let callee = self.stack[callee_idx].clone();

        match callee {
            Value::Function(func_idx) => {
                let func = context
                    .module
                    .functions
                    .get(func_idx as usize)
                    .ok_or_else(|| {
                        RuntimeError::err(
                            RuntimeErrorCode::R152,
                            format!(
                                "function index {func_idx} out of bounds"
                            ),
                        )
                        .build()
                    })?;

                let arity = func.chunk.arity;
                if arg_count != arity {
                    return Err(RuntimeError::err(
                        RuntimeErrorCode::R150,
                        format!(
                            "function '{}' expected {arity} arguments but got {arg_count}",
                            func.name
                        ),
                    )
                    .build());
                }

                let stack_base = callee_idx + 1;
                let local_count = func.chunk.local_count as usize;
                let extra =
                    local_count.saturating_sub(arity as usize);
                for _ in 0..extra {
                    self.stack.push(Value::Unit);
                }

                self.frames.push(CallFrame {
                    source: ChunkSource::Function(func_idx),
                    ip: 0,
                    stack_base,
                });
                Ok(())
            }
            other => Err(RuntimeError::err(
                RuntimeErrorCode::R121,
                format!(
                    "cannot call value of type {}",
                    type_name(&other)
                ),
            )
            .build()),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────

fn type_name(val: &Value) -> &'static str {
    match val {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::String(_) => "String",
        Value::None => "None",
        Value::Unit => "Unit",
        Value::List(_) => "List",
        Value::Object(_) => "Object",
        Value::Function(_) => "Function",
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "None".to_string(),
        Value::Unit => "()".to_string(),
        Value::List(items) => {
            let inner: Vec<String> =
                items.iter().map(value_to_string).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Object(fields) => {
            let inner: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", value_to_string(v)))
                .collect();
            format!("{{ {} }}", inner.join(", "))
        }
        Value::Function(idx) => format!("<fn:{idx}>"),
    }
}

// ── RuntimeErrorBuilder extension ────────────────────────────

trait SpanFromChunk {
    fn span_from_chunk(self, chunk: &Chunk, ip: usize) -> Self;
}

impl SpanFromChunk for crate::error::RuntimeErrorBuilder {
    fn span_from_chunk(self, chunk: &Chunk, ip: usize) -> Self {
        if let Some(span) = chunk.span_at(ip as u32) {
            self.span(span)
        } else {
            self
        }
    }
}
