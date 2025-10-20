/// Minimal base64 decoder that ignores ASCII whitespace. Returns raw bytes.
/// This avoids adding external dependencies to the core crate.
pub fn decode_base64_lossy(s: &str) -> Vec<u8> {
    // Mapping table: 255 = invalid, 254 = padding '='
    let mut map = [255u8; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        map[*c as usize] = i as u8;
    }
    map[b'=' as usize] = 254;

    // Collect values, ignoring whitespace
    let mut vals: Vec<u8> = Vec::new();
    for b in s.bytes() {
        if b.is_ascii_whitespace() { continue; }
        let m = map[b as usize];
        if m == 255 { continue; } // Ignore invalid bytes
        vals.push(m);
    }

    let mut out: Vec<u8> = Vec::with_capacity(vals.len() * 3 / 4 + 3);
    let mut i = 0usize;
    while i + 3 < vals.len() {
        let a = vals[i];
        let b = vals[i+1];
        let c = vals[i+2];
        let d = vals[i+3];
        i += 4;

        if a == 254 || b == 254 { break; }
        
        let x = ((a as u32) << 18) | ((b as u32) << 12) |
                (if c == 254 { 0 } else { (c as u32) << 6 }) |
                (if d == 254 { 0 } else { d as u32 });

        out.push(((x >> 16) & 0xFF) as u8);
        if c != 254 { out.push(((x >> 8) & 0xFF) as u8); }
        if d != 254 { out.push((x & 0xFF) as u8); }

        if c == 254 || d == 254 { break; }
    }

    out
}