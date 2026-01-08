use egui::Color32;

macro_rules! define_color_schemes {
    ($(($variant:ident, $name:expr, $shader:expr, $map_fn:expr)),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum ColorScheme {
            $($variant),*
        }

        impl ColorScheme {
            pub const ALL: &'static [ColorScheme] = &[$(ColorScheme::$variant),*];
            pub const NAMES: &'static [&'static str] = &[$($name),*];
            pub const SHADERS: &'static [&'static str] = &[$($shader),*];

            #[inline]
            pub fn name(&self) -> &'static str {
                match self {
                    $(ColorScheme::$variant => $name),*
                }
            }

            #[inline]
            pub fn from_name(name: &str) -> Option<ColorScheme> {
                match name {
                    $($name => Some(ColorScheme::$variant)),*,
                    _ => None,
                }
            }

            #[inline]
            pub fn to_index(&self) -> usize {
                Self::ALL.iter().position(|x| x == self).unwrap()
            }

            #[inline]
            pub fn shader(&self) -> &'static str {
                match self {
                    $(ColorScheme::$variant => $shader),*
                }
            }

            #[inline]
            pub fn map_value(&self, value: f32, is_empty: bool, base_color_not_empty: f32) -> Color32 {
                let v = if !is_empty {
                    (base_color_not_empty + value * (1.0 - base_color_not_empty)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                
                match self {
                    $(ColorScheme::$variant => ($map_fn)(v)),*
                }
            }
        }
    };
}

// Helper functions for color mapping
fn viridis_map(v: f32) -> Color32 {
    let r = (68.0 + v * (253.0 - 68.0)) as u8;
    let g = (1.0 + v * (231.0 - 1.0)) as u8;
    let b = (84.0 + v * (37.0 - 84.0)) as u8;
    Color32::from_rgb(r, g, b)
}

fn plasma_map(v: f32) -> Color32 {
    let r = (13.0 + v * (240.0 - 13.0)) as u8;
    let g = (8.0 + v * (50.0 - 8.0)) as u8;
    let b = (135.0 + v * (33.0 - 135.0)) as u8;
    Color32::from_rgb(r, g, b)
}

fn grayscale_map(v: f32) -> Color32 {
    let c = (v * 255.0) as u8;
    Color32::from_rgb(c, c, c)
}

fn redblue_map(v: f32) -> Color32 {
    if v < 0.5 {
        let t = v * 2.0;
        Color32::from_rgb(0, 0, (t * 255.0) as u8)
    } else {
        let t = (v - 0.5) * 2.0;
        Color32::from_rgb((t * 255.0) as u8, 0, ((1.0 - t) * 255.0) as u8)
    }
}

// ============================================================
// ADD NEW COLOR SCHEMES HERE - Just add one line!
// Format: (EnumVariant, "display name", shader_str, map_function)
// ============================================================
define_color_schemes!(
    (Viridis,   "viridis",   include_str!("shaders/viridis.glsl"),   viridis_map),
    (Plasma,    "plasma",    include_str!("shaders/plasma.glsl"),    plasma_map),
    (Grayscale, "grayscale", include_str!("shaders/grayscale.glsl"), grayscale_map),
    (RedBlue,   "red-blue",  include_str!("shaders/redblue.glsl"),   redblue_map),
    // Add new color schemes here:
);
