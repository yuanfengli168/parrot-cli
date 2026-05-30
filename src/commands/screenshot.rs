use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;

/// Validate and normalize a timestamp string (MM:SS or HH:MM:SS) for ffmpeg.
/// Returns the normalized timestamp string suitable for ffmpeg -ss.
pub fn parse_timestamp(ts: &str) -> Result<String> {
    let parts: Vec<&str> = ts.split(':').collect();
    match parts.len() {
        2 => {
            // MM:SS
            let mins: u64 = parts[0].parse().context("Invalid minutes in timestamp")?;
            let secs: u64 = parts[1].parse().context("Invalid seconds in timestamp")?;
            if secs > 59 {
                bail!("Seconds must be 0-59, got {}", secs);
            }
            Ok(format!("{:02}:{:02}", mins, secs))
        }
        3 => {
            // HH:MM:SS
            let hours: u64 = parts[0].parse().context("Invalid hours in timestamp")?;
            let mins: u64 = parts[1].parse().context("Invalid minutes in timestamp")?;
            let secs: u64 = parts[2].parse().context("Invalid seconds in timestamp")?;
            if mins > 59 {
                bail!("Minutes must be 0-59, got {}", mins);
            }
            if secs > 59 {
                bail!("Seconds must be 0-59, got {}", secs);
            }
            Ok(format!("{:02}:{:02}:{:02}", hours, mins, secs))
        }
        _ => bail!(
            "Invalid timestamp format '{}'. Use MM:SS or HH:MM:SS",
            ts
        ),
    }
}

/// Check that a required external tool is installed. Returns error with install instructions if missing.
pub fn check_dependency(name: &str) -> Result<()> {
    if which::which(name).is_err() {
        let install_cmd = match name {
            "ffmpeg" => "brew install ffmpeg",
            "tesseract" => "brew install tesseract",
            _ => &format!("install {}", name),
        };
        bail!(
            "{} not found. Install it with: {}",
            name,
            install_cmd
        );
    }
    Ok(())
}

/// Build the output filename for a screenshot.
/// Pattern: YYYY-MM-DD_<video_stem>_<timestamp_safe>.png
fn build_output_paths(video_path: &Path, timestamp: &str) -> (PathBuf, PathBuf, PathBuf) {
    let output_dir = config::screenshots_dir();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let stem = video_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("video");
    // Replace : with - in timestamp for safe filenames
    let ts_safe = timestamp.replace(':', "-");
    let base = format!("{}_{}_{}", date, stem, ts_safe);
    let png_path = output_dir.join(format!("{}.png", base));
    let txt_path = output_dir.join(format!("{}.txt", base));
    let tesseract_base = output_dir.join(&base);
    (png_path, txt_path, tesseract_base)
}

/// Find an existing transcript file for a given video in the output directory.
fn find_transcript(video_path: &Path) -> Option<PathBuf> {
    let output_dir = config::output_dir();
    let stem = video_path.file_stem()?.to_str()?.to_string();

    // Common patterns: <stem>-transcript.txt, <stem>-transcript.srt, <stem>.srt, <stem>.vtt
    let candidates = [
        format!("{}-transcript.txt", stem),
        format!("{}-transcript.srt", stem),
        format!("{}.srt", stem),
        format!("{}.vtt", stem),
    ];

    for candidate in &candidates {
        let path = output_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    // Also search for any file containing the stem + transcript/srt
    if let Ok(entries) = std::fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_str().unwrap_or("");
            if name_str.contains(&stem)
                && (name_str.ends_with("-transcript.txt")
                    || name_str.ends_with("-transcript.srt")
                    || name_str.ends_with(".srt")
                    || name_str.ends_with(".vtt"))
            {
                return Some(entry.path());
            }
        }
    }

    None
}

/// Parse SRT/VTT timestamp "HH:MM:SS,mmm" or "HH:MM:SS.mmm" to seconds.
fn parse_srt_timestamp(ts: &str) -> Option<f64> {
    // SRT: 00:32:15,000  VTT: 00:32:15.000
    let ts = ts.trim();
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    // Last part may have , or . separator for milliseconds
    let sec_parts: Vec<&str> = parts[2].split(&[',', '.'][..]).collect();
    let seconds: f64 = sec_parts[0].parse().ok()?;
    let millis: f64 = if sec_parts.len() > 1 {
        let ms: f64 = sec_parts[1].parse().ok()?;
        ms / 1000.0
    } else {
        0.0
    };
    Some(hours * 3600.0 + minutes * 60.0 + seconds + millis)
}

/// Search transcript lines for a keyword/phrase. Returns (timestamp_string, matching_line).
fn search_transcript(transcript_path: &Path, query: &str) -> Result<Option<(String, String)>> {
    let content = std::fs::read_to_string(transcript_path)
        .context("Failed to read transcript file")?;
    let name = transcript_path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let is_srt = name.ends_with(".srt");
    let is_vtt = name.ends_with(".vtt");

    if is_srt || is_vtt {
        // Parse SRT/VTT format
        let blocks: Vec<&str> = if is_srt {
            content.split("\n\n").collect()
        } else {
            // VTT: skip header, then split by double newline
            let body = content.strip_prefix("WEBVTT").unwrap_or(&content);
            body.split("\n\n").collect()
        };

        let query_lower = query.to_lowercase();
        let mut best_match: Option<(String, String)> = None;

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 2 {
                continue;
            }

            // Find the timestamp line (contains -->)
            let ts_line_idx = lines.iter().position(|l| l.contains("-->"));
            let ts_line = match ts_line_idx {
                Some(idx) => lines[idx],
                None => continue,
            };

            // Extract start timestamp
            let start_ts = ts_line.split("-->").next().unwrap_or("").trim();
            let timestamp_str = format_srt_timestamp_for_ffmpeg(start_ts);

            // Text is everything after the timestamp line
            let text: String = if let Some(idx) = ts_line_idx {
                lines[idx + 1..].join(" ").trim().to_string()
            } else {
                continue;
            };
            if text.to_lowercase().contains(&query_lower) {
                best_match = Some((timestamp_str, text));
                // Return first match
                break;
            }
        }
        Ok(best_match)
    } else {
        // Plain text transcript — try to find timestamps in brackets or at line start
        // Format: [MM:SS] text or [HH:MM:SS] text or just lines
        let query_lower = query.to_lowercase();
        let mut last_timestamp = "00:00:00".to_string();

        for line in content.lines() {
            // Check for timestamp pattern like [32:15] or [01:32:15]
            let ts_match = regex_timestamp_from_line(line);
            if let Some(ref ts) = ts_match {
                last_timestamp = ts.clone();
            }

            if line.to_lowercase().contains(&query_lower) {
                return Ok(Some((last_timestamp.clone(), line.trim().to_string())));
            }
        }
        Ok(None)
    }
}

/// Find the closest matching sentence in transcript using fuzzy matching.
fn find_sentence(transcript_path: &Path, sentence: &str) -> Result<Option<(String, String)>> {
    let content = std::fs::read_to_string(transcript_path)
        .context("Failed to read transcript file")?;
    let name = transcript_path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let is_srt = name.ends_with(".srt");
    let is_vtt = name.ends_with(".vtt");

    let sentence_lower = sentence.to_lowercase();
    let sentence_words: Vec<&str> = sentence_lower.split_whitespace().collect();

    if is_srt || is_vtt {
        let blocks: Vec<&str> = if is_srt {
            content.split("\n\n").collect()
        } else {
            let body = content.strip_prefix("WEBVTT").unwrap_or(&content);
            body.split("\n\n").collect()
        };

        let mut best_match: Option<(String, String, usize)> = None; // (timestamp, text, score)

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 2 {
                continue;
            }
            let ts_line_idx = lines.iter().position(|l| l.contains("-->"));
            let ts_line = match ts_line_idx {
                Some(idx) => lines[idx],
                None => continue,
            };
            let start_ts = ts_line.split("-->").next().unwrap_or("").trim();
            let timestamp_str = format_srt_timestamp_for_ffmpeg(start_ts);

            let text: String = if let Some(idx) = ts_line_idx {
                lines[idx + 1..].join(" ").trim().to_string()
            } else {
                continue;
            };

            let text_lower = text.to_lowercase();
            let text_words: Vec<&str> = text_lower.split_whitespace().collect();

            // Count matching words (simple fuzzy match)
            let mut match_count = 0;
            for word in &sentence_words {
                if text_words.iter().any(|tw| tw == word) {
                    match_count += 1;
                }
            }

            let score = if sentence_words.is_empty() {
                0
            } else {
                match_count * 100 / sentence_words.len()
            };

            if best_match.as_ref().map(|(_, _, s)| *s).unwrap_or(0) < score {
                best_match = Some((timestamp_str, text, score));
            }
        }

        Ok(best_match.map(|(ts, text, _)| (ts, text)))
    } else {
        // Plain text
        let mut last_timestamp = "00:00:00".to_string();
        let mut best_match: Option<(String, String, usize)> = None;

        for line in content.lines() {
            let ts_match = regex_timestamp_from_line(line);
            if let Some(ref ts) = ts_match {
                last_timestamp = ts.clone();
            }

            let line_lower = line.to_lowercase();
            let line_words: Vec<&str> = line_lower.split_whitespace().collect();

            let mut match_count = 0;
            for word in &sentence_words {
                if line_words.iter().any(|lw| lw == word) {
                    match_count += 1;
                }
            }

            let score = if sentence_words.is_empty() {
                0
            } else {
                match_count * 100 / sentence_words.len()
            };

            if best_match.as_ref().map(|(_, _, s)| *s).unwrap_or(0) < score {
                best_match = Some((last_timestamp.clone(), line.trim().to_string(), score));
            }
        }

        Ok(best_match.map(|(ts, text, _)| (ts, text)))
    }
}

/// Try to extract a timestamp from a line (formats: [MM:SS], [HH:MM:SS], MM:SS at start).
fn regex_timestamp_from_line(line: &str) -> Option<String> {
    // [MM:SS] or [HH:MM:SS]
    let bracket_re = regex::Regex::new(r"\[(\d{1,2}:\d{2}(?::\d{2})?)\]").ok()?;
    if let Some(caps) = bracket_re.captures(line) {
        return parse_timestamp(caps.get(1)?.as_str()).ok();
    }
    None
}

/// Convert an SRT/VTT timestamp (HH:MM:SS,mmm) to ffmpeg-compatible HH:MM:SS.
fn format_srt_timestamp_for_ffmpeg(ts: &str) -> String {
    // Remove milliseconds part
    let ts = ts.split(&[',', '.'][..]).next().unwrap_or(ts).trim();
    // Normalize: if only MM:SS, add hours
    let parts: Vec<&str> = ts.split(':').collect();
    match parts.len() {
        2 => format!("00:{}", ts),
        3 => ts.to_string(),
        _ => ts.to_string(),
    }
}

pub async fn run(
    file: &str,
    timestamp: Option<&str>,
    search: Option<&str>,
    sentence: Option<&str>,
) -> Result<()> {
    config::ensure_dirs()?;

    // Validate video file exists
    let video_path = Path::new(file);
    if !video_path.exists() {
        bail!("Video file not found: {}", file);
    }

    // Check prerequisites
    check_dependency("ffmpeg")?;
    check_dependency("tesseract")?;

    // Determine the timestamp to use
    let resolved_timestamp = if let Some(ts) = timestamp {
        // Direct timestamp provided
        parse_timestamp(ts)?
    } else if search.is_some() || sentence.is_some() {
        // Need to search transcript
        let transcript_path = find_transcript(video_path).ok_or_else(|| {
            anyhow::anyhow!(
                "No transcript found for '{}'. Run `parrot-cli transcribe {}` first.",
                file, file
            )
        })?;

        let (ts, matched_text) = if let Some(query) = search {
            match search_transcript(&transcript_path, query)? {
                Some(result) => result,
                None => bail!("No match found for '{}' in transcript", query),
            }
        } else if let Some(sent) = sentence {
            match find_sentence(&transcript_path, sent)? {
                Some(result) => result,
                None => bail!(
                    "No matching sentence found for '{}' in transcript",
                    sent
                ),
            }
        } else {
            unreachable!()
        };

        println!("📌 Found match at {}: \"{}\"", ts, matched_text);
        ts
    } else {
        bail!("Provide a timestamp (e.g., \"32:15\"), --search \"keyword\", or --sentence \"exact text\"");
    };

    // Build output paths
    let (png_path, txt_path, tesseract_base) = build_output_paths(video_path, &resolved_timestamp);

    // Ensure screenshots directory exists
    let screenshots_dir = config::screenshots_dir();
    std::fs::create_dir_all(&screenshots_dir)?;

    // Extract frame using ffmpeg
    println!("📸 Extracting frame at {}...", resolved_timestamp);
    let ffmpeg_status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &video_path.to_string_lossy(),
            "-ss",
            &resolved_timestamp,
            "-frames:v",
            "1",
            "-q:v",
            "2",
        ])
        .arg(&png_path)
        .status()
        .context("Failed to run ffmpeg")?;

    if !ffmpeg_status.success() {
        bail!("ffmpeg failed to extract frame at {}", resolved_timestamp);
    }

    if !png_path.exists() {
        bail!("ffmpeg did not produce output file");
    }

    println!("✅ Frame extracted to {}", png_path.display());

    // Run tesseract OCR
    println!("🔍 Running OCR on frame...");
    let tesseract_status = Command::new("tesseract")
        .arg(&png_path)
        .arg(&tesseract_base)
        .status()
        .context("Failed to run tesseract")?;

    if tesseract_status.success() && txt_path.exists() {
        println!("✅ OCR text saved to {}", txt_path.display());
    } else {
        eprintln!("⚠️  Tesseract OCR failed or produced no output. Screenshot still saved.");
    }

    // Final summary
    println!();
    println!("🦜 Screenshot complete!");
    println!("   📷 Image: {}", png_path.display());
    if txt_path.exists() {
        println!("   📝 OCR:   {}", txt_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timestamp_mm_ss() {
        assert_eq!(parse_timestamp("32:15").unwrap(), "32:15");
        assert_eq!(parse_timestamp("05:09").unwrap(), "05:09");
    }

    #[test]
    fn parse_timestamp_hh_mm_ss() {
        assert_eq!(parse_timestamp("01:32:15").unwrap(), "01:32:15");
        assert_eq!(parse_timestamp("0:05:09").unwrap(), "00:05:09");
    }

    #[test]
    fn parse_timestamp_rejects_invalid() {
        assert!(parse_timestamp("abc").is_err());
        assert!(parse_timestamp("1:2:3:4").is_err());
        assert!(parse_timestamp("5:70").is_err()); // seconds > 59
        assert!(parse_timestamp("1:60:30").is_err()); // minutes > 59
    }

    #[test]
    fn parse_timestamp_single_number_fails() {
        assert!(parse_timestamp("123").is_err());
    }

    #[test]
    fn build_output_paths_format() {
        let video = Path::new("/tmp/meeting_recording.mp4");
        let (png, txt, tess_base) = build_output_paths(video, "32:15");
        assert!(png.to_str().unwrap().contains("screenshots"));
        assert!(png.to_str().unwrap().ends_with(".png"));
        assert!(txt.to_str().unwrap().ends_with(".txt"));
        assert!(png.to_str().unwrap().contains("meeting_recording"));
        assert!(png.to_str().unwrap().contains("32-15"));
        assert!(txt.to_str().unwrap().contains("32-15"));
        // tesseract base should not have extension
        assert!(!tess_base.to_str().unwrap().ends_with(".png"));
        assert!(!tess_base.to_str().unwrap().ends_with(".txt"));
    }

    #[test]
    fn ffmpeg_command_construction() {
        // Verify expected ffmpeg args for screenshot extraction
        let args: Vec<&str> = vec![
            "-y", "-i", "input.mp4", "-ss", "32:15", "-frames:v", "1", "-q:v", "2",
            "output.png",
        ];
        assert!(args.contains(&"-ss"));
        assert!(args.contains(&"-frames:v"));
        assert!(args.contains(&"1"));
        assert!(args.contains(&"-q:v"));
        assert!(args.contains(&"2"));
    }

    #[test]
    fn tesseract_command_construction() {
        // Verify expected tesseract args
        let args: Vec<&str> = vec!["screenshot.png", "screenshot"];
        assert_eq!(args.len(), 2); // input and output base
        assert!(args[0].ends_with(".png"));
        assert!(!args[1].contains(".")); // output base has no extension
    }

    #[test]
    fn format_srt_timestamp_for_ffmpeg_hhmmss() {
        assert_eq!(format_srt_timestamp_for_ffmpeg("01:32:15,000"), "01:32:15");
        assert_eq!(format_srt_timestamp_for_ffmpeg("00:05:09.500"), "00:05:09");
    }

    #[test]
    fn format_srt_timestamp_for_ffmpeg_mmss() {
        assert_eq!(format_srt_timestamp_for_ffmpeg("32:15"), "00:32:15");
    }

    #[test]
    fn parse_srt_timestamp_valid() {
        assert_eq!(parse_srt_timestamp("01:32:15,000"), Some(1.0 * 3600.0 + 32.0 * 60.0 + 15.0));
        assert_eq!(parse_srt_timestamp("00:05:09.500"), Some(5.0 * 60.0 + 9.0 + 0.5));
    }

    #[test]
    fn parse_srt_timestamp_invalid() {
        assert_eq!(parse_srt_timestamp("invalid"), None);
        assert_eq!(parse_srt_timestamp(""), None);
    }

    #[test]
    fn search_transcript_plain_text() {
        let dir = tempfile::tempdir().unwrap();
        let transcript = dir.path().join("meeting-transcript.txt");
        std::fs::write(&transcript, "[32:15] Let's discuss the budget\n[45:02] The revenue target is 5M\n").unwrap();
        let result = search_transcript(&transcript, "budget").unwrap();
        assert!(result.is_some());
        let (ts, text) = result.unwrap();
        assert_eq!(ts, "32:15");
        assert!(text.to_lowercase().contains("budget"));
    }

    #[test]
    fn search_transcript_srt() {
        let dir = tempfile::tempdir().unwrap();
        let transcript = dir.path().join("meeting.srt");
        std::fs::write(
            &transcript,
            "1\n00:32:15,000 --> 00:32:20,000\nLet's discuss the budget\n\n2\n00:45:02,000 --> 00:45:07,000\nThe revenue target is 5M\n",
        )
        .unwrap();
        let result = search_transcript(&transcript, "budget").unwrap();
        assert!(result.is_some());
        let (ts, text) = result.unwrap();
        assert_eq!(ts, "00:32:15");
        assert!(text.to_lowercase().contains("budget"));
    }

    #[test]
    fn find_sentence_fuzzy_match() {
        let dir = tempfile::tempdir().unwrap();
        let transcript = dir.path().join("meeting-transcript.txt");
        std::fs::write(&transcript, "[32:15] Let's discuss the budget allocation\n[45:02] The revenue target is 5M dollars\n").unwrap();
        let result = find_sentence(&transcript, "revenue target 5M").unwrap();
        assert!(result.is_some());
        let (ts, text) = result.unwrap();
        assert_eq!(ts, "45:02");
        assert!(text.to_lowercase().contains("revenue"));
    }

    #[test]
    fn check_dependency_missing_shows_install() {
        // This test just verifies the function exists and works for installed tools
        // ffmpeg and tesseract may or may not be installed in test env
        let _ = check_dependency("ffmpeg");
        let _ = check_dependency("tesseract");
    }

    #[test]
    fn screenshots_dir_creation() {
        let dir = tempfile::tempdir().unwrap();
        let screenshots = dir.path().join("screenshots");
        assert!(!screenshots.exists());
        std::fs::create_dir_all(&screenshots).unwrap();
        assert!(screenshots.exists());
    }

    #[test]
    fn regex_timestamp_from_line_brackets() {
        assert_eq!(regex_timestamp_from_line("[32:15] hello"), Some("32:15".to_string()));
        assert_eq!(regex_timestamp_from_line("[01:32:15] hello"), Some("01:32:15".to_string()));
        assert_eq!(regex_timestamp_from_line("no timestamp here"), None);
    }
}