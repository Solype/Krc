use super::generation::Context;
use super::utils::{load_operand, var_size};
use crate::elf::instructions::Operand::Sym;
use crate::elf::instructions::register::{
    R8, R9, RBP, RCX, RDI, RDX, RSI, RSP, Register, XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
};
use crate::elf::instructions::{
    BinOp, ConvOp, ExtendOp, Instruction, LabelId, ShiftOp, Size, register::RAX,
};
use crate::samples::helpers::{
    BYTE, DWORD, QWORD, bin, call, convert, extend, imm, jcc, jmp, label, prefixed, reg, ret,
    shift, stack, string,
};

use crate::converter::ir::structs::{Label, Operand as IrOp, VarId};
use crate::elf::instructions::{
    ConditionCode, MemAddress, Operand as ElfOper, Prefix, StackOp, StringOp,
};
use crate::parser::postparser::structs::Type;

/// `ctx.offsets`: the offset under rbp of each variable, see var_offsets
pub fn convert_operand(op: &IrOp, ctx: &Context) -> ElfOper {
    match op {
        IrOp::Var(var) => {
            ElfOper::MemoryAddress(MemAddress::new().base(RBP).disp(-ctx.offsets[var.id]))
        }
        IrOp::Int(n, _, _) => ElfOper::Imm(*n),
        IrOp::Bool(b) => ElfOper::Imm(*b as i64),
        IrOp::Null => ElfOper::Imm(0),
        // a float has no immediate form: it is read from its constant in .rodata
        IrOp::Float(f) => {
            ElfOper::MemoryAddress(MemAddress::symbol(&ctx.float_names[*f]).rip_relative())
        }
        _ => unimplemented!(),
    }
}

/// `mov rsp, rbp ; pop rbp ; ret`: undoes the prologue and returns
pub fn func_epilogue() -> Vec<Instruction> {
    vec![
        bin(BinOp::Mov, reg(RSP), reg(RBP), None),
        stack(StackOp::Pop, reg(RBP), None),
        ret(),
    ]
}

/// `mov rax, op` then the epilogue: the value comes back in rax (System V)
pub fn generate_return(op: &IrOp, ctx: &Context) -> Vec<Instruction> {
    let mut epi = func_epilogue();
    if matches!(ctx.return_type, Some(Type::Float { .. })) {
        epi.insert(0, load_operand(XMM0, op, ctx, None));
    } else {
        epi.insert(0, load_operand(RAX, op, ctx, None));
    }
    epi
}

pub const PARAM_FREGS: [Register; 8] = [XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7];

/// The float argument of rank `i_f` into xmm`i_f`; not done yet, panics
pub fn float_parameter(op: &IrOp, i_f: &mut i32, ctx: &Context) -> Instruction {
    let Some(reg) = PARAM_FREGS.get(*i_f as usize) else {
        unimplemented!()
    };
    *i_f += 1;
    load_operand(*reg, op, ctx, None)
}

/// Registers of the integer arguments, in order (System V)
pub const PARAM_REGS: [Register; 6] = [RDI, RSI, RDX, RCX, R8, R9];

/// `mov` of the argument into the register of its rank `i`, then the next rank
pub fn normal_param(op: &IrOp, i: &mut i32, ctx: &Context) -> Instruction {
    // shortcut: no 7th integer argument (it goes on the stack), add it for calls of more than 6
    let Some(reg) = PARAM_REGS.get(*i as usize) else {
        unimplemented!()
    };
    *i += 1;
    load_operand(*reg, op, ctx, None)
}

/// The size in bytes of the value of `op`, and true for a 소수
pub fn operand_kind(op: &IrOp, ctx: &Context) -> (usize, bool, bool) {
    match op {
        IrOp::Var(var) => (var.ty.size(), var.ty.is_float(), var.ty.is_unsigned()),
        IrOp::Int(_, short, unsigned) => (if *short { 4 } else { 8 }, false, *unsigned),
        IrOp::Float(f) => (if ctx.floats[*f].1 { 4 } else { 8 }, true, false),
        IrOp::Bool(_) => (1, false, true),
        // an address
        _ => (8, false, true),
    }
}

/// `dest = src` from a 정수 to a wider one: movsxd for a signed 정수, `mov eax` (which clears the
/// upper half) for a 부호 없는 one
fn cast_between_two_non_float(
    dest: &VarId,
    src_op: ElfOper,
    src_size: usize,
    src_unsigned: bool,
    ctx: &Context,
) -> Vec<Instruction> {
    let mut instrs = vec![];

    match (src_size, src_unsigned) {
        // on 8 bytes, the value is already in rax: a constant or a widened 1-byte 정수
        (8, _) => {}
        (4, false) => instrs.push(extend(ExtendOp::Movsxd, RAX, src_op, DWORD, QWORD)),
        (4, true) => instrs.push(bin(BinOp::Mov, reg(RAX), src_op, DWORD)),
        _ => unimplemented!(),
    }
    instrs.push(bin(
        BinOp::Mov,
        convert_operand(&IrOp::Var(dest.clone()), ctx),
        reg(RAX),
        var_size(dest),
    ));
    instrs
}

/// cvtsi2sd reads a signed 정수: a 부호 없는 one ≥ 2^63 is halved, its lost bit kept in bit 0 so
/// it still rounds right, converted, then doubled, as gcc does
fn unsigned_64_to_float(dest: &VarId, src_op: ElfOper, ctx: &Context) -> Vec<Instruction> {
    let (big, done) = (LabelId::new(), LabelId::new());
    let cvt = if dest.ty.size() == 8 {
        ConvOp::Cvtsi2sd
    } else {
        ConvOp::Cvtsi2ss
    };
    let (q, f) = (Some(Size::U64), var_size(dest));
    let dest_op = convert_operand(&IrOp::Var(dest.clone()), ctx);
    vec![
        bin(BinOp::Mov, reg(RAX), src_op, q),
        bin(BinOp::Test, reg(RAX), reg(RAX), q),
        jcc(ConditionCode::S, big),
        convert(cvt, XMM0, reg(RAX), q),
        jmp(done),
        label(big),
        bin(BinOp::Mov, reg(RCX), reg(RAX), q),
        shift(ShiftOp::Shr, reg(RCX), imm(1), q),
        bin(BinOp::And, reg(RAX), imm(1), q),
        bin(BinOp::Or, reg(RCX), reg(RAX), q),
        convert(cvt, XMM0, reg(RCX), q),
        bin(BinOp::AddF, reg(XMM0), reg(XMM0), f),
        label(done),
        bin(BinOp::StoreF, dest_op, reg(XMM0), f),
    ]
}

/// `dest = 바꾸다(src)`: cvtss2sd / cvtsd2ss between two 소수, cvtsi2sd from a 정수, cvttsd2si
/// to a 정수 (truncated toward 0, as in C), see cast_between_two_non_float to a wider 정수; else a
/// copy
pub fn generate_cast(dest: &VarId, src: &IrOp, ctx: &Context) -> Vec<Instruction> {
    let (src_size, src_float, src_unsigned) = operand_kind(src, ctx);
    let (dest_size, dest_float) = (dest.ty.size(), dest.ty.is_float());
    let same_kind = src_float == dest_float && (src_size == dest_size || !dest_float);
    // to a smaller 정수, an address or the same type: the low bytes, as a copy sized by dest
    if same_kind && dest_size <= src_size {
        return generate_copy(dest, src, ctx);
    }
    let mut instrs = vec![];
    // cvt and movsx take no immediate: a 정수 or 논리 constant goes through rax, on 8 bytes
    let (mut src_op, mut src_size) = match src {
        IrOp::Int(_, short, _) => {
            let size = if *short { Size::U32 } else { Size::U64 };
            instrs.push(load_operand(RAX, src, ctx, Some(size)));
            (ElfOper::Reg(RAX), size as usize)
        }
        IrOp::Bool(_) => {
            instrs.push(load_operand(RAX, src, ctx, Some(Size::U8)));
            (ElfOper::Reg(RAX), 1)
        }
        _ => (convert_operand(src, ctx), src_size),
    };
    // cvtsi2sd and movsxd want 4 or 8 bytes: a 논리, 문자 or 바이트 is widened by movzx first
    if !src_float && src_size == 1 {
        instrs.push(extend(ExtendOp::Movzx, RAX, src_op, BYTE, QWORD));
        (src_op, src_size) = (ElfOper::Reg(RAX), 8);
    }
    if !src_float && dest_float && src_unsigned && src_size == 8 {
        instrs.extend(unsigned_64_to_float(dest, src_op, ctx));
        return instrs;
    }
    let (op, out, size) = match (src_float, dest_float) {
        (true, true) if dest_size == 8 => (ConvOp::Cvtss2sd, XMM0, None),
        (true, true) => (ConvOp::Cvtsd2ss, XMM0, None),
        (false, true) if dest_size == 8 => (ConvOp::Cvtsi2sd, XMM0, Some(Size::from(src_size))),
        (false, true) => (ConvOp::Cvtsi2ss, XMM0, Some(Size::from(src_size))),
        // cvt writes 4 or 8 bytes: a 1-byte 정수 is converted on 4, the mov below stores its low byte
        (true, false) if src_size == 8 => {
            (ConvOp::Cvttsd2si, RAX, Some(Size::from(dest_size.max(4))))
        }
        (true, false) => (ConvOp::Cvttss2si, RAX, Some(Size::from(dest_size.max(4)))),
        (false, false) => {
            instrs.extend(cast_between_two_non_float(
                dest,
                src_op,
                src_size,
                src_unsigned,
                ctx,
            ));
            return instrs;
        }
    };
    instrs.push(convert(op, out, src_op, size));
    instrs.push(bin(
        if dest_float {
            BinOp::StoreF
        } else {
            BinOp::Mov
        },
        convert_operand(&IrOp::Var(dest.clone()), ctx),
        reg(out),
        var_size(dest),
    ));
    instrs
}

/// `dest = src`, sized by `dest`: `mov rax, src ; mov [dest], rax`
pub fn generate_copy(dest: &VarId, src: &IrOp, ctx: &Context) -> Vec<Instruction> {
    let var = convert_operand(&IrOp::Var(dest.clone()), ctx);
    // x86 has no memory to memory mov: the value goes through rax
    vec![
        load_operand(RAX, src, ctx, var_size(dest)),
        bin(BinOp::Mov, var, reg(RAX), var_size(dest)),
    ]
}

/// Every byte of `dest` to 0: `lea rdi, [dest] ; mov rcx, size ; mov rax, 0 ; rep stosb`
pub fn generate_zero(dest: &VarId, ctx: &Context) -> Vec<Instruction> {
    let size = IrOp::Int(dest.ty.size() as i64, false, false);
    vec![
        Instruction::Lea {
            dst: RDI,
            src: convert_operand(&IrOp::Var(dest.clone()), ctx),
            size: None,
        },
        load_operand(RCX, &size, ctx, Some(Size::U64)),
        load_operand(RAX, &IrOp::Int(0, false, false), ctx, Some(Size::U64)),
        prefixed(vec![Prefix::Rep], string(StringOp::Stos, BYTE)),
    ]
}

/// `call symbol` (System V): the arguments in rdi, rsi, rdx, rcx, r8, r9, al = the number of
/// float arguments (read by variadic functions such as printf), then `mov [dest], rax` if kept
pub fn generate_call(
    dest: &Option<VarId>,
    symbol: &str,
    args: &[IrOp],
    ctx: &Context,
) -> Vec<Instruction> {
    let mut instr: Vec<Instruction> = vec![];
    let mut i = 0;
    let mut i_f = 0;

    for arg in args {
        let new_instr = match arg {
            IrOp::Float(_) => float_parameter(arg, &mut i_f, ctx),
            IrOp::Var(var) if var.ty.is_float() => float_parameter(arg, &mut i_f, ctx),
            _ => normal_param(arg, &mut i, ctx),
        };
        instr.push(new_instr);
    }
    instr.push(bin(BinOp::Mov, reg(RAX), imm(i64::from(i_f)), None));
    instr.push(call(Sym(symbol.to_string())));
    // the result comes back in rax
    if let Some(var) = dest {
        instr.push(bin(
            if var.ty.is_float() {
                BinOp::StoreF
            } else {
                BinOp::Mov
            },
            convert_operand(&IrOp::Var(var.clone()), ctx),
            reg(if var.ty.is_float() { XMM0 } else { RAX }),
            var_size(var),
        ));
    }
    instr
}

pub fn generate_jmp_if_not(cond: &IrOp, target: &Label, ctx: &Context) -> Vec<Instruction> {
    let instrs: Vec<Instruction> = vec![
        load_operand(RAX, cond, ctx, Some(Size::U8)),
        bin(BinOp::Cmp, reg(RAX), imm(0), BYTE),
        jcc(ConditionCode::E, ctx.labels[target.0]),
    ];

    instrs
}
