use super::generation::Context;
use super::instructions::{generate_cast, generate_zero};
use crate::converter::ir::structs::{Operand as IrOp, VarId};
use crate::elf::instructions::register::{RAX, RCX, RDI};
use crate::elf::instructions::{
    BinOp, ExtendOp, Instruction, Operand as ElfOper, Prefix, Size, StringOp,
};
use crate::parser::postparser::structs::Type;

/// No program can declare this type before #20: `부호 없는 짧은 정수` to `정수`, extended with zeros
#[test]
fn four_byte_unsigned_is_zero_extended() {
    let ctx = Context {
        return_type: None,
        offsets: vec![8, 16],
        data_names: &[],
        float_names: &[],
        floats: &[],
        labels: vec![],
    };
    let unsigned = Type::Int {
        short: true,
        unsigned: true,
    };
    let long = Type::Int {
        short: false,
        unsigned: false,
    };
    let src = IrOp::Var(VarId::new(1, unsigned));
    let instrs = generate_cast(&VarId::new(0, long), &src, &ctx);
    let signed = instrs.iter().any(|instr| {
        matches!(
            instr,
            Instruction::Extend {
                op: ExtendOp::Movsxd,
                ..
            }
        )
    });
    assert!(!signed, "movsxd extends the sign: {instrs:?}");
}

/// The whole array is cleared, from its lowest address: rcx holds its size in bytes, not its length
#[test]
fn zero_of_an_array_covers_all_its_bytes() {
    let ctx = Context {
        return_type: None,
        offsets: vec![24],
        data_names: &[],
        float_names: &[],
        floats: &[],
        labels: vec![],
    };
    let element = Type::Int {
        short: false,
        unsigned: false,
    };
    let dest = VarId::new(0, Type::Array(Box::new(element), 3));
    let instrs = generate_zero(&dest, &ctx);

    assert!(
        matches!(&instrs[0], Instruction::Lea { dst, src: ElfOper::MemoryAddress(_), .. } if *dst == RDI),
        "{instrs:?}"
    );
    assert!(
        matches!(&instrs[1], Instruction::Binary { op: BinOp::Mov, dst: ElfOper::Reg(r), src: ElfOper::Imm(24), .. } if *r == RCX),
        "{instrs:?}"
    );
    assert!(
        matches!(&instrs[2], Instruction::Binary { op: BinOp::Mov, dst: ElfOper::Reg(r), src: ElfOper::Imm(0), .. } if *r == RAX),
        "{instrs:?}"
    );
    let Instruction::Prefix { prefix, ins } = &instrs[3] else {
        panic!("{instrs:?}")
    };
    assert!(matches!(prefix[..], [Prefix::Rep]));
    assert!(matches!(
        **ins,
        Instruction::String {
            op: StringOp::Stos,
            size: Some(Size::U8)
        }
    ));
}
