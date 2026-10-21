pub(crate) fn hex_char_for(number: u32) -> char {
    debug_assert!(number < 0x10);
    std::char::from_u32(if number < 0xA {
        0x30 + number
    } else {
        0x61 - 0xA + number
    })
    .unwrap()
}

pub(crate) fn is_name(c: char) -> bool {
    is_name_start(c) || c.is_ascii_digit() || c == '-'
}

pub(crate) fn is_name_start(c: char) -> bool {
    c == '_' || c.is_alphabetic() || c as u32 >= 0x0080
}

pub(crate) fn as_hex(c: char) -> u32 {
    match c {
        '0'..='9' => c as u32 - '0' as u32,
        'A'..='F' => 10 + c as u32 - 'A' as u32,
        'a'..='f' => 10 + c as u32 - 'a' as u32,
        _ => unreachable!(),
    }
}
