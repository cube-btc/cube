use super::prefix::Prefix;

type Bytes = Vec<u8>;

pub enum CSVFlag {
    CSVBlock,
    CSVHour,
    CSVDay,
    CSVWeek,
    CSVTwoWeeks,
    CSVMonth,
    CSVTwoMonths,
    CSVThreeMonths,
    CSVSixMonths,
    CSVYear,
    Days(u8),
}

pub trait CSVEncode {
    fn n_sequence(flag: CSVFlag) -> Bytes;
    fn csv_num_encode(flag: CSVFlag) -> Bytes;
}

impl CSVEncode for Bytes {
    fn n_sequence(flag: CSVFlag) -> Bytes {
        let mut encoded = Vec::<u8>::new();

        match flag {
            CSVFlag::CSVBlock => encoded.extend(vec![0x01, 0x00, 0x00, 0x00]),
            CSVFlag::CSVHour => encoded.extend(vec![0x06, 0x00, 0x00, 0x00]),
            CSVFlag::CSVDay => encoded.extend(vec![0x90, 0x00, 0x00, 0x00]),
            CSVFlag::CSVWeek => encoded.extend(vec![0xf0, 0x03, 0x00, 0x00]),
            CSVFlag::CSVTwoWeeks => encoded.extend(vec![0xe0, 0x07, 0x00, 0x00]),
            CSVFlag::CSVMonth => encoded.extend(vec![0xe0, 0x10, 0x00, 0x00]),
            CSVFlag::CSVTwoMonths => encoded.extend(vec![0xc0, 0x21, 0x00, 0x00]),
            CSVFlag::CSVThreeMonths => encoded.extend(vec![0xa0, 0x32, 0x00, 0x00]),
            CSVFlag::CSVSixMonths => encoded.extend(vec![0x40, 0x65, 0x00, 0x00]),
            CSVFlag::CSVYear => encoded.extend(vec![0x50, 0xcd, 0x00, 0x00]),
            CSVFlag::Days(days) => encoded.extend(pad_four(days_to_bytes(days, false))),
        }

        encoded
    }

    fn csv_num_encode(flag: CSVFlag) -> Bytes {
        let mut encoded = Vec::<u8>::new();

        match flag {
            CSVFlag::CSVBlock => encoded.extend(vec![0x51]),
            CSVFlag::CSVHour => encoded.extend(vec![0x56]),
            CSVFlag::CSVDay => encoded.extend(vec![0x02, 0x90, 0x00]),
            CSVFlag::CSVWeek => encoded.extend(vec![0x02, 0xf0, 0x03]),
            CSVFlag::CSVTwoWeeks => encoded.extend(vec![0x02, 0xe0, 0x07]),
            CSVFlag::CSVMonth => encoded.extend(vec![0x02, 0xe0, 0x10]),
            CSVFlag::CSVTwoMonths => encoded.extend(vec![0x02, 0xc0, 0x21]),
            CSVFlag::CSVThreeMonths => encoded.extend(vec![0x02, 0xa0, 0x32]),
            CSVFlag::CSVSixMonths => encoded.extend(vec![0x02, 0x40, 0x65]),
            CSVFlag::CSVYear => encoded.extend(vec![0x03, 0x50, 0xcd, 0x00]),
            CSVFlag::Days(days) => encoded.extend(&days_to_bytes(days, true).prefix_pushdata()),
        }

        encoded
    }
}

fn days_to_bytes(days: u8, cscript_num: bool) -> Bytes {
    let blocks: u16 = days as u16 * 144;
    let mut vec = Vec::<u8>::new();

    if blocks <= 255 {
        // Single-byte
        vec.push(blocks as u8);
        if cscript_num == true && blocks > 127 {
            // CScriptNum
            vec.push(0x00);
        }
    } else {
        // Two-bytes
        vec.extend(vec![(blocks & 0xFF) as u8, (blocks >> 8 & 0xFF) as u8]);
        if cscript_num == true && blocks > 32767 {
            // CScriptNum
            vec.push(0x00);
        }
    }

    vec
}

fn pad_four(input: Bytes) -> Bytes {
    let input_len = input.len();
    let mut padded = input;

    match input_len {
        0 => padded.extend(vec![0x00, 0x00, 0x00, 0x00]),
        1 => padded.extend(vec![0x00, 0x00, 0x00]),
        2 => padded.extend(vec![0x00, 0x00]),
        3 => padded.extend(vec![0x00]),
        4 => (),
        _ => panic!(),
    }

    padded
}