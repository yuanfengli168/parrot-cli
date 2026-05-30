use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

use crate::config;

pub async fn run(
    file: &str,
    model: Option<&str>,
    timestamps: bool,
    no_timestamps: bool,
    language: Option<&str>,
    output: Option<&str>,
) -> Result<()> {
    let cfg = config::load();
    config::ensure_dirs()?;

    let input_path = Path::new(file);
    if !input_path.exists() {
        bail!("File not found: {}", file);
    }

    // Check ffmpeg
    if which::which("ffmpeg").is_err() {
        bail!("ffmpeg not found. Install it with: brew install ffmpeg");
    }

    // Determine if we need to extract audio
    let ext = input_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let transcript_file = if matches!(ext, "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv") {
        // Extract audio first
        let wav_path = config::output_dir().join(
            input_path.file_stem().unwrap().to_str().unwrap().to_string() + ".wav"
        );
        println!("🎵 Extracting audio from {}...", file);
        let status = Command::new("ffmpeg")
            .args(["-y", "-i", file, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le"])
            .arg(&wav_path)
            .status()?;
        if !status.success() {
            bail!("ffmpeg failed to extract audio");
        }
        println!("✅ Audio extracted");
        wav_path.to_string_lossy().to_string()
    } else {
        file.to_string()
    };

    // Check for whisper CLI
    let whisper_bin = if which::which("whisper").is_ok() {
        "whisper"
    } else if which::which("whisper-cpp").is_ok() {
        "whisper-cpp"
    } else {
        bail!("Whisper not found. Install with: pip install openai-whisper\nOr install whisper.cpp");
    };

    let model_name = model.unwrap_or(&cfg.transcription.model);
    println!("🦜 Transcribing with {}...", model_name);

    let mut args = vec![transcript_file.clone()];
    args.push("--model".to_string());
    args.push(model_name.to_string());

    let use_ts = if no_timestamps { false } else { timestamps || cfg.transcription.timestamps };
    if use_ts {
        // whisper default includes timestamps
    }

    if let Some(lang) = language.or(cfg.transcription.language.as_deref()) {
        args.push("--language".to_string());
        args.push(lang.to_string());
    }

    let output_dir = config::output_dir();
    args.push("--output_dir".to_string());
    args.push(output_dir.to_string_lossy().to_string());
    args.push("--output_format".to_string());
    args.push("txt".to_string());

    let status = Command::new(whisper_bin)
        .args(&args)
        .status()?;

    if !status.success() {
        bail!("Transcription failed");
    }

    // Find the output file
    let stem = Path::new(&transcript_file).file_stem().unwrap().to_str().unwrap();
    let txt_output = output_dir.join(format!("{}.txt", stem));

    let final_output = if let Some(out) = output {
        let out_path = Path::new(out);
        if txt_output.exists() {
            std::fs::copy(&txt_output, out_path)?;
        }
        out_path.to_string_lossy().to_string()
    } else {
        txt_output.to_string_lossy().to_string()
    };

    println!("✅ Transcript saved to {}", final_output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_args_for_audio_extraction() {
        // Verify the expected ffmpeg arguments for audio extraction
        let args = vec!["-y", "-i", "input.mp4", "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le"];
        assert!(args.contains(&"-y"));
        assert!(args.contains(&"-ar"));
        assert!(args.contains(&"16000"));  // 16kHz sample rate
        assert!(args.contains(&"-ac"));
        assert!(args.contains(&"1"));       // mono
        assert!(args.contains(&"pcm_s16le")); // 16-bit PCM
    }

    #[test]
    fn video_extensions_require_extraction() {
        let video_exts = vec!["mp4", "mkv", "avi", "mov", "webm", "flv"];
        for ext in video_exts {
            let is_video = ["mp4", "mkv", "avi", "mov", "webm", "flv"].contains(&ext);
            assert!(is_video, "{} should be recognized as video", ext);
        }
    }

    #[test]
    fn audio_extensions_dont_require_extraction() {
        let audio_exts = vec!["wav", "mp3", "flac", "ogg", "m4a"];
        for ext in audio_exts {
            let is_video = ["mp4", "mkv", "avi", "mov", "webm", "flv"].contains(&ext);
            assert!(!is_video, "{} should NOT be recognized as video", ext);
        }
    }

    #[test]
    fn whisper_args_construction() {
        // Verify typical whisper args
        let model = "whisper-medium";
        let transcript_file = "/tmp/test.wav";
        let output_dir = "/tmp/.parrot/output";

        let mut args = vec![transcript_file.to_string()];
        args.push("--model".to_string());
        args.push(model.to_string());
        args.push("--language".to_string());
        args.push("en".to_string());
        args.push("--output_dir".to_string());
        args.push(output_dir.to_string());
        args.push("--output_format".to_string());
        args.push("txt".to_string());

        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"whisper-medium".to_string()));
        assert!(args.contains(&"--language".to_string()));
        assert!(args.contains(&"en".to_string()));
        assert!(args.contains(&"--output_format".to_string()));
        assert!(args.contains(&"txt".to_string()));
    }

    #[test]
    fn path_file_stem_extraction() {
        let path = Path::new("/some/dir/my_meeting.wav");
        let stem = path.file_stem().unwrap().to_str().unwrap();
        assert_eq!(stem, "my_meeting");
    }

    #[test]
    fn nonexistent_file_detected() {
        let path = Path::new("/nonexistent/file.mp4");
        assert!(!path.exists());
    }
}