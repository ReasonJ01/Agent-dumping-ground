//! A 32 second MGSV-style mission opening for the Krakatoa end of sprint demos.
//!
//! Four scenes, each hard cut through a glitch:
//! cold open (typed transmission) → mission area (topographic map with the
//! sprint goals as markers) → credits (the squad, MGSV style) → episode title.

use std::sync::LazyLock;

use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Duration, FFramesContext, Frame, Scene,
    Scenes, Shader, ShaderUniforms, Svgr, Video, include_media_dir,
};

include_media_dir!(pub struct IntroMedia, "examples/krakatoa-intro/media");

pub const FPS: usize = 30;

// palette: near black, bone white, iDroid amber, one alert red
pub const BG: &str = "#060606";
pub const BONE: &str = "#e9e6df";
pub const AMBER: &str = "#f0a531";
pub const AMBER_DIM: &str = "#7a5a24";
pub const GREY: &str = "#86827a";
pub const RED: &str = "#d8432b";

pub const COND: &str = "Anton";
pub const MONO: &str = "IBM Plex Mono";
pub const SANS: &str = "Inter 24pt";

// scene starts, in seconds of the video
pub const COLD: f32 = 0.0;
pub const MAP: f32 = 4.5;
pub const CREDITS: f32 = 13.5;
pub const TITLE: f32 = 24.5;
pub const END: f32 = 32.0;

pub struct Shaders {
    pub grain: Shader,
    pub contour: Shader,
}

pub static SHADERS: LazyLock<Shaders> = LazyLock::new(|| Shaders {
    grain: Shader::sksl(include_str!("shaders/grain.sksl")),
    contour: Shader::sksl(include_str!("shaders/contour.sksl")),
});

// ---------------------------------------------------------------------------
// easing and noise, all on 0..1

pub fn prog(x: f32, a: f32, b: f32) -> f32 {
    ((x - a) / (b - a)).clamp(0.0, 1.0)
}

pub fn expo_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 { 1.0 } else { 1.0 - 2f32.powf(-10.0 * t) }
}

pub fn cubic_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

/// Deterministic hash noise, 0..1.
pub fn hash(n: f32) -> f32 {
    let x = (n * 127.1 + 311.7).sin() * 43_758.547;
    x - x.floor()
}

fn gsec(frame: &Frame) -> f32 {
    frame.global_index as f32 / FPS as f32
}

// ---------------------------------------------------------------------------
// shared pieces

/// Crop-mark corners around a rectangle.
fn corners(x: f32, y: f32, w: f32, h: f32, arm: f32, color: &str, width: f32) -> Svgr<'static> {
    let d = format!(
        "M{x} {y1} V{y} H{x1} M{x2} {y} H{x3} V{y1} M{x3} {y2} V{y3} H{x2} M{x1} {y3} H{x} V{y2}",
        x1 = x + arm,
        x2 = x + w - arm,
        x3 = x + w,
        y1 = y + arm,
        y2 = y + h - arm,
        y3 = y + h,
    );
    fframes::svgr!(<path d={d} fill="none" stroke={color.to_owned()} stroke-width={width} />)
}

fn mono(x: f32, y: f32, text: String, color: &str, size: f32, spacing: f32, anchor: &'static str) -> Svgr<'static> {
    fframes::svgr!(
        <text x={x} y={y} font-family={MONO} font-weight="500" font-size={size} letter-spacing={spacing}
              fill={color.to_owned()} text-anchor={anchor}>{text}</text>
    )
}

/// Text typed out at `cps` characters per second from `start`, with a block caret.
fn typed(t: f32, start: f32, x: f32, y: f32, text: &str, color: &str, size: f32, caret: bool) -> Svgr<'static> {
    if t < start {
        return Svgr::empty();
    }
    let n = (((t - start) * 30.0) as usize).min(text.chars().count());
    let shown: String = text.chars().take(n).collect();
    let done = n == text.chars().count();
    // IBM Plex Mono advance is 0.6 em, plus the letter spacing
    let cx = x + n as f32 * (size * 0.6 + 2.0);
    let blink = !done || (t * 2.5).fract() < 0.5;
    let caret_o = if caret && blink { 1.0 } else { 0.0 };
    fframes::svgr!(
        <g>
            {mono(x, y, shown, color, size, 2.0, "start")}
            <rect x={cx + 2.0} y={y - size * 0.8} width={size * 0.55} height={size * 0.95} fill={color.to_owned()} opacity={caret_o} />
        </g>
    )
}

/// MGSV-style digital tear: chromatic copies pushed sideways and the picture
/// split into horizontal bands that jump. `g` is the strength (0 = clean).
fn glitch<'a, F>(g: f32, seed: f32, content: F) -> Svgr<'a>
where
    F: Fn(&'static str) -> Svgr<'a>,
{
    if g <= 0.01 {
        return content(BONE);
    }
    let tick = seed * 31.0;
    let dx = 26.0 * g;
    let mut layers = vec![
        fframes::svgr!(<g transform={format!("translate({} 0)", dx)} opacity={0.75 * g} style="mix-blend-mode:screen">{content("#ff3b30")}</g>),
        fframes::svgr!(<g transform={format!("translate({} 0)", -dx)} opacity={0.75 * g} style="mix-blend-mode:screen">{content("#30e0ff")}</g>),
    ];
    // bands of random height across the whole frame
    let mut y = 0.0;
    let mut i = 0.0;
    while y < 1080.0 {
        let h = 30.0 + hash(tick + i * 7.3) * 140.0;
        let jump = (hash(tick + i * 3.1 + 0.5) - 0.5) * 2.0;
        let off = if hash(tick + i * 1.7) > 0.45 { jump * 90.0 * g } else { 0.0 };
        let id = format!("gl{}-{}", (seed * 1000.0) as i64, i as i32);
        layers.push(fframes::svgr!(
            <g>
                <clipPath id={id.clone()}><rect x="-200" y={y} width="2320" height={h} /></clipPath>
                <g clip-path={format!("url(#{id})")} transform={format!("translate({off} 0)")}>{content(BONE)}</g>
            </g>
        ));
        y += h;
        i += 1.0;
    }
    fframes::svgr!(<g>{layers}</g>)
}

/// Glitch strength for an element that tears in at `a` and out at `b`.
fn tear(t: f32, a: f32, b: f32) -> (f32, f32) {
    let len = 0.2;
    if t < a || t > b {
        return (0.0, 0.0);
    }
    let g_in = 1.0 - prog(t, a, a + len);
    let g_out = prog(t, b - len, b);
    let g = g_in.max(g_out);
    // flicker while tearing: some frames drop out entirely
    let f = (t * FPS as f32).floor();
    let visible = if g > 0.35 && hash(f + a * 13.0) < 0.3 { 0.0 } else { 1.0 };
    (g, visible)
}

// ---------------------------------------------------------------------------
// the video

#[derive(Debug)]
pub struct KrakatoaIntro;

impl Video for KrakatoaIntro {
    const FPS: usize = FPS;
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }

    fn audio(&self) -> AudioMap<'_> {
        let sfx = |name: &'static str, at: f32, gain: f32| AudioTrack::new(name, Second(at)..Eof).gain_db(gain);
        let mut tracks = vec![
            AudioTrack::new("drone.wav", Second(0.0)..Eof).gain_db(-4.0).fade_out(2.0),
            AudioTrack::new("sfx_typing.mp3", Second(COLD + 0.5)..Second(COLD + 1.7)).gain_db(-7.0),
            AudioTrack::new("sfx_typing.mp3", Second(COLD + 1.9)..Second(COLD + 3.0)).gain_db(-7.0),
            AudioTrack::new("sfx_typing.mp3", Second(COLD + 3.1)..Second(COLD + 3.8)).gain_db(-7.0),
            sfx("sfx_glitch.mp3", MAP - 0.15, -5.0),
            sfx("sfx_whoosh.mp3", MAP - 0.2, -14.0),
            sfx("sfx_glitch.mp3", CREDITS - 0.15, -5.0),
            sfx("sfx_braam.mp3", TITLE + 0.2, -6.0),
            sfx("sfx_impact.mp3", TITLE + TITLE_SLAM, -6.0),
            sfx("sfx_glitch.mp3", END - 0.9, -9.0),
        ];
        for (i, _) in TARGETS.iter().enumerate() {
            tracks.push(sfx("sfx_blip.mp3", MAP + target_at(i), -4.0));
        }
        for i in 0..CARDS.len() {
            tracks.push(sfx("sfx_glitch.mp3", CREDITS + card_at(i), -9.0));
        }
        AudioMap::from(tracks)
    }

    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(vec![&ColdOpen as &dyn Scene, &MissionArea, &Credits, &EpisodeTitle])
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = gsec(&frame);
        let grain = SHADERS.grain.draw(
            &frame,
            ShaderUniforms::new().float("uGrain", 0.09).float("uVignette", 0.7),
        );
        // every cut tears for a few frames
        let cut = [MAP, CREDITS, TITLE]
            .iter()
            .map(|c| t - c)
            .find(|d| (-0.1..0.15).contains(d));
        let scene = match cut {
            Some(d) => glitch(1.0 - (d + 0.1) / 0.25, (t * FPS as f32).floor(), |_| ctx.render_scenes(&frame)),
            None => ctx.render_scenes(&frame),
        };
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080">
                <rect width="1920" height="1080" fill={BG} />
                {scene}
                <image href={grain.href()} x="0" y="0" width="1920" height="1080" />
            </svg>
        )
    }
}

// ---------------------------------------------------------------------------
// 1. cold open: an incoming transmission is typed out

#[derive(Debug)]
pub struct ColdOpen;

impl Scene for ColdOpen {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(MAP - COLD)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let scan_y = (t * 260.0) % 1300.0 - 110.0;
        let rec = if (t * 1.6).fract() < 0.6 { 1.0 } else { 0.0 };
        fframes::svgr!(
            <g>
                <rect x="0" y={scan_y} width="1920" height="110" fill={AMBER} opacity="0.025" />
                <g opacity={prog(t, 0.2, 0.5)}>
                    <circle cx="172" cy="152" r="8" fill={RED} opacity={rec} />
                    {mono(194.0, 160.0, "INCOMING TRANSMISSION".to_owned(), GREY, 20.0, 5.0, "start")}
                </g>
                {typed(t, 0.5, 160.0, 800.0, "SAVINGS ENGINEERING // KRAKATOA SQUAD", AMBER, 34.0, false)}
                {typed(t, 1.9, 160.0, 856.0, "SPRINT 3990 · 21 OCT – 04 NOV 2026", AMBER, 34.0, false)}
                {typed(t, 3.1, 160.0, 930.0, "END OF SPRINT DEMOS", BONE, 52.0, true)}
            </g>
        )
    }
}

// ---------------------------------------------------------------------------
// 2. mission area: the sprint goals drop onto a topographic map

/// (x, y, label, primary goal)
const TARGETS: &[(f32, f32, &str, bool)] = &[
    (470.0, 400.0, "01  ISA RUNS ON E2", true),
    (1230.0, 330.0, "02  DM ISA FEATURES", true),
    (720.0, 720.0, "03  INTERNAL REC IMPROVEMENTS", true),
    (1400.0, 640.0, "04  COP CHECKS", true),
    (1120.0, 900.0, "SIDE OP  LBG GEM API", false),
];

fn target_at(i: usize) -> f32 {
    0.9 + i as f32 * 0.95
}

#[derive(Debug)]
pub struct MissionArea;

fn marker(t: f32, i: usize) -> Svgr<'static> {
    let (x, y, text, primary) = TARGETS[i];
    let at = target_at(i);
    if t < at {
        return Svgr::empty();
    }
    let color = if primary { AMBER } else { GREY };
    let s = t - at;
    // the reticle drops in from large and rotated
    let k = expo_out(s / 0.45);
    let scale = 2.6 - 1.6 * k;
    let rot = 45.0 + (1.0 - k) * 90.0;
    let ring = (s * 0.8).fract();
    // callout: elbow up and out, then the label
    let p = prog(s, 0.25, 0.75);
    let right = x < 1500.0;
    let lx = if right { x + 90.0 } else { x - 90.0 };
    let ly = y - 70.0;
    let l1 = 70.0;
    let l2 = 90.0;
    let total = l1 + l2;
    let dash = format!("{} {}", total * p, total);
    let path = format!("M{x} {} V{ly} H{lx}", y - 22.0);
    let tx = if right { lx + 14.0 } else { lx - 14.0 };
    let anchor = if right { "start" } else { "end" };
    let text_o = prog(s, 0.6, 0.8);
    // label types on with the callout
    let n = ((prog(s, 0.6, 1.1)) * text.chars().count() as f32) as usize;
    let shown: String = text.chars().take(n).collect();
    fframes::svgr!(
        <g>
            <g transform={format!("translate({x} {y}) scale({scale}) rotate({rot})")} opacity={k}>
                <rect x="-14" y="-14" width="28" height="28" fill="none" stroke={color.to_owned()} stroke-width="3" />
                <rect x="-4" y="-4" width="8" height="8" fill={color.to_owned()} />
            </g>
            <circle cx={x} cy={y} r={22.0 + ring * 50.0} fill="none" stroke={color.to_owned()} stroke-width="2" opacity={(1.0 - ring) * 0.6 * k} />
            <path d={path} fill="none" stroke={color.to_owned()} stroke-width="2" stroke-dasharray={dash} />
            <g opacity={text_o}>
                <text x={tx} y={ly + 9.0} text-anchor={anchor} font-family={MONO} font-weight="600" font-size="28"
                      letter-spacing="3" fill={if primary { BONE } else { GREY }}>{shown}</text>
            </g>
        </g>
    )
}

impl Scene for MissionArea {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(CREDITS - MAP)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let d = CREDITS - MAP;
        // slow push in over the whole scene
        let zoom = 1.06 - 0.06 * cubic_in_out(t / d);
        let well_x = 0.55 + 0.08 * (t / d);
        let map = SHADERS.contour.draw(
            &frame,
            ShaderUniforms::new()
                .float2("uWell", well_x, 0.55)
                .float("uDepth", 0.35)
                .float("uDensity", 18.0)
                .float("uBright", 0.55 * prog(t, 0.0, 0.6))
                .color("uInk", Color::hex(AMBER_DIM))
                .color("uHot", Color::hex(AMBER)),
        );
        let sweep_x = (t / 3.0).fract() * 2100.0 - 90.0;
        let markers: Vec<Svgr> = (0..TARGETS.len()).map(|i| marker(t, i)).collect();
        let grid: Vec<Svgr> = (1..16)
            .map(|i| {
                let x = i as f32 * 120.0;
                fframes::svgr!(<line x1={x} y1="0" x2={x} y2="1080" stroke={AMBER} stroke-width="1" opacity="0.07" />)
            })
            .chain((1..9).map(|i| {
                let y = i as f32 * 120.0;
                fframes::svgr!(<line x1="0" y1={y} x2="1920" y2={y} stroke={AMBER} stroke-width="1" opacity="0.07" />)
            }))
            .collect();
        let hud_o = prog(t, 0.3, 0.7);
        fframes::svgr!(
            <g>
                <g transform={format!("translate(960 540) scale({zoom}) translate(-960 -540)")}>
                    <image href={map.href()} x="0" y="0" width="1920" height="1080" />
                    {grid}
                    <rect x={sweep_x} y="0" width="3" height="1080" fill={AMBER} opacity="0.35" />
                    <rect x={sweep_x - 90.0} y="0" width="90" height="1080" fill={AMBER} opacity="0.04" />
                    {markers}
                </g>
                <g opacity={hud_o}>
                    {corners(72.0, 72.0, 1776.0, 936.0, 30.0, BONE, 2.0)}
                    {mono(120.0, 140.0, "MISSION AREA".to_owned(), GREY, 20.0, 5.0, "start")}
                    <text x="120" y="190" font-family={MONO} font-weight="600" font-size="38" letter-spacing="4" fill={BONE}>"KRAKATOA / SAV"</text>
                    {mono(1800.0, 140.0, "SPRINT 3990".to_owned(), GREY, 20.0, 5.0, "end")}
                    <text x="1800" y="190" text-anchor="end" font-family={MONO} font-weight="600" font-size="38" letter-spacing="4" fill={AMBER}>"78 TASKS"</text>
                    {mono(120.0, 990.0, "OBJECTIVES MARKED".to_owned(), GREY, 18.0, 5.0, "start")}
                    {mono(1800.0, 990.0, "21.10 — 04.11.2026".to_owned(), GREY, 18.0, 5.0, "end")}
                </g>
            </g>
        )
    }
}

// ---------------------------------------------------------------------------
// 3. credits: the squad, one name at a time, tearing in and out

/// (role, name, x, y, anchor)
const CARDS: &[(&str, &str, f32, f32, &str)] = &[
    ("STARRING", "KODIAK CONRAD", 160.0, 860.0, "start"),
    ("STARRING", "JOSHUA MCCORMICK", 1760.0, 300.0, "end"),
    ("STARRING", "DOMINYKAS SLIUZAS", 160.0, 300.0, "start"),
    ("STARRING", "TERENCE SONG", 1760.0, 860.0, "end"),
    ("GUEST STARRING", "MIKE DAVIES", 160.0, 860.0, "start"),
    ("SPECIAL GUEST STARRING", "LIAM BATES   YUSUF YUSUF", 1760.0, 300.0, "end"),
    ("AND", "JOEL REASON", 160.0, 860.0, "start"),
    ("", "A KRAKATOA SQUAD PRODUCTION", 960.0, 560.0, "middle"),
];

const CARD_LEN: f32 = 1.3;

fn card_at(i: usize) -> f32 {
    0.4 + i as f32 * CARD_LEN
}

#[derive(Debug)]
pub struct Credits;

impl Scene for Credits {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(TITLE - CREDITS)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let d = TITLE - CREDITS;
        let bg = SHADERS.contour.draw(
            &frame,
            ShaderUniforms::new()
                .float2("uWell", 0.3 + 0.3 * (t / d), 0.5)
                .float("uDepth", 0.25)
                .float("uDensity", 14.0)
                .float("uBright", 0.18)
                .color("uInk", Color::hex("#4a4740"))
                .color("uHot", Color::hex(AMBER_DIM)),
        );
        let last = CARDS.len() - 1;
        let cards: Vec<Svgr> = CARDS
            .iter()
            .enumerate()
            .map(|(i, (role, name, x, y, anchor))| {
                let a = card_at(i);
                // the last card holds to the end of the scene
                let b = if i == last { d - 0.15 } else { a + CARD_LEN - 0.1 };
                let (g, visible) = tear(t, a, b);
                if visible == 0.0 || t < a || t > b {
                    return Svgr::empty();
                }
                let (x, y, anchor) = (*x, *y, *anchor);
                let (role, name) = (*role, *name);
                let size = if i == last { 44.0 } else { 60.0 };
                // a slow drift while it holds
                let drift = (t - a) * 6.0 * if anchor == "end" { -1.0 } else { 1.0 };
                glitch(g, (t * FPS as f32).floor() + i as f32 * 100.0, move |fill| {
                    let role_fill = if fill == BONE { GREY } else { fill };
                    fframes::svgr!(
                        <g transform={format!("translate({drift} 0)")}>
                            <text x={x} y={y - 72.0} text-anchor={anchor} font-family={MONO} font-weight="500" font-size="24"
                                  letter-spacing="6" fill={role_fill}>{role}</text>
                            <text x={x} y={y} text-anchor={anchor} font-family={SANS} font-weight="400" font-size={size}
                                  letter-spacing="12" fill={fill}>{name}</text>
                        </g>
                    )
                })
            })
            .collect();
        fframes::svgr!(
            <g>
                <image href={bg.href()} x="0" y="0" width="1920" height="1080" />
                {cards}
            </g>
        )
    }
}

// ---------------------------------------------------------------------------
// 4. episode title

const TITLE_SLAM: f32 = 1.1;

#[derive(Debug)]
pub struct EpisodeTitle;

impl Scene for EpisodeTitle {
    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(END - TITLE)
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let d = END - TITLE;
        let out_at = d - 0.9;
        let (g_ep, v_ep) = tear(t, 0.5, out_at);
        let (g_title, v_title) = tear(t, TITLE_SLAM, out_at);
        // the slam tears harder and longer than the rest
        let slam = if t >= TITLE_SLAM { (-(t - TITLE_SLAM) * 5.0).exp() } else { 0.0 };
        let g_title = g_title.max(slam * 0.9);
        let flash = slam * 0.35;
        // a hairline draws out from the centre under the title
        let line_w = 1200.0 * expo_out(prog(t, TITLE_SLAM + 0.4, TITLE_SLAM + 1.4)).max(0.0005);
        let line_o = if t < out_at { 1.0 } else { 0.0 };
        // a very slow push in on the title during the hold
        let push = 1.0 + 0.03 * prog(t, TITLE_SLAM, out_at);
        let episode = if v_ep > 0.0 && t >= 0.5 && t <= out_at {
            glitch(g_ep, (t * FPS as f32).floor() + 7.0, |fill| {
                let fill = if fill == BONE { AMBER } else { fill };
                fframes::svgr!(
                    <text x="960" y="470" text-anchor="middle" font-family={MONO} font-weight="600" font-size="30"
                          letter-spacing="16" fill={fill}>"EPISODE 3990"</text>
                )
            })
        } else {
            Svgr::empty()
        };
        let title = if v_title > 0.0 && t >= TITLE_SLAM && t <= out_at {
            glitch(g_title, (t * FPS as f32).floor() + 13.0, |fill| {
                fframes::svgr!(
                    <text x="960" y="620" text-anchor="middle" font-family={COND} font-size="176"
                          letter-spacing="10" fill={fill}>"FUTALOGNKOSAURUS"</text>
                )
            })
        } else {
            Svgr::empty()
        };
        fframes::svgr!(
            <g>
                <g transform={format!("translate(960 540) scale({push}) translate(-960 -540)")}>
                    {episode}
                    {title}
                    <rect x={960.0 - line_w / 2.0} y="676" width={line_w} height="2" fill={AMBER} opacity={line_o} />
                </g>
                <rect width="1920" height="1080" fill={BONE} opacity={flash} />
            </g>
        )
    }
}
