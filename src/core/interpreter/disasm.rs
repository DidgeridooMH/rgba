pub fn print_offset_as_immediate(offset: i32) -> String {
    if offset >= 0 {
        format!("#0x{:X}", offset)
    } else {
        format!("#-0x{:X}", -offset)
    }
}
