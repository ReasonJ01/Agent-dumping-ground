use fframes::{
    AudioMixOptions, CombinedMediaProvider, EncoderOptions, LimiterOptions, MediaDirectory,
    MediaProvider, RenderOptions, cli,
};
use fframes_skia_renderer::{SkiaPipelineConfig, cli::SkiaRenderers};
use krakatoa_intro::{IntroMedia, KrakatoaIntro};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let media = IntroMedia::new().expect("media");
    // drone and sound effects: loaded at runtime, stereo
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("dynamic_media");
    let dir = MediaDirectory::read_folder(&folder).expect("dynamic media folder");
    let dynamic = dir.process_media_source().expect("dynamic media");
    let all = CombinedMediaProvider::from([&media as &dyn MediaProvider, &dynamic]);

    cli::new(
        &KrakatoaIntro,
        RenderOptions {
            media: Some(&all),
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "17"), ("preset", "medium"), ("tune", "film")]),
                ..Default::default()
            },
            audio_mix: AudioMixOptions {
                limiter: Some(LimiterOptions { ceiling_db: -1.3, ..Default::default() }),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    // Skia on the CPU by default (cloud containers have no GPU); `--renderer gpu` on a workstation
    .renderers(SkiaRenderers {
        default: Some(cli::Renderer::SkiaCpu),
        pipeline: SkiaPipelineConfig::default(),
    })
    .run()
}
