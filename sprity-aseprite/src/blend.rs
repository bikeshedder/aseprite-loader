use crate::binary::blend_mode::BlendMode;

// Rust port of Aseprite's blend functions
// https://github.com/aseprite/aseprite/blob/master/src/doc/blend_funcs.cpp
// original implementation: https://github.com/alpine-alpaca/asefile/blob/main/src/blend.rs

pub(crate) type Color = [u8; 4];
pub(crate) trait Channels {
    fn r(&self) -> u8;
    fn g(&self) -> u8;
    fn b(&self) -> u8;
    fn a(&self) -> u8;
    fn r_i32(&self) -> i32;
    fn g_i32(&self) -> i32;
    fn b_i32(&self) -> i32;
    fn a_i32(&self) -> i32;
    fn r_f64(&self) -> f64;
    fn g_f64(&self) -> f64;
    fn b_f64(&self) -> f64;
    fn a_f64(&self) -> f64;
}

impl Channels for Color {
    fn r(&self) -> u8 {
        self[0]
    }
    fn g(&self) -> u8 {
        self[1]
    }
    fn b(&self) -> u8 {
        self[2]
    }
    fn a(&self) -> u8 {
        self[3]
    }

    fn r_i32(&self) -> i32 {
        self.r() as i32
    }

    fn g_i32(&self) -> i32 {
        self.g() as i32
    }

    fn b_i32(&self) -> i32 {
        self.b() as i32
    }

    fn a_i32(&self) -> i32 {
        self.a() as i32
    }

    fn r_f64(&self) -> f64 {
        self.a() as f64 / 255.
    }

    fn g_f64(&self) -> f64 {
        self.g() as f64 / 255.
    }

    fn b_f64(&self) -> f64 {
        self.b() as f64 / 255.
    }

    fn a_f64(&self) -> f64 {
        self.a() as f64 / 255.
    }
}

pub(crate) trait FromSlice {
    fn from_slice_u8(slice: &[u8]) -> Self;
    fn from_slice_i32(slice: &[i32]) -> Self;
    fn from_slice_f64(slice: &[f64]) -> Self;
}

impl FromSlice for Color {
    fn from_slice_u8(slice: &[u8]) -> Color {
        slice
            .try_into()
            .expect("slice too small to represent 32 bit color")
    }

    fn from_slice_i32(slice: &[i32]) -> Self {
        debug_assert!((0..=255).contains(&slice[0]));
        debug_assert!((0..=255).contains(&slice[1]));
        debug_assert!((0..=255).contains(&slice[2]));
        debug_assert!((0..=255).contains(&slice[3]));

        [
            slice[0] as u8,
            slice[1] as u8,
            slice[2] as u8,
            slice[3] as u8,
        ]
    }

    fn from_slice_f64(slice: &[f64]) -> Self {
        Color::from_slice_i32(&[
            (slice[0] * 255.0) as i32,
            (slice[1] * 255.0) as i32,
            (slice[2] * 255.0) as i32,
            (slice[3] * 255.0) as i32,
        ])
    }
}

type BlendFn = Box<dyn Fn(Color, Color, u8) -> Color>;
pub(crate) fn blend_mode_to_blend_fn(mode: BlendMode) -> BlendFn {
    match mode {
        BlendMode::Normal => Box::new(normal),
        BlendMode::Multiply => Box::new(multiply),
        BlendMode::Screen => Box::new(screen),
        BlendMode::Overlay => Box::new(overlay),
        BlendMode::Darken => Box::new(darken),
        BlendMode::Lighten => Box::new(lighten),
        BlendMode::ColorDodge => Box::new(color_dodge),
        BlendMode::ColorBurn => Box::new(color_burn),
        BlendMode::HardLight => Box::new(hard_light),
        BlendMode::SoftLight => Box::new(soft_light),
        BlendMode::Difference => Box::new(difference),
        BlendMode::Exclusion => Box::new(exclusion),
        BlendMode::Hue => Box::new(hsl_hue),
        BlendMode::Saturation => Box::new(hsl_saturation),
        BlendMode::Color => Box::new(hsl_color),
        BlendMode::Luminosity => Box::new(hsl_luminosity),
        BlendMode::Addition => Box::new(addition),
        BlendMode::Subtract => Box::new(subtract),
        BlendMode::Divide => Box::new(divide),
        m => panic!("unimplemented blend mode: {:?}", m),
    }
}

// --- addition ----------------------------------------------------------------
fn addition(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, addition_baseline)
}
fn addition_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let r = back.r_i32() + front.r_i32();
    let g = back.g_i32() + front.g_i32();
    let b = back.b_i32() + front.b_i32();

    let front = Color::from_slice_i32(&[r.min(255), g.min(255), b.min(255), front.a_i32()]);
    normal(back, front, opacity)
}

// --- subtract ----------------------------------------------------------------
fn subtract(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, subtract_baseline)
}

fn subtract_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let r = back.r_i32() - front.r_i32();
    let g = back.g_i32() - front.g_i32();
    let b = back.b_i32() - front.b_i32();

    let front = Color::from_slice_i32(&[r.max(0), g.max(0), b.max(0), front.a_i32()]);
    normal(back, front, opacity)
}

// --- hsl_hue -----------------------------------------------------------------
fn hsl_hue(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, hsl_hue_baseline)
}

fn hsl_hue_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let sat = saturation(back.r_f64(), back.g_f64(), back.b_f64());
    let lum = luminosity(back.r_f64(), back.g_f64(), back.b_f64());

    let (r, g, b) = set_saturation(front.r_f64(), front.g_f64(), front.b_f64(), sat);
    let (r, g, b) = set_luminocity(r, g, b, lum);

    let front = Color::from_slice_f64(&[r, g, b, front.a_f64()]);
    normal(back, front, opacity)
}

// --- hsl_saturation ----------------------------------------------------------
fn hsl_saturation(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, hsl_saturation_baseline)
}

fn hsl_saturation_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let sat = saturation(front.r_f64(), front.g_f64(), front.b_f64());

    let lum = luminosity(back.r_f64(), back.g_f64(), back.b_f64());

    let (r, g, b) = set_saturation(back.r_f64(), back.g_f64(), back.b_f64(), sat);
    let (r, g, b) = set_luminocity(r, g, b, lum);

    let front = Color::from_slice_f64(&[r, g, b, front.a_f64()]);
    normal(back, front, opacity)
}

// --- hsl_color ---------------------------------------------------------------
fn hsl_color(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, hsl_color_baseline)
}

fn hsl_color_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let lum = luminosity(back.r_f64(), back.g_f64(), back.b_f64());
    let (r, g, b) = set_luminocity(front.r_f64(), front.g_f64(), front.b_f64(), lum);
    let front = Color::from_slice_f64(&[r, g, b, front.a_f64()]);
    normal(back, front, opacity)
}

// --- hsl_luminosity ----------------------------------------------------------
fn hsl_luminosity(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, hsl_luminosity_baseline)
}

fn hsl_luminosity_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let lum = luminosity(front.r_f64(), front.g_f64(), front.b_f64());
    let (r, g, b) = set_luminocity(back.r_f64(), back.g_f64(), back.b_f64(), lum);

    let front = Color::from_slice_f64(&[r, g, b, front.a_f64()]);
    normal(back, front, opacity)
}

// --- exclusion ----------------------------------------------------------------
fn exclusion(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, exclusion_baseline)
}

fn exclusion_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_exclusion)
}

// blend_exclusion(b, s, t)  ((t) = MUL_UN8((b), (s), (t)), ((b) + (s) - 2*(t)))
fn blend_exclusion(b: i32, s: i32) -> u8 {
    let t = mul8(b, s) as i32;
    (b + s - 2 * t) as u8
}
// --- difference ----------------------------------------------------------------
fn difference(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, difference_baseline)
}

fn difference_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_difference)
}

fn blend_difference(b: i32, s: i32) -> u8 {
    (b - s).unsigned_abs() as u8
}
// --- divide ----------------------------------------------------------------
fn divide(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, divide_baseline)
}

fn divide_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_divide)
}

fn blend_divide(b: i32, s: i32) -> u8 {
    if b == 0 {
        0
    } else if b >= s {
        255
    } else {
        div8(b, s)
    }
}
// --- soft light ----------------------------------------------------------------
fn soft_light(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, soft_light_baseline)
}

fn soft_light_baseline(back: Color, front: Color, opacity: u8) -> Color {
    let r = blend_soft_light(back.r_i32(), front.r_i32());
    let g = blend_soft_light(back.g_i32(), front.g_i32());
    let b = blend_soft_light(back.b_i32(), back.b_i32());

    let front = Color::from_slice_i32(&[r, g, b, front.a_i32()]);
    normal(back, front, opacity)
}

fn blend_soft_light(b: i32, s: i32) -> i32 {
    // The original uses double, but since inputs & output are only 8 bits using
    // f32 should actually be enough.
    let b: f64 = b as f64 / 255.0;
    let s: f64 = s as f64 / 255.0;

    let d = if b <= 0.25 {
        ((16.0 * b - 12.0) * b + 4.0) * b
    } else {
        b.sqrt()
    };

    let r = if s <= 0.5 {
        b - (1.0 - 2.0 * s) * b * (1.0 - b)
    } else {
        b + (2.0 * s - 1.0) * (d - b)
    };

    (r * 255.0 + 0.5) as u32 as i32
}
// --- hard light ----------------------------------------------------------------
fn hard_light(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, hard_light_baseline)
}

fn hard_light_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_hard_light)
}

// --- color burn ----------------------------------------------------------------
fn color_burn(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, color_burn_baseline)
}

fn color_burn_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_color_burn)
}

fn blend_color_burn(b: i32, s: i32) -> u8 {
    if b == 255 {
        return 255;
    }
    let b = 255 - b;
    if b >= s {
        0
    } else {
        255 - div8(b, s)
    }
}

// --- color doge ----------------------------------------------------------------
fn color_dodge(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, color_dodge_baseline)
}

fn color_dodge_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_color_dodge)
}
fn blend_color_dodge(b: i32, s: i32) -> u8 {
    if b == 0 {
        return 0;
    }
    let s = 255 - s;
    if b >= s {
        255
    } else {
        // in floating point: b / (1-s)
        div8(b, s)
    }
}

// --- lighten ----------------------------------------------------------------
fn lighten(backdrop: Color, src: Color, opacity: u8) -> Color {
    blender(backdrop, src, opacity, lighten_baseline)
}

fn lighten_baseline(backdrop: Color, src: Color, opacity: u8) -> Color {
    blend_channel(backdrop, src, opacity, blend_lighten)
}

fn blend_lighten(b: i32, s: i32) -> u8 {
    b.max(s) as u8
}

// --- darken ----------------------------------------------------------------
fn darken(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, darken_baseline)
}

fn darken_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_darken)
}

fn blend_darken(b: i32, s: i32) -> u8 {
    b.min(s) as u8
}
// --- overlay ----------------------------------------------------------------
fn overlay(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, overlay_baseline)
}

fn overlay_baseline(backdrop: Color, src: Color, opacity: u8) -> Color {
    blend_channel(backdrop, src, opacity, blend_overlay)
}

fn blend_overlay(b: i32, s: i32) -> u8 {
    blend_hard_light(s, b)
}
fn blend_hard_light(b: i32, s: i32) -> u8 {
    if s < 128 {
        blend_multiply(b, s << 1)
    } else {
        blend_screen(b, (s << 1) - 255)
    }
}
// --- screen ----------------------------------------------------------------
fn screen(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, screen_baseline)
}

fn screen_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_screen)
}

// blend_screen(b, s, t)     ((b) + (s) - MUL_UN8((b), (s), (t)))
fn blend_screen(a: i32, b: i32) -> u8 {
    (a + b - mul8(a, b) as i32) as u8
}
// --- multiply ----------------------------------------------------------------
fn multiply(back: Color, front: Color, opacity: u8) -> Color {
    blender(back, front, opacity, multiply_baseline)
}

fn multiply_baseline(back: Color, front: Color, opacity: u8) -> Color {
    blend_channel(back, front, opacity, blend_multiply)
}

fn blend_multiply(a: i32, b: i32) -> u8 {
    mul8(a, b)
}

// --- Util --------------------------------------------------------------------

fn blender<F>(back: Color, front: Color, opacity: u8, f: F) -> Color
where
    F: Fn(Color, Color, u8) -> Color,
{
    if back.a() != 0 {
        let norm = normal(back, front, opacity);
        let blend = f(back, front, opacity);
        let normal_to_blend_merge = merge(norm, blend, back.a());
        let src_total_alpha = mul8(front.a_i32(), opacity as i32);
        let composite_alpha = mul8(back.a_i32(), src_total_alpha as i32);
        merge(normal_to_blend_merge, blend, composite_alpha)
    } else {
        normal(back, front, opacity)
    }
}

fn blend_channel<F>(back: Color, front: Color, opacity: u8, f: F) -> Color
where
    F: Fn(i32, i32) -> u8,
{
    let r = f(back.r_i32(), front.r_i32());
    let g = f(back.g_i32(), front.g_i32());
    let b = f(back.b_i32(), front.b_i32());

    let res = [r, g, b, front.a()];
    normal(back, res, opacity)
}

fn mul8(a: i32, b: i32) -> u8 {
    let t = a * b + 0x80;
    let r = ((t >> 8) + t) >> 8;
    r as u8
}

fn div8(a: i32, b: i32) -> u8 {
    let t = a * 0xff;
    let r = (t + (b / 2)) / b;
    r as u8
}

fn blend8(back: u8, src: u8, opacity: u8) -> u8 {
    let src_x = src as i32;
    let back_x = back as i32;
    let a = src_x - back_x;
    let b = opacity as i32;
    let t = a * b + 0x80;
    let r = ((t >> 8) + t) >> 8;
    (back as i32 + r) as u8
}

fn normal(back: Color, front: Color, opacity: u8) -> Color {
    if back.a() == 0 {
        let alpha = mul8(front.a_i32(), opacity as i32);
        return [front.r(), front.g(), front.b(), alpha];
    } else if front.a() == 0 {
        return back;
    }

    let front_a = mul8(front.a_i32(), opacity as i32);

    let res_a = front_a as i32 + back.a_i32() - mul8(back.a_i32(), front_a as i32) as i32;

    let res_r = back.r_i32() + ((front.r_i32() - back.r_i32()) * front.a_i32()) / res_a;
    let res_g = back.g_i32() + ((front.g_i32() - back.g_i32()) * front.a_i32()) / res_a;
    let res_b = back.b_i32() + ((front.b_i32() - back.b_i32()) * front.a_i32()) / res_a;

    Color::from_slice_i32(&[res_r, res_g, res_b, res_a])
}

fn merge(back: Color, front: Color, opacity: u8) -> Color {
    let res_r;
    let res_g;
    let res_b;

    if back.a() == 0 {
        res_r = front.r();
        res_g = front.g();
        res_b = front.b();
    } else if front.a() == 0 {
        res_r = back.r();
        res_g = back.g();
        res_b = back.b();
    } else {
        res_r = blend8(back.r(), front.r(), opacity);
        res_g = blend8(back.g(), front.g(), opacity);
        res_b = blend8(back.b(), front.b(), opacity);
    }
    let res_a = blend8(back.a(), front.a(), opacity);

    if res_a == 0 {
        [0, 0, 0, 0]
    } else {
        [res_r, res_g, res_b, res_a]
    }
}

fn clip_color(mut r: f64, mut g: f64, mut b: f64) -> (f64, f64, f64) {
    let lum = luminosity(r, g, b);
    let min = r.min(g.min(b));
    let max = r.max(g.max(b));

    if min < 0.0 {
        r = lum + (((r - lum) * lum) / (lum - min));
        g = lum + (((g - lum) * lum) / (lum - min));
        b = lum + (((b - lum) * lum) / (lum - min));
    }

    if max > 1.0 {
        r = lum + (((r - lum) * (1.0 - lum)) / (max - lum));
        g = lum + (((g - lum) * (1.0 - lum)) / (max - lum));
        b = lum + (((b - lum) * (1.0 - lum)) / (max - lum));
    }
    (r, g, b)
}

fn set_luminocity(r: f64, g: f64, b: f64, lum: f64) -> (f64, f64, f64) {
    let delta = lum - luminosity(r, g, b);
    clip_color(r + delta, g + delta, b + delta)
}

fn saturation(r: f64, g: f64, b: f64) -> f64 {
    r.max(g.max(b)) - r.min(g.min(b))
}

fn luminosity(r: f64, g: f64, b: f64) -> f64 {
    0.3 * r + 0.59 * g + 0.11 * b
}

fn static_sort3(r: f64, g: f64, b: f64) -> (usize, usize, usize) {
    let (min0, mid0, max0) = ((r, 0), (g, 1), (b, 2));
    // dbg!("--------");
    // dbg!(min0, mid0, max0);
    let (min1, mid1) = if min0.0 < mid0.0 {
        (min0, mid0)
    } else {
        (mid0, min0)
    };
    // dbg!(min1, mid1);
    let (min2, max1) = if min1.0 < max0.0 {
        (min1, max0)
    } else {
        (max0, min1)
    };
    // dbg!(min2, max1);
    let (mid2, max2) = if mid1.0 < max1.0 {
        (mid1, max1)
    } else {
        (max1, mid1)
    };
    // dbg!(mid2, max2);
    (min2.1, mid2.1, max2.1)
}

fn static_sort3_orig(r: f64, g: f64, b: f64) -> (usize, usize, usize) {
    // min = MIN(r, MIN(g, b));
    // ((r) < (((g) < (b)) ? (g) : (b))) ? (r) : (((g) < (b)) ? (g) : (b));
    // max = MAX(r, MAX(g, b));
    // ((r) > (((g) > (b)) ? (g) : (b))) ? (r) : (((g) > (b)) ? (g) : (b))
    // mid = ((r) > (g) ?
    //          ((g) > (b) ?
    //             (g) :
    //             ((r) > (b) ?
    //                (b) :
    //                (r)
    //             )
    //          ) :
    //          ((g) > (b) ?
    //             ((b) > (r) ?
    //                (b) :
    //                (r)
    //             ) :
    //             (g)))

    let min = if r < g.min(b) {
        0 // r
    } else if g < b {
        1 // g
    } else {
        2 // b
    };
    let max = if r > g.max(b) {
        0 // r
    } else if g > b {
        1 // g
    } else {
        2 // b
    };
    let mid = if r > g {
        if g > b {
            1 // g
        } else if r > b {
            2 // b
        } else {
            0 // r
        }
    } else if g > b {
        if b > r {
            2 // b
        } else {
            0 // r
        }
    } else {
        1 // g
    };
    (min, mid, max)
}

const ASEPRITE_SATURATION_BUG_COMPATIBLE: bool = true;

fn set_saturation(r: f64, g: f64, b: f64, sat: f64) -> (f64, f64, f64) {
    let mut col = [r, g, b];

    let (min, mid, max) = if ASEPRITE_SATURATION_BUG_COMPATIBLE {
        static_sort3_orig(r, g, b)
    } else {
        static_sort3(r, g, b)
    };
    if col[max] > col[min] {
        // i.e., they're not all the same
        col[mid] = ((col[mid] - col[min]) * sat) / (col[max] - col[min]);
        col[max] = sat;
    } else {
        col[mid] = 0.0;
        col[max] = 0.0;
    }
    col[min] = 0.0;
    (col[0], col[1], col[2])
}
