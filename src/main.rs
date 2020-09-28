fn main() {
    let mut p = 0x200; let mut m = [0; 0x2ff]; let mut r = [0; 2]; let mut i = 0;
    let mut f = [false; 64 * 32]; use std::io::Read;
    for (a, v) in std::io::stdin().bytes().enumerate() { m[a + p] = v.unwrap() as _; }
    loop { let n = m[p] << 8 | m[p + 1]; let k = n & 0x00ff; let z = (n >> 8) & 0xf;
        match n >> 12 { 0x6 => r[z] = k, 0x7 => r[z] = r[z] + k, 0xa => i = n & 0x0fff, _ => {
        for y in 0..(n & 0x000f) { for x in 0..8 { if (m[i + y] & (0x80 >> x)) != 0 {
            f[(r[(n >> 4) & 0xf] + y) * 64 + (r[z] + x)] = true; } } } } };
        p += 2; if p == 0x228 { break; } }
    print!("P1\n{} {}", 64, 32); i = 0; for p in f.iter() {
        if i % 64 == 0 { println!(); } if *p { print!("1 ") } else { print!("0 ") } i += 1; }
    println!();
}
