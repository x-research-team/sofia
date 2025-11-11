use crate::bytecode::instructions::Instructions;

/// Дизассемблирует последовательность инструкций в читаемую строку.
///
/// Возвращает строку, где каждая строка представляет собой одну инструкцию
/// с её смещением, мнемоникой и операндами.
pub fn disassemble(instructions: &Instructions) -> String {
    let mut output = String::new();
    let mut i = 0;

    output.push_str("=== BYTECODE DISASSEMBLY ===\n\n");

    // Выводим константы, если они есть
    if !instructions.constants.is_empty() {
        output.push_str("=== CONSTANTS POOL ===\n");
        for (idx, constant) in instructions.constants.iter().enumerate() {
            output.push_str(&format!("[{}] {}\n", idx, constant));
        }
        output.push_str("\n");
    }

    output.push_str("=== INSTRUCTIONS ===\n");

    while i < instructions.bytes.len() {
        let op = Instructions::read_opcode(&instructions.bytes, i);
        if let Some(opcode) = op {
            let (operands, read) = Instructions::read_operands(opcode, &instructions.bytes, i + 1);

            output.push_str(&format!("{:04} {}", i, opcode.mnemonic()));

            // Выводим операнды
            if !operands.is_empty() {
                for operand in &operands {
                    output.push_str(&format!(" {}", operand));
                }
            }
            output.push('\n');

            i += 1 + read; // Смещение + байт опкода + байты операндов
        } else {
            output.push_str(&format!(
                "{:04} UNKNOWN_OPCODE ({})\n",
                i, instructions.bytes[i]
            ));
            i += 1;
        }
    }
    output
}
