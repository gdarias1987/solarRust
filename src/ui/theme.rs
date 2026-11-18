//! Design tokens for the Mission Deck shell: one dark appearance, semantic colours,
//! a type scale and the `rune-kit` theme the controls render with.
//!
//! Rune composites in linear light, so white overlays on a near-black ground sit at much
//! lower alpha than a CSS dashboard would use (0.04-0.09, not 0.12-0.18).

use std::sync::OnceLock;

use rune::{Color, TextStyle, color::hex};
use rune_kit::{Kit, PaletteFocus, PaletteSeed};

/// Opaque colours are authored as sRGB hex; alpha is applied later.
fn c(value: &str) -> Color {
    hex(value).unwrap_or(Color::WHITE)
}

/// The palette. Names are semantic: a widget asks for `text_2`, never for "grey".
pub struct Tokens {
    pub bg_deep: Color,

    pub panel: Color,
    pub panel_hi: Color,
    pub panel_sunken: Color,
    pub glass: Color,
    pub glass_strong: Color,
    pub glass_soft: Color,
    pub hairline: Color,

    pub hairline_soft: Color,
    pub text: Color,
    pub text_2: Color,
    pub text_3: Color,
    pub text_4: Color,
    pub teal: Color,

    pub amber: Color,
    pub amber_dim: Color,
    pub violet: Color,

    pub success: Color,

    pub danger: Color,
}

impl Tokens {
    fn new() -> Self {
        Self {
            bg_deep: c("#05060A"),

            panel: c("#0D1017"),
            panel_hi: c("#121721"),
            panel_sunken: c("#070910"),
            glass: Color::WHITE.alpha(0.045),
            glass_strong: Color::WHITE.alpha(0.075),
            glass_soft: Color::WHITE.alpha(0.025),
            hairline: Color::WHITE.alpha(0.085),

            hairline_soft: Color::WHITE.alpha(0.045),
            text: c("#F3F6FA"),
            text_2: c("#A9B4C4"),
            text_3: c("#74809A"),
            text_4: c("#4B5468"),
            teal: c("#3FE3C8"),

            amber: c("#FFAE45"),
            amber_dim: c("#A86A1E"),
            violet: c("#9484FF"),

            success: c("#4BE08D"),

            danger: c("#FF6070"),
        }
    }

    pub fn get() -> &'static Tokens {
        static TOKENS: OnceLock<Tokens> = OnceLock::new();
        TOKENS.get_or_init(Tokens::new)
    }
}

/// Type scale. The mono micro-label with wide tracking is the instrument signature.
pub const DISPLAY: TextStyle = TextStyle::new(38.0, 600.0).features("'tnum' 1");
pub const NUM: TextStyle = TextStyle::new(28.0, 600.0).features("'tnum' 1");
pub const H1: TextStyle = TextStyle::new(22.0, 620.0).tracking(-0.2);
pub const H2: TextStyle = TextStyle::new(16.0, 600.0).tracking(-0.1);
pub const H3: TextStyle = TextStyle::new(13.0, 600.0);
pub const WORDMARK: TextStyle = TextStyle::new(13.5, 700.0).tracking(3.2);

pub const BODY: TextStyle = TextStyle::new(13.0, 430.0).lh(20.0);
pub const BODY_SM: TextStyle = TextStyle::new(12.0, 430.0).lh(18.0);
pub const LABEL: TextStyle = TextStyle::new(11.5, 560.0);
pub const MICRO: TextStyle = TextStyle::new(9.5, 620.0).mono().tracking(1.15);
pub const MONO: TextStyle = TextStyle::new(11.0, 500.0).mono().tracking(0.35);
pub const MONO_SM: TextStyle = TextStyle::new(9.5, 500.0).mono().tracking(0.7);

/// The `rune-kit` theme: derived from three seeds, then retuned to the Mission Deck scale.
pub fn kit() -> &'static Kit {
    static KIT: OnceLock<Kit> = OnceLock::new();
    KIT.get_or_init(|| {
        let t = Tokens::get();
        let mut kit = Kit::from_palette_focus(
            PaletteSeed {
                bg: t.bg_deep,
                fg: t.text,
                accent: t.teal,
            },
            PaletteFocus::Accent,
        );
        kit.ty.ui = TextStyle::new(13.0, 450.0).lh(20.0);
        kit.ty.ui_medium = TextStyle::new(13.0, 560.0).lh(20.0);
        kit.ty.ui_compact = TextStyle::new(13.0, 450.0).lh(16.0);
        kit.ty.small = TextStyle::new(11.5, 500.0).lh(16.0);
        kit.ty.small_medium = TextStyle::new(11.5, 560.0).lh(16.0);
        kit.ty.body = TextStyle::new(13.0, 430.0).lh(20.0);
        kit.ty.body_medium = TextStyle::new(13.0, 560.0).lh(20.0);
        kit.ty.text_small = TextStyle::new(12.0, 430.0).lh(18.0);
        kit.ty.text_small_medium = TextStyle::new(12.0, 560.0).lh(18.0);
        kit.ty.text_xsmall = TextStyle::new(11.0, 430.0).lh(16.0);
        kit.ty.text_xsmall_medium = TextStyle::new(11.0, 560.0).lh(16.0);
        kit.ty.mono = TextStyle::new(11.0, 500.0).mono().tracking(0.35);
        kit.radius.xs = 4.0;
        kit.radius.sm = 6.0;
        kit.radius.md = 10.0;
        kit.radius.lg = 14.0;
        kit.radius.xl = 20.0;
        kit
    })
}
