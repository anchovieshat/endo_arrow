extern crate image;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::cmp;
use std::str;

use image::{ImageBuffer, Rgba, Pixel};

fn nat(bytes: &[u8], idx: usize) -> (usize, usize) {
    let tmp_idx;
    let n;
    match bytes[idx] as char {
        'P' => {
            n = 0;
            tmp_idx = idx + 1;
        },
        'I' | 'F' => {
            let (x, y) = nat(bytes, idx + 1);
            n = x * 2;
            tmp_idx = y;
        },
        'C' => {
            let (x, y) = nat(bytes, idx + 1);
            n = x * 2 + 1;
            tmp_idx = y;
        },
        _ => {
            unimplemented!();
        }
    }

    return (n, tmp_idx);
}

fn consts(bytes: &[u8], idx: usize) -> (String, usize) {
    let tmp_idx;
    let mut tmp_s;

    match bytes[idx] as char {
        'C' => {
            let (s, ret_idx) = consts(bytes, idx + 1);
            tmp_s = String::from("I");
            tmp_s.push_str(s.as_str());
            tmp_idx = ret_idx;
        },
        'F' => {
            let (s, ret_idx) = consts(bytes, idx + 1);
            tmp_s = String::from("C");
            tmp_s.push_str(s.as_str());
            tmp_idx = ret_idx;
        },
        'P' => {
            let (s, ret_idx) = consts(bytes, idx + 1);
            tmp_s = String::from("F");
            tmp_s.push_str(s.as_str());
            tmp_idx = ret_idx;
        },
        _ => {
            let tmp_str = format!("{}{}", bytes[idx] as char, bytes[idx + 1] as char);
            match tmp_str.as_str() {
                "IC" => {
                    let (s, ret_idx) = consts(bytes, idx + 2);
                    tmp_s = String::from("P");
                    tmp_s.push_str(s.as_str());
                    tmp_idx = ret_idx;
                },
                _ => {
                    return (String::new(), idx);
                }
            }
        }
    }

    return (tmp_s, tmp_idx);
}

fn pattern(dna: &[u8], rna: &mut Vec<u8>, idx: usize) -> (String, usize) {
    let mut p = String::new();
    let mut lvl = 0;
    let mut i = idx;

    loop {
        let c = dna[i] as char;
        match c {
            'C' => {
                i += 1;
                p.push('I');
            },
            'F' => {
                i += 1;
                p.push('C');
            },
            'P' => {
                i += 1;
                p.push('F');
            },
            _ => {
                let c2 = dna[i + 1] as char;
                let tmp = String::from(format!("{}{}", c, c2));
                let tmp_str = tmp.as_str();
                match tmp_str {
                    "IC" => {
                        i += 2;
                        p.push('P');
                    },
                    "IP" => {
                        let (n, idx) = nat(dna, i + 2);
                        p.push_str(format!("!{}", n).as_str());
                        i = idx;
                    },
                    "IF" => {
                        i += 3;
                        let (s, idx) = consts(dna, i);
                        p.push_str(format!("?{}", s).as_str());
                        i = idx;
                    },
                    _ => {
                        let c3 = dna[i + 2] as char;
                        let tmp2 = String::from(format!("{}{}{}", c, c2, c3));
                        let tmp2_str = tmp2.as_str();

                        match tmp2_str {
                            "IIP" => {
                                i += 3;
                                p.push('(');
                                lvl += 1;
                            },
                            "IIC" | "IIF" => {
                                i += 3;
                                if lvl == 0 {
                                    break;
                                } else {
                                    p.push(')');
                                    lvl -= 1;
                                }
                            },
                            "III" => {
                                rna.extend_from_slice(&dna[i+3..i+10]);
                                i += 10;
                            },
                            _ => {
                                unimplemented!()
                            }
                        }
                    }
                }
            }
        }
    }

    return (p, i);
}

fn template(dna: &[u8], rna: &mut Vec<u8>, idx: usize) -> (String, usize) {
    let mut t = String::new();
    let mut i = idx;
    loop {
        let c = dna[i] as char;
        match c {
            'C' => {
                i += 1;
                t.push('I');
            },
            'F' => {
                i += 1;
                t.push('C');
            },
            'P' => {
                i += 1;
                t.push('F');
            },
            _ => {
                let c2 = dna[i + 1] as char;
                let tmp = String::from(format!("{}{}", c, c2));
                let tmp_str = tmp.as_str();
                match tmp_str {
                    "IC" => {
                        i += 2;
                        t.push('P');
                    },
                    "IF" | "IP" => {
                        i += 2;
                        let (l, n1_idx) = nat(dna, i);
                        let (n, n2_idx) = nat(dna, n1_idx);
                        i = n2_idx;

                        if l == 0 {
                            t.push_str(format!("\\{}", n).as_str());
                        } else {
                            t.push_str(format!("\\{}({})", n, l).as_str());
                        }
                    },
                    _ => {
                        let c3 = dna[i + 2] as char;
                        let tmp2 = String::from(format!("{}{}{}", c, c2, c3));
                        let tmp2_str = tmp2.as_str();
                        match tmp2_str {
                            "IIC" | "IIF" => {
                                i += 3;
                                break;
                            },
                            "IIP" => {
                                i += 3;
                                let (n, n3_idx) = nat(dna, i);
                                t.push_str(format!("|{}|", n).as_str());

                                i = n3_idx;
                            },
                            "III" => {
                                rna.extend_from_slice(&dna[i+3..i+10]);
                                i += 10;
                            },
                            _ => {
                                unimplemented!()
                            }
                        }
                    }
                }
            }
        }
    }

    return (t, i);
}

type Pos = (u32, u32);
type RGB = (u8, u8, u8);
type Transparency = u8;

enum Color {
    RGB(RGB),
    A(Transparency)
}

type Bucket = Vec<Color>;
type Bitmap = ImageBuffer<Rgba<u8>, Vec<u8>>;

const BLACK: RGB = (0, 0, 0);
const RED: RGB = (255, 0, 0);
const GREEN: RGB = (0, 255, 0);
const YELLOW: RGB = (255, 255, 0);
const BLUE: RGB = (0, 0, 255);
const MAGENTA: RGB = (255, 0, 255);
const CYAN: RGB = (0, 255, 255);
const WHITE: RGB = (255, 255, 255);
const TRANSPARENT: Transparency = 0;
const OPAQUE: Transparency = 255;

enum Dir { North, East, South, West }

fn turn_clockwise(dir: Dir) -> Dir {
    return match dir {
        Dir::North => Dir::East,
        Dir::East => Dir::South,
        Dir::South => Dir::West,
        Dir::West => Dir::North,
    }
}

fn turn_counterclockwise(dir: Dir) -> Dir {
    return match dir {
        Dir::North => Dir::West,
        Dir::East => Dir::North,
        Dir::South => Dir::East,
        Dir::West => Dir::South,
    }
}

fn move_pos(pos: Pos, dir: &Dir) -> Pos {
    return match (pos, dir) {
        ((x, y), &Dir::North) => (x, (y + 600 - 1) % 600),
        ((x, y), &Dir::East) => ((x + 1) % 600, y),
        ((x, y), &Dir::South) => (x, (y + 1) % 600),
        ((x, y), &Dir::West) => ((x + 600 - 1) % 600, y),
    }
}

fn current_color(bucket: &Bucket) -> image::Rgba<u8> {
    let mut r_sum = 0;
    let mut g_sum = 0;
    let mut b_sum = 0;
    let mut a_sum = 0;
    let mut rgb_count = 0;
    let mut a_count = 0;
    for c in bucket {
        match c {
            &Color::RGB((r, g, b)) => {
                rgb_count += 1;
                r_sum += r as u32;
                g_sum += g as u32;
                b_sum += b as u32;
            },
            &Color::A(a) => {
                a_count += 1;
                a_sum += a as u32;
            }
        }
    }
    let rc = if rgb_count == 0 { 0 } else { r_sum / rgb_count };
    let gc = if rgb_count == 0 { 0 } else { g_sum / rgb_count };
    let bc = if rgb_count == 0 { 0 } else { b_sum / rgb_count };
    let ac = if a_count == 0 { 255 } else { a_sum / a_count };

    Rgba::from_channels((rc * ac / 255) as u8, (gc * ac / 255) as u8, (bc * ac / 255) as u8, ac as u8)
}

fn draw_line(image: &mut Bitmap, bucket: &Bucket, pos: &Pos, mark: &Pos) {
    let px = current_color(bucket);

    let &(x0, y0) = pos;
    let &(x1, y1) = mark;
    let dx = (x1 as i32) - (x0 as i32);
    let dy = (y1 as i32) - (y0 as i32);

    let d = cmp::max(dx.abs(), dy.abs());
    let c = if dx * dy <= 0 { 1 } else { 0 };

    let off = (d - c) / 2;

    let mut x = (x0 as i32) * d + off;
    let mut y = (y0 as i32) * d + off;
    for _ in 0..d {
        image.put_pixel((x / d) as u32, (y / d) as u32, px);
        x += dx;
        y += dy;
    }
    image.put_pixel(x1, y1, px);
}

fn new_bitmap() -> Bitmap {
    ImageBuffer::from_pixel(600, 600, Rgba::from_channels(0, 0, 0, 0))
}

fn compose(bitmaps: &mut Vec<Bitmap>) {
    if bitmaps.len() < 2 { return }

    for x in 0..600 {
        for y in 0..600 {
            let (r0, g0, b0, a0) = bitmaps[0].get_pixel(x, y).channels4();
            let (r1, g1, b1, a1) = bitmaps[1].get_pixel(x, y).channels4();
            bitmaps[1].put_pixel(x, y, Rgba::from_channels(
                r0 + (((r1 as u32) * ((255 - a0) as u32)) / 255) as u8,
                g0 + (((g1 as u32) * ((255 - a0) as u32)) / 255) as u8,
                b0 + (((b1 as u32) * ((255 - a0) as u32)) / 255) as u8,
                a0 + (((a1 as u32) * ((255 - a0) as u32)) / 255) as u8
            ))
        }
    }

    bitmaps.remove(0);
}

fn try_fill(bitmap: &mut Bitmap, cur_color: Rgba<u8>, pos: &Pos) {
    let &(x, y) = pos;
    let initial_color = *bitmap.get_pixel(x, y);

    if initial_color == cur_color {
        fill(bitmap, cur_color, pos, initial_color);
    }
}

fn fill(bitmap: &mut Bitmap, cur_color: Rgba<u8>, pos: &Pos, initial: Rgba<u8>) {
    let mut to_fill = vec![*pos];
    loop {
        match to_fill.pop() {
            None => return,
            Some((x, y)) => if *bitmap.get_pixel(x, y) == initial {
                bitmap.put_pixel(x, y, cur_color);
                if x > 0 { to_fill.push((x - 1, y)) }
                if x < 599 { to_fill.push((x + 1, y)) }
                if y > 0 { to_fill.push((x, y - 1)) }
                if y < 599 { to_fill.push((x, y + 1)) }
            }
        }
    }
}

fn clip(bitmaps: &mut Vec<Bitmap>) {
    if bitmaps.len() < 2 { return }

    for x in 0..600 {
        for y in 0..600 {
            let (_, _, _, a0) = bitmaps[0].get_pixel(x, y).channels4();
            let (r1, g1, b1, a1) = bitmaps[1].get_pixel(x, y).channels4();
            bitmaps[1].put_pixel(x, y, Rgba::from_channels(
                (((r1 as u32) * (a0 as u32)) / 255) as u8,
                (((g1 as u32) * (a0 as u32)) / 255) as u8,
                (((b1 as u32) * (a0 as u32)) / 255) as u8,
                (((a1 as u32) * (a0 as u32)) / 255) as u8
            ))
        }
    }

    bitmaps.remove(0);
}

fn build(rna: &[u8]) {
    let mut pos: Pos = (0, 0);
    let mut mark: Pos = (0, 0);
    let mut dir = Dir::East;
    let mut bucket: Bucket = Vec::new();

    let mut bitmaps = vec![new_bitmap()];
    let mut i = 0;

    while i < rna.len() {
        let bytes = str::from_utf8(&rna[i..i+7]).unwrap();

        match bytes {
            "PIPIIIC" => { bucket.push(Color::RGB(BLACK)); },
            "PIPIIIP" => { bucket.push(Color::RGB(RED)); },
            "PIPIICC" => { bucket.push(Color::RGB(GREEN)); },
            "PIPIICF" => { bucket.push(Color::RGB(YELLOW)); },
            "PIPIICP" => { bucket.push(Color::RGB(BLUE)); },
            "PIPIIFC" => { bucket.push(Color::RGB(MAGENTA)); },
            "PIPIIFF" => { bucket.push(Color::RGB(CYAN)); },
            "PIPIIPC" => { bucket.push(Color::RGB(WHITE)); },
            "PIPIIPF" => { bucket.push(Color::A(TRANSPARENT)); },
            "PIPIIPP" => { bucket.push(Color::A(OPAQUE)); },
            "PIIPICP" => { bucket.clear(); },
            "PIIIIIP" => { pos = move_pos(pos, &dir); },
            "PCCCCCP" => { dir = turn_counterclockwise(dir); },
            "PFFFFFP" => { dir = turn_clockwise(dir); },
            "PCCIFFP" => { mark = pos; },
            "PFFICCP" => { draw_line(&mut bitmaps[0], &bucket, &pos, &mark); },
            "PIIPIIP" => { try_fill(&mut bitmaps[0], current_color(&bucket), &pos); },
            "PCCPFFP" => { bitmaps.insert(0, new_bitmap())},
            "PFFPCCP" => { compose(&mut bitmaps); },
            "PFFICCF" => { clip(&mut bitmaps); },
            _ => { println!("{}", bytes); unimplemented!() }
        }

        i += 7;
    }

    let _ = bitmaps[0].save(&Path::new("dump.png")).unwrap();
}

fn main() {
    let mut f = File::open("endo.dna").unwrap();
    let mut dna_str = String::new();
    f.read_to_string(&mut dna_str).unwrap();

    let dna = dna_str.as_bytes();
    println!("dna length: {}", dna.len());

    let mut rna = Vec::new();

    let mut i = 0;
    let (p, p_idx) = pattern(dna, &mut rna, i);
    i = p_idx;
    let (t, t_idx) = template(dna, &mut rna, i);
    i = t_idx;

    println!("pattern: {}, template: {}", p, t);
    println!("rna length: {}", rna.len());

    build(rna.as_slice());
}
