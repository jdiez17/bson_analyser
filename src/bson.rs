use std::collections::HashMap;
use std::str;
use byteorder::{ByteOrder, LittleEndian};

type SizeMap<'a> = HashMap<Vec<&'a str>, usize>;

fn get_cstring(bytes: &[u8]) -> &str {
    let str_len = bytes.iter().position(|&c| c == 0x00).unwrap();
    str::from_utf8(&bytes[1..str_len]).unwrap()
}

fn get_string(bytes: &[u8]) -> &str {
    let str_len = (LittleEndian::read_u32(&bytes) - 1) as usize;
    str::from_utf8(&bytes[4..4+str_len]).unwrap()
}

fn element_size<'a>(path: Vec<&'a str>, bytes: &'a [u8]) -> (SizeMap<'a>, usize) {
    let el_name = get_cstring(&bytes);
    let el_type = bytes[0];

    // TODO this does not need to be mut
    let mut el_path = path.clone();
    el_path.push(el_name);

    let mut res = SizeMap::new();
    let mut i = 1 /* el_type */ + el_name.len() + 1 /* cstring null */;
    i += match el_type {
        // double
        0x01 => 8,
        // string
        0x02 => {
            4 + get_string(&bytes[i..]).len() + 1
        },
        // embedded document
        0x03 => {
            let (sz, incr) = document_size(el_path.clone(), &bytes[i..]);
            res.extend(sz.into_iter());
            incr
        },
        // array
        0x04 => {
            let (sz, incr) = document_size(el_path.clone(), &bytes[i..]);
            res.extend(sz.into_iter());
            incr
        },
        // binary
        0x05 => {
            4 + LittleEndian::read_u32(&bytes[i..]) as usize + 1 /* subtype */ + 1
        },
        // undefined (?)
        0x06 => 0,
        // ObjectId
        0x07 => 12,
        // bool
        0x08 => 1,
        // utc datetime
        0x09 => 4,
        // null
        0x0a => 0,
        // regex
        0x0b => {
            let pattern_sz = get_cstring(&bytes[i..]).len() + 1;
            let options_sz = get_cstring(&bytes[i+pattern_sz..]).len() + 1;

            pattern_sz + options_sz
        },
        // DBPointer
        0x0c => {
            4 + get_string(&bytes[i..]).len() + 1 + 12
        },
        // js code
        0x0d => {
            4 + get_string(&bytes[i..]).len() + 1
        },
        // symbol
        0x0e => {
            4 + get_string(&bytes[i..]).len() + 1
        },
        // js code with scope
        0x0f => {
            panic!("TODO");
        },
        // 32bit int
        0x10 => 4,
        // timestamp
        0x11 => 8,
        // 64bit int
        0x12 => 8,
        // 128bit decimal fp
        0x13 => 16,

        _ => panic!("Unknown element type {}", el_type)
    };

    res.insert(el_path, i);

    (res, i)
}

fn document_size<'a>(path: Vec<&'a str>, bytes: &'a [u8]) -> (SizeMap<'a>, usize) {
    let doc_size = LittleEndian::read_u32(&bytes) as usize;
    let el_path = path.clone();

    let mut res = SizeMap::new();
    let mut i = 4;
    while bytes[i] != 0x00 {
        let (sz, incr) = element_size(el_path.clone(), &bytes[i..]);
        res.extend(sz.into_iter());

        i += incr;
    }
    i += 1;

    assert!(doc_size == i);
    res.insert(el_path, i);

    (res, i)
}

pub fn bson_size(bytes: &[u8]) -> SizeMap {
    let (sz, _) = document_size(vec!["root"], &bytes[..]);
    sz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sizes() {
        let doc = vec![
            0x21, 0x00, 0x00, 0x00,             // Document size
            0x08, 0x66, 0x6f, 0x6f, 0x00, 0x01, // "foo" => true
            0x08, 0x62, 0x61, 0x72, 0x00, 0x00, // "bar" => false
            0x03, 0x62, 0x61, 0x7a, 0x00,       // "baz" => {
            0x0b, 0x00, 0x00, 0x00,
            0x08, 0x71, 0x75, 0x78, 0x00, 0x01, //   "qux" => true }
            0x00,
            0x00                                // Document end
        ];
        let mut expct = SizeMap::new();
        expct.insert(vec!["root"], 0x21);
        expct.insert(vec!["root", "foo"], 0x06);
        expct.insert(vec!["root", "bar"], 0x06);
        expct.insert(vec!["root", "baz"], 0x10);
        expct.insert(vec!["root", "baz", "qux"], 0x06);

        assert_eq!(expct, bson_size(&doc[..]));
    }
}
