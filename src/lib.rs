use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn emoji_filter(buffer: Vec<u8>, canvas_width: u32, canvas_height: u32, dot_size: u32, emoji_chk: bool) -> String {
    let width = canvas_width as usize;
    let height = canvas_height as usize;
    let dot_size = dot_size as usize;
    let mut new_buffer = buffer.clone(); // 元のバッファをコピーして変更を加える

    for i in 0..width * height {
        let index = i * 4; // RGBAなので4倍
        let r = new_buffer[index] as f32;
        let g = new_buffer[index + 1] as f32;
        let b = new_buffer[index + 2] as f32;
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let new_s = s * 2.0;
        let new_s = new_s.clamp(0.0, 1.0); // 彩度が1.0まで
        // RGBに変換
        let (new_r, new_g, new_b) = hsl_to_rgb(h, new_s, l);
        new_buffer[index] = new_r as u8;
        new_buffer[index + 1] = new_g as u8;
        new_buffer[index + 2] = new_b as u8;
    }

    for y in (0..height).step_by(dot_size) {
        for x in (0..width).step_by(dot_size) {
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;

            for dy in 0..dot_size {
                for dx in 0..dot_size {
                    let i = ((y + dy) * width + (x + dx)) * 4; // RGBAなので4倍
                    if i + 3 < new_buffer.len() {
                        r += new_buffer[i] as u32;
                        g += new_buffer[i + 1] as u32;
                        b += new_buffer[i + 2] as u32;
                    }
                }
            }
            // ドット内のすべてのピクセルに平均色を設定
            r /= (dot_size * dot_size) as u32;
            g /= (dot_size * dot_size) as u32;
            b /= (dot_size * dot_size) as u32;
            (r,g,b)=closest_color(r,g,b);
            for dy in 0..dot_size {
                for dx in 0..dot_size {
                    let i = ((y + dy) * width + (x + dx)) * 4;
                    if i + 3 < new_buffer.len() {
                        new_buffer[i] = r as u8;
                        new_buffer[i + 1] = g as u8;
                        new_buffer[i + 2] = b as u8;
                        new_buffer[i + 3] = 255;
                    }
                }
            }
        }
    }

    // 絵文字変換
    let mut result = String::new();
    for y in (0..height).step_by(dot_size) {
        for x in (0..width).step_by(dot_size) {
            let cell = extract_cell(&new_buffer, x, y, width, height, dot_size);
            let recognized_char = analyze_cell(&cell,emoji_chk);
            result.push(recognized_char);
        }
        result.push('\n');
    }
    result
}

fn extract_cell(buffer: &[u8], x: usize, y: usize, width: usize, height: usize, dot_size: usize) -> Vec<u8> {
    let mut cell = Vec::new();
    for dy in 0..dot_size {
        for dx in 0..dot_size {
            let px = x + dx;
            let py = y + dy;
            if px < width && py < height {
                let index = (py * width + px) * 4;
                if index + 4 <= buffer.len() {
                    cell.extend_from_slice(&buffer[index..index + 4]);
                } else {
                    // バッファの範囲外の場合は、透明ピクセルを追加
                    cell.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
    }
    cell
}

// Cellの色が近い下記の絵文字を返す
fn analyze_cell(cell: &[u8],emoji_chk:bool) -> char {
    let mut r_sum: u32 = 0;
    let mut g_sum: u32 = 0;
    let mut b_sum: u32 = 0;
    let mut count: u32 = 0;

    for i in (0..cell.len()).step_by(4) {
        r_sum += cell[i] as u32;
        g_sum += cell[i + 1] as u32;
        b_sum += cell[i + 2] as u32;
        count += 1;
    }

    if count == 0 {
        return '⬜'; // セルが空の場合は白を返す
    }

    let r_avg = r_sum / count;
    let g_avg = g_sum / count;
    let b_avg = b_sum / count;
    if emoji_chk{
        match closest_color(r_avg, g_avg, b_avg) {
            (255, 119, 99) => '🟥',   // 赤
            (255, 155, 59) => '🟧',   // オレンジ
            (243, 191, 63) => '🟨',   // 黄色
            (131, 211, 19) => '🟩',   // 緑
            (0, 235, 219) => '🟦',   // 青
            (63, 191, 255) => '🟪',   // 紫
            (134, 74, 43) => '🟫',   // 茶
            (0, 0, 0) => '⬛',         // 黒
            (255, 255, 255) => '⬜',   // 白
            _ => '⬜',                // デフォルトは白
        }
    }
    else{
        match closest_color(r_avg, g_avg, b_avg) {
            (255, 119, 99) => '😡',   // 赤
            (255, 155, 59) => '🍊',   // オレンジ
            (243, 191, 63) => '⭐',   // 黄色
            (131, 211, 19) => '🤢',   // 緑
            (0, 235, 219) => '🥶',   // 青
            (63, 191, 255) => '😈',   // 紫
            (134, 74, 43) => '💩',   // 茶
            (0, 0, 0) => '👾',         // 黒
            (255, 255, 255) => '👻',   // 白
            _ => '👻',                // デフォルトは白
        }
    }

}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let c = max - min;

    let mut h = 0.0;
    if c != 0.0 {
        if max == r {
            h = 60.0 * ((g - b) / c % 6.0);
        } else if max == g {
            h = 60.0 * ((b - r) / c + 2.0);
        } else if max == b {
            h = 60.0 * ((r - g) / c + 4.0);
        }
    }

    if h < 0.0 {
        h += 360.0;
    }

    let l = (max + min) / 2.0;

    let s = if c == 0.0 {
        0.0
    } else {
        c / (1.0 - (2.0 * l - 1.0).abs())
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h >= 0.0 && h < 60.0 {
        (c, x, 0.0)
    } else if h >= 60.0 && h < 120.0 {
        (x, c, 0.0)
    } else if h >= 120.0 && h < 180.0 {
        (0.0, c, x)
    } else if h >= 180.0 && h < 240.0 {
        (0.0, x, c)
    } else if h >= 240.0 && h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    ((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0)
}

fn closest_color(r: u32, g: u32, b: u32) -> (u32, u32, u32) {
    let colors = [
        (255, 119, 99) ,   // 赤
        (255, 155, 59) ,   // オレンジ
        (243, 191, 63) ,   // 黄色
        (131, 211, 19) ,   // 緑
        (0, 235, 219) ,   // 青
        (63, 191, 255),   // 紫
        (134, 74, 43) ,   // 茶
        (0, 0, 0) ,         // 黒
        (255, 255, 255) ,   // 白
    ];
    *colors
        .iter()
        .min_by_key(|&&(cr, cg, cb)| color_distance(r, g, b, cr, cg, cb))
        .unwrap()
}

fn color_distance(r1: u32, g1: u32, b1: u32, r2: u32, g2: u32, b2: u32) -> u32 {
    let r_diff = r1 as i32 - r2 as i32;
    let g_diff = g1 as i32 - g2 as i32;
    let b_diff = b1 as i32 - b2 as i32;
    // 3次元空間上の距離を計算
    (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff) as u32
}