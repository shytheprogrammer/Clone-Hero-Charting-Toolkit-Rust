//! Decode local recordings, draw their envelope, and play the same aligned
//! timeline that is written during export. FFmpeg remains a separate process.
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{mpsc::{self, Receiver}, Arc},
    thread,
    time::Duration,
};
pub const RATE: u32 = 48_000;
pub const CHANNELS: u16 = 2;
const PEAK_FRAMES: usize = 480;
const MAX_PREVIEW_BYTES: u64 = 256 * 1024 * 1024;
pub struct AudioClip {
    pub samples: Arc<[i16]>,
    pub peaks: Vec<(f32, f32)>,
}
impl AudioClip {
    pub fn from_samples(samples: Vec<i16>) -> Self {
        let peaks = samples.chunks(PEAK_FRAMES * CHANNELS as usize).map(|chunk| {
            let mut low = 0_i16;
            let mut high = 0_i16;
            for &sample in chunk { low = low.min(sample); high = high.max(sample); }
            (low as f32 / 32768., high as f32 / 32768.)
        }).collect();
        Self { samples: samples.into(), peaks }
    }
    pub fn duration(&self) -> f64 {
        self.samples.len() as f64 / CHANNELS as f64 / RATE as f64
    }
    pub fn envelope(&self, from: f64, to: f64) -> (f32, f32) {
        if to <= 0. || from >= self.duration() { return (0., 0.); }
        let start = (from.max(0.) * RATE as f64 / PEAK_FRAMES as f64).floor() as usize;
        let end = (to.max(0.) * RATE as f64 / PEAK_FRAMES as f64).ceil() as usize;
        self.peaks.get(start..end.min(self.peaks.len())).unwrap_or(&[])
            .iter().fold((0_f32, 0_f32), |(low, high), &(a, b)| (low.min(a), high.max(b)))
    }
}
fn ffmpeg() -> Command {
    let bundled = crate::engine::asset_root().join(if cfg!(windows){"vendor/ffmpeg.exe"}else{"vendor/ffmpeg"});
    let mut command = Command::new(if bundled.exists() { bundled } else { PathBuf::from("ffmpeg") });
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-protocol_whitelist", "file,pipe"]);
    command
}
pub fn decode(path: &Path) -> Result<AudioClip, String> {
    if !path.is_file() { return Err("The recording file is missing.".into()); }
    let mut child = ffmpeg().arg("-i").arg(path)
        .args(["-map", "0:a:0", "-vn", "-sn", "-dn", "-ac", "2", "-ar", "48000", "-f", "s16le", "pipe:1"])
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        .map_err(|e| format!("Could not start the audio reader: {e}"))?;
    let stderr = child.stderr.take().unwrap();
    let errors = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.take(64 * 1024).read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    });
    let mut bytes = Vec::new();
    let read = child.stdout.take().unwrap().take(MAX_PREVIEW_BYTES + 1).read_to_end(&mut bytes);
    if read.is_err() || bytes.len() as u64 > MAX_PREVIEW_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        let _ = errors.join();
        return Err("This recording is too large for the in-memory preview (about 23 minutes). Export can still process the full recording.".into());
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    let error = errors.join().unwrap_or_default();
    if !status.success() || bytes.len() < CHANNELS as usize * 2 {
        return Err(format!("Could not decode the recording: {}", error.trim()));
    }
    if bytes.len() % (CHANNELS as usize * 2) != 0 { return Err("The audio reader returned incomplete frames.".into()); }
    let samples = bytes.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
    Ok(AudioClip::from_samples(samples))
}
/// Positive shift adds silence; negative shift trims the beginning. Only the
/// new destination is written. Native sample rate and channels are retained.
pub fn write_aligned(input: &Path, output: &Path, shift_ms: i64) -> Result<f64, String> {
    if !(-3_600_000..=3_600_000).contains(&shift_ms) { return Err("Audio alignment must be within one hour of the original recording.".into()); }
    let filter = if shift_ms >= 0 {
        format!("adelay={shift_ms}:all=1")
    } else {
        format!("atrim=start={:.3},asetpts=PTS-STARTPTS", -shift_ms as f64 / 1000.)
    };
    let result = ffmpeg().arg("-i").arg(input)
        .args(["-map", "0:a:0", "-vn", "-sn", "-dn", "-af", &filter, "-map_metadata", "-1", "-c:a", "pcm_s24le", "-n"])
        .arg(output).output().map_err(|e| format!("Could not align the audio: {e}"))?;
    if !result.status.success() {
        return Err(format!("Audio alignment failed: {}", String::from_utf8_lossy(&result.stderr).trim()));
    }
    let wav = hound::WavReader::open(output).map_err(|e| e.to_string())?;
    if wav.duration() == 0 { return Err("This alignment would trim away the entire recording. Move the waveform later.".into()); }
    Ok(wav.duration() as f64 / wav.spec().sample_rate as f64)
}
pub fn duration(path:&Path)->Result<f64,String> {
    if let Ok(wav)=hound::WavReader::open(path) {return Ok(wav.duration() as f64/wav.spec().sample_rate as f64);}
    let result=ffmpeg().args(["-loglevel","info","-i"]).arg(path).args(["-map","0:a:0","-t","0","-f","null","-"]).output().map_err(|e|e.to_string())?;
    if !result.status.success(){return Err(format!("Could not read the recording: {}",String::from_utf8_lossy(&result.stderr).trim()));}
    let text=String::from_utf8_lossy(&result.stderr);
    let stamp=regex::Regex::new(r"Duration: (\d+):(\d+):(\d+(?:\.\d+)?)").unwrap();
    let Some(c)=stamp.captures(&text) else {return Ok(0.);};
    Ok(c[1].parse::<f64>().unwrap_or(0.)*3600.+c[2].parse::<f64>().unwrap_or(0.)*60.+c[3].parse::<f64>().unwrap_or(0.))
}
/// Lazy source: silence and trimmed samples never allocate a second recording.
pub struct AlignedSource {
    samples: Arc<[i16]>,
    shift_frames: i64,
    first_frame: i64,
    frames: u64,
    emitted: u64,
}
impl AlignedSource {
    pub fn new(clip: &AudioClip, shift_ms: i64, seek_seconds: f64, end_seconds: f64) -> Self {
        let first_frame = (seek_seconds.max(0.) * RATE as f64).round() as i64;
        let end_frame = (end_seconds.max(0.) * RATE as f64).round() as i64;
        Self { samples: clip.samples.clone(), shift_frames: shift_ms * RATE as i64 / 1000,
            first_frame, frames: (end_frame - first_frame).max(0) as u64, emitted: 0 }
    }
}
impl Iterator for AlignedSource {
    type Item = i16;
    fn next(&mut self) -> Option<i16> {
        if self.emitted >= self.frames * CHANNELS as u64 { return None; }
        let frame = self.first_frame + (self.emitted / CHANNELS as u64) as i64 - self.shift_frames;
        let channel = (self.emitted % CHANNELS as u64) as usize;
        self.emitted += 1;
        if frame < 0 { return Some(0); }
        Some(self.samples.get(frame as usize * CHANNELS as usize + channel).copied().unwrap_or(0))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.frames * CHANNELS as u64 - self.emitted) as usize;
        (remaining, Some(remaining))
    }
}
impl Source for AlignedSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { CHANNELS }
    fn sample_rate(&self) -> u32 { RATE }
    fn total_duration(&self) -> Option<Duration> { Some(Duration::from_secs_f64(self.frames as f64 / RATE as f64)) }
}
#[derive(Default)]
pub struct Player {
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    base_seconds: f64,
    end_seconds: f64,
}
impl Player {
    pub fn start(&mut self, clip: &AudioClip, shift_ms: i64, seconds: f64, end: f64, volume: f32) -> Result<(), String> {
        self.stop();
        if self.stream.is_none() {
            let (stream, handle) = OutputStream::try_default().map_err(|e| format!("Could not open an audio output device: {e}"))?;
            self.stream = Some(stream); self.handle = Some(handle);
        }
        let sink = Sink::try_new(self.handle.as_ref().unwrap()).map_err(|e| e.to_string())?;
        self.base_seconds = seconds; self.end_seconds = end;
        sink.set_volume(volume.clamp(0., 1.));
        sink.append(AlignedSource::new(clip, shift_ms, seconds, end));
        self.sink = Some(sink); Ok(())
    }
    pub fn position(&self) -> Option<f64> {
        self.sink.as_ref().map(|s| if s.empty() {self.end_seconds}else{(self.base_seconds + s.get_pos().as_secs_f64()).min(self.end_seconds)})
    }
    pub fn playing(&self) -> bool { self.sink.as_ref().is_some_and(|s| !s.is_paused() && !s.empty()) }
    pub fn pause(&self) { if let Some(s) = &self.sink { s.pause(); } }
    pub fn volume(&self, volume: f32) { if let Some(s) = &self.sink { s.set_volume(volume.clamp(0., 1.)); } }
    pub fn stop(&mut self) { if let Some(s) = self.sink.take() { s.stop(); } }
}
#[derive(Default)]
pub struct PreviewAudio {
    pub path: Option<PathBuf>,
    pub clip: Option<Arc<AudioClip>>,
    pub error: Option<String>,
    pub player: Player,
    loading: Option<Receiver<Result<AudioClip, String>>>,
}
impl PreviewAudio {
    pub fn sync(&mut self, selected: &Option<PathBuf>) {
        if &self.path != selected {
            self.player.stop(); self.clip = None; self.error = None; self.loading = None;
            self.path = selected.clone();
            if let Some(path) = selected.clone() {
                let (tx, rx) = mpsc::channel(); self.loading = Some(rx);
                thread::spawn(move || { let _ = tx.send(decode(&path)); });
            }
        }
        if let Some(result) = self.loading.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.loading = None;
            match result { Ok(clip) => self.clip = Some(Arc::new(clip)), Err(e) => self.error = Some(e) }
        }
    }
    pub fn loading(&self) -> bool { self.loading.is_some() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a native audio output device"]
    fn native_device_playback_clock_pause_and_seek() {
        let clip=AudioClip::from_samples(vec![0;RATE as usize*CHANNELS as usize*3]);
        let mut player=Player::default();player.start(&clip,250,0.,3.25,0.).unwrap();
        thread::sleep(Duration::from_millis(250));assert!(player.playing());assert!(player.position().unwrap()>0.05);
        player.pause();thread::sleep(Duration::from_millis(100));let paused=player.position().unwrap();thread::sleep(Duration::from_millis(100));assert!((player.position().unwrap()-paused).abs()<0.02);assert!(!player.playing());
        player.start(&clip,-250,1.25,2.75,0.).unwrap();thread::sleep(Duration::from_millis(150));assert!(player.position().unwrap()>=1.25);assert!(player.position().unwrap()<2.);player.stop();assert!(player.position().is_none());
    }
    #[test]
    fn aligned_playback_adds_silence_trims_and_seeks_in_stereo() {
        let clip = AudioClip::from_samples(vec![1, 2, 3, 4, 5, 6]);
        let padded: Vec<_> = AlignedSource::new(&clip, 1, 0., 0.002).collect();
        assert!(padded[..96].iter().all(|v| *v == 0));
        assert_eq!(&padded[96..102], &[1, 2, 3, 4, 5, 6]);
        let long = AudioClip::from_samples((0..200).collect());
        let trimmed: Vec<_> = AlignedSource::new(&long, -1, 0., 1. / RATE as f64).collect();
        assert_eq!(trimmed, vec![96, 97]);
        let sought: Vec<_> = AlignedSource::new(&long, 0, 1. / RATE as f64, 2. / RATE as f64).collect();
        assert_eq!(sought, vec![2, 3]);
    }
    #[test]
    fn waveform_envelope_handles_negative_and_out_of_range_time() {
        let clip = AudioClip::from_samples(vec![-16_384, 16_384, 0, 0]);
        assert_eq!(clip.envelope(-1., 0.), (0., 0.));
        assert_eq!(clip.envelope(1., 2.), (0., 0.));
        assert_eq!(clip.envelope(0., 0.001), (-0.5, 0.5));
    }
    #[test]
    fn exported_audio_padding_and_trimming_match_original_samples() {
        let root = std::env::temp_dir().join(format!("chart-starter-audio-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        let input = root.join("input.wav");
        let mut writer = hound::WavWriter::create(&input, hound::WavSpec { channels: 2, sample_rate: 48000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int }).unwrap();
        for frame in 0..4800 { writer.write_sample((frame + 1) as i16).unwrap(); writer.write_sample(-(frame as i16 + 1)).unwrap(); }
        writer.finalize().unwrap();
        let padded = root.join("padded.wav");
        assert!((write_aligned(&input, &padded, 10).unwrap() - 0.11).abs() < 0.0001);
        let decoded = decode(&padded).unwrap();
        assert!(decoded.samples[..960].iter().all(|s| *s == 0));
        assert_eq!(&decoded.samples[960..966], &[1, -1, 2, -2, 3, -3]);
        let trimmed = root.join("trimmed.wav");
        assert!((write_aligned(&input, &trimmed, -10).unwrap() - 0.09).abs() < 0.0001);
        let decoded = decode(&trimmed).unwrap();
        assert_eq!(&decoded.samples[..4], &[481, -481, 482, -482]);
        let before = std::fs::read(&input).unwrap();
        assert!(write_aligned(&input, &root.join("empty.wav"), -200).is_err());
        assert_eq!(before, std::fs::read(&input).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }
}
