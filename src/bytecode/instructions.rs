use crate::bytecode::opcode::Opcode;
use crate::object::Object;

/// Представляет последовательность байткода, состоящую из опкодов, операндов и пула констант.
#[derive(Debug, PartialEq, Clone)]
pub struct Instructions {
    /// Вектор байтов, содержащий инструкции.
    pub bytes: Vec<u8>,
    /// Пул констант для оптимизации хранения литеральных значений.
    pub constants: Vec<Object>,
}

impl Instructions {
    /// Создает новый пустой набор инструкций.
    pub fn new() -> Self {
        Instructions {
            bytes: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Добавляет другие инструкции к текущим.
    pub fn append(&mut self, other: &Instructions) {
        self.bytes.extend_from_slice(&other.bytes);
        // Примечание: константы из other не добавляются,
        // так как это должно быть обработано компилятором.
    }

    /// Добавляет константу в пул и возвращает её индекс.
    pub fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    /// Эмитирует опкод и его операнды, добавляя их в список инструкций.
    /// Возвращает смещение, с которого начинается добавленный опкод.
    pub fn emit(&mut self, op: Opcode, operands: &[u16]) -> usize {
        let pos = self.bytes.len();
        self.bytes.push(op as u8); // Добавляем байт опкода

        for (i, &operand) in operands.iter().enumerate() {
            let width = op.operand_widths()[i];
            match width {
                1 => self.bytes.push(operand as u8),
                2 => self.bytes.extend_from_slice(&operand.to_be_bytes()),
                _ => panic!("Неподдерживаемая ширина операнда: {}", width),
            }
        }
        pos
    }

    /// Читает опкод из байтов по заданному смещению.
    pub fn read_opcode(bytes: &[u8], offset: usize) -> Option<Opcode> {
        if offset >= bytes.len() {
            return None;
        }
        Opcode::from_byte(bytes[offset])
    }

    /// Читает операнды для данного опкода из байтов по заданному смещению.
    /// Возвращает вектор операндов и количество прочитанных байтов.
    pub fn read_operands(op: Opcode, bytes: &[u8], offset: usize) -> (Vec<u16>, usize) {
        let mut operands = Vec::new();
        let mut bytes_read = 0;

        for &width in op.operand_widths().iter() {
            match width {
                1 => {
                    if offset + bytes_read >= bytes.len() {
                        break; // Недостаточно байтов для чтения операнда
                    }
                    operands.push(bytes[offset + bytes_read] as u16);
                    bytes_read += 1;
                }
                2 => {
                    if offset + bytes_read + 1 >= bytes.len() {
                        break; // Недостаточно байтов для чтения операнда
                    }
                    let operand_bytes: [u8; 2] =
                        [bytes[offset + bytes_read], bytes[offset + bytes_read + 1]];
                    operands.push(u16::from_be_bytes(operand_bytes));
                    bytes_read += 2;
                }
                _ => panic!("Неподдерживаемая ширина операнда: {}", width),
            }
        }
        (operands, bytes_read)
    }

    /// Получить константу по индексу.
    pub fn get_constant(&self, index: usize) -> Option<&Object> {
        self.constants.get(index)
    }

    /// Получить все константы.
    pub fn get_constants(&self) -> &[Object] {
        &self.constants
    }
}

impl Default for Instructions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instructions_new() {
        let instr = Instructions::new();
        assert_eq!(instr.bytes.len(), 0);
        assert_eq!(instr.constants.len(), 0);
    }

    #[test]
    fn test_add_constant() {
        let mut instr = Instructions::new();

        // Добавляем несколько констант
        let idx1 = instr.add_constant(Object::Integer(42));
        let idx2 = instr.add_constant(Object::String("hello".to_string()));
        let idx3 = instr.add_constant(Object::Integer(100));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(instr.constants.len(), 3);

        // Проверяем что константы правильно сохранены
        assert_eq!(instr.get_constant(0), Some(&Object::Integer(42)));
        assert_eq!(
            instr.get_constant(1),
            Some(&Object::String("hello".to_string()))
        );
        assert_eq!(instr.get_constant(2), Some(&Object::Integer(100)));
        assert_eq!(instr.get_constant(3), None);
    }

    #[test]
    fn test_emit_no_operands() {
        let mut instr = Instructions::new();

        // Эмитируем опкоды без операндов
        instr.emit(Opcode::Add, &[]);
        instr.emit(Opcode::Sub, &[]);
        instr.emit(Opcode::Pop, &[]);

        assert_eq!(instr.bytes.len(), 3);
        assert_eq!(instr.bytes[0], Opcode::Add as u8);
        assert_eq!(instr.bytes[1], Opcode::Sub as u8);
        assert_eq!(instr.bytes[2], Opcode::Pop as u8);
    }

    #[test]
    fn test_emit_with_operands() {
        let mut instr = Instructions::new();

        // Эмитируем опкоды с 2-байтовыми операндами
        let pos1 = instr.emit(Opcode::Constant, &[255]);
        let pos2 = instr.emit(Opcode::Jump, &[512]);

        assert_eq!(pos1, 0);
        assert_eq!(pos2, 3); // 1 byte opcode + 2 bytes operand

        // Проверяем байты
        assert_eq!(instr.bytes[0], Opcode::Constant as u8);
        assert_eq!(instr.bytes[1], 0); // Первый байт 255 в big-endian
        assert_eq!(instr.bytes[2], 255); // Второй байт 255

        assert_eq!(instr.bytes[3], Opcode::Jump as u8);
        assert_eq!(instr.bytes[4], 2); // Первый байт 512 в big-endian (512 = 0x0200)
        assert_eq!(instr.bytes[5], 0); // Второй байт 512
    }

    #[test]
    fn test_emit_with_1byte_operand() {
        let mut instr = Instructions::new();

        let pos = instr.emit(Opcode::GetLocal, &[5]);

        assert_eq!(pos, 0);
        assert_eq!(instr.bytes.len(), 2); // 1 byte opcode + 1 byte operand
        assert_eq!(instr.bytes[0], Opcode::GetLocal as u8);
        assert_eq!(instr.bytes[1], 5);
    }

    #[test]
    fn test_read_opcode() {
        let mut instr = Instructions::new();
        instr.emit(Opcode::Add, &[]);
        instr.emit(Opcode::Constant, &[10]);

        assert_eq!(
            Instructions::read_opcode(&instr.bytes, 0),
            Some(Opcode::Add)
        );
        assert_eq!(
            Instructions::read_opcode(&instr.bytes, 1),
            Some(Opcode::Constant)
        );
        assert_eq!(Instructions::read_opcode(&instr.bytes, 10), None);
    }

    #[test]
    fn test_read_operands() {
        let mut instr = Instructions::new();
        instr.emit(Opcode::Constant, &[255]);

        // Читаем операнды для Constant (должны быть 2 байта)
        let (operands, bytes_read) = Instructions::read_operands(Opcode::Constant, &instr.bytes, 1);

        assert_eq!(bytes_read, 2);
        assert_eq!(operands.len(), 1);
        assert_eq!(operands[0], 255);
    }

    #[test]
    fn test_read_operands_1byte() {
        let mut instr = Instructions::new();
        instr.emit(Opcode::GetLocal, &[7]);

        let (operands, bytes_read) = Instructions::read_operands(Opcode::GetLocal, &instr.bytes, 1);

        assert_eq!(bytes_read, 1);
        assert_eq!(operands.len(), 1);
        assert_eq!(operands[0], 7);
    }

    #[test]
    fn test_big_endian_encoding() {
        let mut instr = Instructions::new();

        // Проверяем big-endian кодирование больших чисел
        instr.emit(Opcode::Jump, &[1024]); // 0x0400

        assert_eq!(instr.bytes[1], 4); // Старший байт
        assert_eq!(instr.bytes[2], 0); // Младший байт

        let (operands, _) = Instructions::read_operands(Opcode::Jump, &instr.bytes, 1);
        assert_eq!(operands[0], 1024);
    }

    #[test]
    fn test_append() {
        let mut instr1 = Instructions::new();
        instr1.emit(Opcode::Add, &[]);
        instr1.emit(Opcode::Sub, &[]);

        let mut instr2 = Instructions::new();
        instr2.emit(Opcode::Mul, &[]);

        instr1.append(&instr2);

        assert_eq!(instr1.bytes.len(), 3);
        assert_eq!(instr1.bytes[0], Opcode::Add as u8);
        assert_eq!(instr1.bytes[1], Opcode::Sub as u8);
        assert_eq!(instr1.bytes[2], Opcode::Mul as u8);
    }

    #[test]
    fn test_get_constants() {
        let mut instr = Instructions::new();
        instr.add_constant(Object::Integer(1));
        instr.add_constant(Object::Integer(2));
        instr.add_constant(Object::Integer(3));

        let constants = instr.get_constants();
        assert_eq!(constants.len(), 3);
        assert_eq!(constants[0], Object::Integer(1));
        assert_eq!(constants[1], Object::Integer(2));
        assert_eq!(constants[2], Object::Integer(3));
    }

    #[test]
    fn test_default() {
        let instr = Instructions::default();
        assert_eq!(instr.bytes.len(), 0);
        assert_eq!(instr.constants.len(), 0);
    }
}
