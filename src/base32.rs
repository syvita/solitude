//Copyright (c) 2015 The base32 Developers
//taken from `https://github.com/andreasots/base32/blob/master/src/lib.rs`.

const ALPHABET: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz234567";

pub fn encode(data: &[u8]) -> String {
	let mut retainer = Vec::with_capacity((data.len() + 3) / 4 * 5);

	for chunk in data.chunks(5) {
		let buffer = {
			let mut buffer = [0u8; 5];

			for (index, &byte) in chunk.iter().enumerate() {
				buffer[index] = byte;
			}

			buffer
		};

		retainer.push(ALPHABET[((buffer[0] & 0xF8) >> 3) as usize]);
		retainer.push(ALPHABET[(((buffer[0] & 0x07) << 2) | ((buffer[1] & 0xC0) >> 6)) as usize]);
		retainer.push(ALPHABET[((buffer[1] & 0x3E) >> 1) as usize]);
		retainer.push(ALPHABET[(((buffer[1] & 0x01) << 4) | ((buffer[2] & 0xF0) >> 4)) as usize]);
		retainer.push(ALPHABET[(((buffer[2] & 0x0F) << 1) | (buffer[3] >> 7)) as usize]);
		retainer.push(ALPHABET[((buffer[3] & 0x7C) >> 2) as usize]);
		retainer.push(ALPHABET[(((buffer[3] & 0x03) << 3) | ((buffer[4] & 0xE0) >> 5)) as usize]);
		retainer.push(ALPHABET[(buffer[4] & 0x1F) as usize]);
	}

	if data.len() % 5 != 0 {
		let length = retainer.len();
		let extra = 8 - (data.len() % 5 * 8 + 4) / 5;

		retainer.truncate(length - extra);
	}

	String::from_utf8(retainer).unwrap()
}
