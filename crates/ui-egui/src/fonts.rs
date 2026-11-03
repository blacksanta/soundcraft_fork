//! Fonts: egui's bundled defaults, plus the operating system's own UI fonts loaded at runtime when
//! present (never redistributed). A "bold" family is always defined so the UI can use it.

use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// (path, ttc index) candidates for regular and bold UI text, by platform.
const REGULAR: &[(&str, u32)] = &[
    ("/System/Library/Fonts/HelveticaNeue.ttc", 0),
    ("/System/Library/Fonts/Helvetica.ttc", 0),
    ("C:\\Windows\\Fonts\\segoeui.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 0),
    ("/usr/local/share/fonts/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", 0),
    ("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf", 0),
];
const BOLD: &[(&str, u32)] = &[
    ("/System/Library/Fonts/HelveticaNeue.ttc", 1),
    ("/System/Library/Fonts/Helvetica.ttc", 1),
    ("C:\\Windows\\Fonts\\segoeuib.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf", 0),
    ("/usr/local/share/fonts/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf", 0),
    ("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Bold.ttf", 0),
];
/// Japanese fallback (kana and kanji), appended after every family so the Japanese interface and
/// Japanese session, track and clip names draw whatever the interface language. It comes before the
/// Hangul fallback so shared CJK ideographs take Japanese glyph shapes. The Noto Sans CJK
/// collection's face 0 is its Japanese face.
const JAPANESE: &[(&str, u32)] = &[
    ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans W3.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
    ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
    ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/truetype/fonts-japanese-gothic.ttf", 0),
];
/// Bold Japanese for the "bold" family; the regular Japanese face stands in when none is found.
const JAPANESE_BOLD: &[(&str, u32)] = &[
    ("/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans W6.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothB.ttc", 0),
    ("C:\\Windows\\Fonts\\meiryob.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc", 0),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Bold.ttc", 0),
];
/// Hangul fallback, appended after every family so Korean session, track and clip names draw
/// whatever the interface language. The Noto Sans CJK collection's face 1 is its Korean face.
const HANGUL: &[(&str, u32)] = &[
    ("/System/Library/Fonts/AppleSDGothicNeo.ttc", 0),
    ("/System/Library/Fonts/Supplemental/AppleGothic.ttf", 0),
    ("C:\\Windows\\Fonts\\malgun.ttf", 0),
    ("C:\\Windows\\Fonts\\gulim.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/truetype/nanum/NanumGothic.ttf", 0),
    ("/usr/share/fonts/naver-nanum/NanumGothic.ttf", 0),
];
/// Font files larger than this are skipped: egui keeps the whole file in memory.
#[cfg(not(target_arch = "wasm32"))]
const MAX_FONT_BYTES: u64 = 64 << 20;

/// Asks fontconfig for the desktop's font matching `pattern`. Only TrueType/OpenType results are
/// used: egui cannot read Type 1 or bitmap fonts, which `fc-match` may return on minimal systems.
/// With `lang`, the match must support that language (`fc-match` always returns its best match,
/// which is a Latin face when no Korean font is installed).
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn load_from_fontconfig(pattern: &str, lang: Option<&str>) -> Option<FontData> {
    let output = std::process::Command::new("fc-match").args(["--format=%{file}\\n%{index}\\n%{lang}\\n", pattern]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut lines = stdout.lines();
    let path = lines.next()?.trim();
    // The upper 16 bits carry a variable font's named instance; the face index is the lower 16.
    let index = lines.next()?.trim().parse::<u32>().ok()? & 0xFFFF;
    if let Some(lang) = lang
        && !lines.next().unwrap_or_default().split('|').any(|l| l.trim() == lang)
    {
        return None;
    }
    if std::fs::metadata(path).ok()?.len() > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let sfnt = matches!(bytes.get(..4), Some(b"\0\x01\0\0" | b"OTTO" | b"true" | b"ttcf"));
    if !sfnt {
        return None;
    }
    let mut font = FontData::from_owned(bytes);
    font.index = index;
    Some(font)
}

#[cfg(not(target_arch = "wasm32"))]
fn load(cands: &[(&str, u32)], pattern: &str, lang: Option<&str>, system_fonts: bool) -> Option<FontData> {
    if !system_fonts {
        return None;
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if let Some(font) = load_from_fontconfig(pattern, lang) {
        return Some(font);
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    let _ = (pattern, lang);
    for (p, idx) in cands {
        if std::fs::metadata(p).is_ok_and(|m| m.len() > MAX_FONT_BYTES) {
            continue;
        }
        if let Ok(bytes) = std::fs::read(p) {
            let mut fd = FontData::from_owned(bytes);
            fd.index = *idx;
            return Some(fd);
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn load(_: &[(&str, u32)], _: &str, _: Option<&str>, _: bool) -> Option<FontData> {
    None
}

/// How far below the Latin baseline `face` draws by default, as a fraction of the font size.
/// egui centres a fallback face's line box in the first face's, which suits emoji but lifts CJK
/// faces with a deep descent (Hiragino, Apple SD Gothic Neo) above the Latin baseline, so their
/// tops were clipped in track names and rulers. Measured rather than tabled: the metrics differ
/// by platform font.
fn baseline_drop(fonts: &FontDefinitions, primary: &str, face: &str, sample: char) -> Option<f32> {
    const SIZE: f32 = 100.0;
    let mut probe = FontDefinitions { font_data: fonts.font_data.clone(), families: Default::default() };
    let family = FontFamily::Name("baseline-probe".into());
    probe.families.insert(family.clone(), vec![primary.to_owned(), face.to_owned()]);
    let ctx = egui::Context::default();
    ctx.set_fonts(probe);
    let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
    output.textures_delta.clear();
    let galley = ctx.fonts_mut(|f| f.layout_no_wrap(format!("H{sample}"), egui::FontId::new(SIZE, family), egui::Color32::WHITE));
    let row = galley.rows.first()?;
    let (latin, cjk) = (row.glyphs.first()?, row.glyphs.get(1)?);
    let drop = (latin.pos.y - cjk.pos.y) / SIZE;
    (drop.is_finite() && drop.abs() < 0.5).then_some(drop)
}

/// Shift each CJK fallback down onto the Latin baseline of the family it serves. The line height
/// is unchanged (the tweak is visual only), so fixed-height boxes keep their layout.
fn align_cjk_fallbacks(fonts: &mut FontDefinitions) {
    let first = |fonts: &FontDefinitions, family: &FontFamily| fonts.families.get(family).and_then(|f| f.first().cloned());
    let regular = first(fonts, &FontFamily::Proportional);
    let bold = first(fonts, &FontFamily::Name("bold".into()));
    for (face, primary, sample) in [("system-japanese", &regular, 'あ'), ("system-japanese-bold", &bold, 'あ'), ("system-hangul", &regular, '가')]
    {
        let Some(primary) = primary else { continue };
        if !fonts.font_data.contains_key(face) {
            continue;
        }
        if let Some(drop) = baseline_drop(fonts, primary, face, sample)
            && let Some(data) = fonts.font_data.get_mut(face)
        {
            Arc::make_mut(data).tweak.y_offset_factor = drop;
        }
    }
}

/// The font set: the bundled faces, plus the system ones when `system_fonts` is true.
pub fn definitions(system_fonts: bool) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut bold_family = base.clone();
    if let Some(reg) = load(REGULAR, "sans-serif", None, system_fonts) {
        fonts.font_data.insert("system-regular".into(), Arc::new(reg));
        if let Some(f) = fonts.families.get_mut(&FontFamily::Proportional) {
            f.insert(0, "system-regular".into());
        }
    }
    if let Some(b) = load(BOLD, "sans-serif:style=Bold", None, system_fonts) {
        fonts.font_data.insert("system-bold".into(), Arc::new(b));
        bold_family.insert(0, "system-bold".into());
    } else if fonts.font_data.contains_key("system-regular") {
        bold_family.insert(0, "system-regular".into());
    }
    fonts.families.insert(FontFamily::Name("bold".into()), bold_family);
    if let Some(j) = load(JAPANESE, "sans-serif:lang=ja", Some("ja"), system_fonts) {
        fonts.font_data.insert("system-japanese".into(), Arc::new(j));
        let bold = load(JAPANESE_BOLD, "sans-serif:lang=ja:style=Bold", Some("ja"), system_fonts).map(|b| {
            fonts.font_data.insert("system-japanese-bold".into(), Arc::new(b));
            "system-japanese-bold"
        });
        for (name, family) in &mut fonts.families {
            let face = if *name == FontFamily::Name("bold".into()) { bold.unwrap_or("system-japanese") } else { "system-japanese" };
            family.push(face.into());
        }
    }
    if let Some(h) = load(HANGUL, "sans-serif:lang=ko", Some("ko"), system_fonts) {
        fonts.font_data.insert("system-hangul".into(), Arc::new(h));
        for family in fonts.families.values_mut() {
            family.push("system-hangul".into());
        }
    }
    align_cjk_fallbacks(&mut fonts);
    fonts
}

pub fn install(ctx: &egui::Context, system_fonts: bool) {
    ctx.set_fonts(definitions(system_fonts));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangul_candidates_cover_every_desktop_platform() {
        for prefix in ["/System/Library/Fonts/", "C:\\Windows\\Fonts\\", "/usr/share/fonts/"] {
            assert!(HANGUL.iter().any(|(p, _)| p.starts_with(prefix)), "no Hangul candidate under {prefix}");
        }
        // Face 0 of the Noto Sans CJK collection is Japanese; the Korean face is 1.
        assert!(HANGUL.iter().filter(|(p, _)| p.ends_with("NotoSansCJK-Regular.ttc")).all(|(_, i)| *i == 1));
    }

    #[test]
    fn japanese_candidates_cover_every_desktop_platform() {
        for list in [JAPANESE, JAPANESE_BOLD] {
            for prefix in ["/System/Library/Fonts/", "C:\\Windows\\Fonts\\", "/usr/share/fonts/"] {
                assert!(list.iter().any(|(p, _)| p.starts_with(prefix)), "no Japanese candidate under {prefix}");
            }
            // Face 0 of the Noto Sans CJK collection is Japanese.
            assert!(list.iter().filter(|(p, _)| p.contains("NotoSansCJK-")).all(|(_, i)| *i == 0));
        }
    }

    /// The Japanese interface draws in every family, not as boxes; skipped when the system has no
    /// Japanese font.
    #[test]
    fn every_family_draws_japanese_with_a_system_japanese_font() {
        let defs = definitions(true);
        if !defs.font_data.contains_key("system-japanese") {
            eprintln!("skipping: no Japanese system font");
            return;
        }
        let families: Vec<FontFamily> = defs.families.keys().cloned().collect();
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        for family in families {
            let font = egui::FontId::new(13.0, family.clone());
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, "日本語セッションのトラックを録音")), "{family:?} has no Japanese");
        }
    }

    /// Japanese glyphs draw on the Latin baseline without making the row taller, so names in
    /// fixed-height boxes aren't clipped; skipped when the system has no Japanese font.
    #[test]
    fn japanese_sits_on_the_latin_baseline() {
        let defs = definitions(true);
        let Some(ja) = defs.font_data.get("system-japanese") else {
            eprintln!("skipping: no Japanese system font");
            return;
        };
        let drop = ja.tweak.y_offset_factor;
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        let font = egui::FontId::proportional(13.0);
        let layout = |text: &str| ctx.fonts_mut(|f| f.layout_no_wrap(text.into(), font.clone(), egui::Color32::WHITE));
        let (mixed, latin) = (layout("Ab小節"), layout("Ab"));
        assert_eq!(mixed.rect.height(), latin.rect.height(), "Japanese text grew the row");
        let row = &mixed.rows[0];
        let drawn = row.glyphs[2].pos.y + drop * 13.0;
        assert!((row.glyphs[0].pos.y - drawn).abs() < 0.05, "kanji baseline {drawn} vs Latin {}", row.glyphs[0].pos.y);
    }

    /// Every literal glyph the UI fixtures paint is covered by the bundled faces in every family,
    /// without relying on an installed system font. This is the 0.37 font-feature regression test.
    #[test]
    fn bundled_faces_cover_the_ui_glyphs_in_every_family() {
        let mut defs = FontDefinitions::default();
        let proportional = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        defs.families.insert(FontFamily::Name("bold".into()), proportional);
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        // Status/menu text, transport/meter symbols, file/audio/search icons, music symbols and
        // platform key hints used by the UI. Close/delete controls paint a cross in `icons.rs`
        // instead of typesetting U+2715, which no bundled face covers in every family.
        let glyphs = ["›", "v", "♩", "𝄞", "⌘", "📁", "🔊", "🔍", "⬆", "−", "∞"];
        for family in [FontFamily::Proportional, FontFamily::Monospace, FontFamily::Name("bold".into())] {
            let font = egui::FontId::new(13.0, family.clone());
            for glyph in glyphs {
                assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, glyph)), "{family:?} has no glyph for {glyph:?}");
            }
        }
    }

    /// Korean names draw in every family, not as boxes; skipped when the system has no Korean font.
    #[test]
    fn every_family_draws_hangul_with_a_system_korean_font() {
        let defs = definitions(true);
        if !defs.font_data.contains_key("system-hangul") {
            eprintln!("skipping: no Korean system font");
            return;
        }
        let families: Vec<FontFamily> = defs.families.keys().cloned().collect();
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        for family in families {
            let font = egui::FontId::new(13.0, family.clone());
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, "가한국어세션")), "{family:?} has no Hangul");
        }
    }
}
