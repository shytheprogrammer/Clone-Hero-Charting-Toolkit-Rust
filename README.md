# Charting Toolkit v1.2.2

by ShyTheProgrammer

A native Windows desktop recreation of [Chart Starter](https://github.com/shytheprogrammer/Chart-Starter), with a modern dark interface and a Rust conversion engine.

## Launch

Double-click **ChartingToolkit.exe**. Keep `vendor`, `gp_bridge.cjs`, and `examples` beside the executable. The portable package includes Node.js and FFmpeg; no Python, Rust, Node, or FFmpeg installation is needed to use it. Files are processed locally.

1. Browse or drop a Guitar Pro score, or try one of the included demos.
2. Select Drums, Guitar, Bass, and Rhythm tracks. Check the note preview and drum mapping. The live collision summary under the preview updates after each MIDI reassignment and shows whether you reduced or increased overlapping hits. Expand collision details to see named source sounds and jump to their positions.
3. Set the overall chart BPM directly on Score & tracks or in Song details. Both controls share one value for every instrument and difficulty, lyric timing, ratings and export; recordings are not stretched. Set song information, enabled difficulties, and optional rating overrides.
4. Attach the matching recording in Pattern preview. Press **Play**, click the preview to seek, and drag its background waveform left/right to line it up with the notes. Use **< 1 bar** / **1 bar >** to move every charted instrument and difficulty together. Attach optional LRC lyrics, artwork, backgrounds, or synchronized stems.
5. Open **Lyrics** to manage standard/enhanced LRC files, edit words and times, review repaired backward timestamps and align the first lyric to the audio playhead.
6. Choose an export folder and create the song package.
7. Open `notes.chart` in Moonscraper, review timing and every difficulty and instrument, and playtest in Clone Hero.

Save/open projects preserve settings and file paths in JSON. Attached files remain at their original locations.

## Playback and alignment

The sixteen-beat preview draws the recording's waveform behind the notes and follows the moving playhead during playback. Play/Pause, Stop, volume, a playhead slider, and the start-beat slider let you review different sections. Click the preview to seek; drag it horizontally to change audio alignment, or type an exact millisecond value.

**Constant BPM**, beside the BPM number on Score & tracks and Song details, overrides every Guitar Pro tempo change. When checked, export writes exactly one BPM flag at tick zero using the entered BPM, even after shifting notes. When unchecked (the default), Overall BPM scales the source tempo map relative to its first tempo. Time signatures remain preserved in either mode. Playback, lyrics, spacing audits, intensity ratings and export use the selected timing mode. The setting is saved with the project; older projects default to unchecked. Changing BPM does not stretch the recording.

Moving audio right adds leading silence to the exported recording. Moving it left removes that amount from the beginning, including any music in that interval. Aligned audio exports as 24-bit WAV, preserving its original sample rate and channels. All full-length audio stems receive the same adjustment. Preview excerpts are kept intact. The exported `song.ini` delay and chart offset stay zero because alignment is already applied to the audio. Original files are never modified. An adjustment that removes an entire recording or stem is rejected.

The note arrows change one shared offset in starting-meter bars across every instrument and difficulty, even when you are previewing only one part: four quarter-note beats for a 4/4 source, three for a 3/4 source, and so on. Each click applies the same tick offset to every part. Tempo/time-signature changes, sections and Star Power move with the notes. The left arrow stops before the earliest selected note would cross time zero. LRC lyrics stay tied to the recording and follow audio alignment and their separate lyric offset. Reset buttons restore the original note/audio positions. Saved projects retain both offsets; earlier projects' audio delay is carried into the new alignment.

Preview decoding uses up to 256 MB of audio memory (about 23 minutes of stereo sound). Longer recordings can still be exported, with alignment entered numerically. Playback requires an available audio output device. BPM changes move the musical grid; audio is not stretched.

## LRC lyrics and alignment (v1.2.2)

The **Lyrics** tab replaces the pitched-vocal screen. It opens or creates standard and enhanced LRC, provides editable timestamp/word rows and a raw-text editor, previews lyric markers over the recording waveform, and saves aligned LRC copies. Play/Pause, Stop, a seek slider, row-level **Seek** buttons and click-to-seek support reviewing synchronization. LRC editing and recording playback also work without a loaded Guitar Pro score. Pitched YARG vocals, vocal difficulty tags and `notes.mid` export have been removed.

**Normal LRC:** each nonfinal phrase ends exactly one chart tick before the next phrase starts, even across tempo changes or long gaps. The final phrase reaches the chart end; the chart extends when necessary to contain the final lyric. Multiple line timestamps and embedded `[offset:...]` values remain supported.

**Enhanced LRC:** words from a source line share one phrase and keep their written order. When a timestamp moves backward, the app finds the preceding valid lyric and the next later lyric, then spaces the intervening backward-timestamp run evenly between those two anchors. For example, times `10, 8, 7, 16` become `10, 12, 14, 16`. A later line can provide the next anchor. Runs with no following anchor, or too little time between anchors, show an actionable message so you can supply a valid later timestamp.

The tab prominently notifies you about repairs and lists each affected lyric's source line, original time and repaired time. Edited lyrics and repair notices persist in saved projects. Export also writes the changes to `conversion-report.json` and `LYRIC-REVIEW.txt`, and the export summary/status reports the repair count. You can adjust timestamps later in the row editor or raw LRC text. **Use repaired timestamps in editor** updates the editable copy while retaining review notices.

**Starting point:** seek to the first sung word and click **Align first lyric to playhead**, type the desired **First lyric at** time, or use the ±100 ms buttons/global lyric offset. Every lyric moves by the same amount; charted notes and audio keep their positions. The offset is saved with the project. Recording alignment is also added to the exported lyric timing, so padded/trimmed audio and lyrics stay together. Negative export times clamp to zero. `lyrics.lrc` and chart lyric events use the same repaired and adjusted timings.

**Save aligned LRC copy** includes the current lyric offset and recording alignment. Normal/enhanced timestamp style and artist/title metadata are preserved; embedded file offsets are folded into the saved timestamps. In-memory editing and song export preserve the original LRC file. Choosing the original filename in the save dialog would explicitly replace it; use a new filename for a separate copy.

For scores with embedded lyrics, **Make an LRC from Guitar Pro lyrics** imports a selected track/verse into the editor with current score timing and repeats. This is an editable timing snapshot: reimport after changing the score's BPM or bar position. It provides lyric text only, without generating pitched vocal gameplay.

Use **Try lyric demo**, or launch with `--demo-lyrics --page=5`, to inspect the included backward-timestamp example and practice alignment against a synthesized recording.

## Included

- Native Rust/egui interface: sidebar navigation, dark cards, mint accents, drag-and-drop import, and a playable five-lane note preview with a background waveform and shared alignment controls.
- GP3/4/5, GPX, GP7/8 import using the original alphaTab playback bridge, including repeats and tuplets.
- Named Pro Drums mapping, live collision counts and improvement feedback under the preview, highlighted collision positions, cymbal flags, Expert ghost/accent markers, optional velocity inference, strict collision checks, and automatic Expert double-kick alternation above 110 BPM.
- Guitar/Bass/Rhythm Rulebook v1.0 policy: phrase contour and repeated-motif mapping, faithful Expert attacks, hierarchical difficulty arrangements, stable chord shapes, correct strum/HOPO flip markers, Expert-only taps, and tempo-aware sustain release clearance.
- Pitched-instrument review warnings below the preview, with jump buttons, plus a full `CHARTING-REVIEW.txt` and structured findings in the conversion report.
- LRC manager with editable lyric timestamps/text, automatic backward-timestamp interpolation and persistent review notices, waveform playback, first-lyric alignment, and optional Guitar Pro-to-LRC import.
- Easy, Medium, Hard, and Expert generation, automatic Star Power, density-based 0–6 intensity estimates and overrides.
- Song properties, standard/enhanced LRC import, embedded lyric offsets, physical audio alignment, artwork, backgrounds, preview audio, stems, highways, icons, and color profiles.
- Unique song directories, staged export, conversion reports, and a silent practice WAV when audio is omitted.
- Searchable help and JSON projects.

## Scope and differences

The UI and conversion/export logic are Rust. Guitar Pro decoding intentionally retains the upstream JavaScript/alphaTab importer and its bundled Node runtime; this is not an all-Rust GP parser.

Guitar, Rhythm and Bass now follow the supplied [Charting Rulebook](CHARTING-RULEBOOK.md). See [Rulebook implementation](RULEBOOK-IMPLEMENTATION.md) for the exact automated policy and the musical decisions that still require listening. Easy uses three colored lanes and singles; Medium uses four colors and at most two-note chords; Hard uses all five colors without opens/taps; Expert retains all source attacks. Guitar Pro open strings remain colored notes. Purple bass notes require an explicit gameplay annotation on Expert, not a zero fret number.

This recreates the original workflow rather than promising identical output. The Rust fret-mapping, difficulty-reduction, intensity, lyric-phrase, and Star Power heuristics are independent implementations. They do not reproduce every special-case rule in the original Python algorithms. Enhanced LRC words share source-line phrases; backward timestamps are repaired by interpolation between valid lyric anchors. Timing comes from the Guitar Pro playback map; it is not automatically transcribed from the recording. These outputs are charting aids and require musical review. Intentional overlapping extended sustains and open chords are not generated automatically; individual chord-note releases are retained with conservative clearance before subsequent attacks. Aligned recordings and stems are converted to WAV; other media is copied intact. Custom assets under `Extras/Custom` require separate installation.

## Build from source

The GitHub source ZIP contains the Rust source, lockfile, importer, alphaTab, demos and license notices. It excludes compiled executables, build caches and test exports. Extract it and upload the contents of the `charting-toolkit` directory to your GitHub repository. The Windows release is provided separately; macOS and Linux releases are not included.

Install stable Rust and the platform's native build tools. On Windows, the standard MSVC Rust toolchain with Visual Studio C++ Build Tools is recommended.

```text
cargo test --locked
cargo build --release --locked
```

For source tests/builds, place Node.js and FFmpeg on PATH, or copy `vendor/node.exe` and `vendor/ffmpeg.exe` from the supplied Windows release into this source tree. The GitHub source ZIP excludes these large runtime binaries. Copy `target/release/charting-toolkit.exe` beside this README and rename it `ChartingToolkit.exe`. Keep `gp_bridge.cjs`, `vendor` and `examples` alongside it. The provided Windows release already includes all runtime files. Only the Windows build is validated here.

For the bundled end-to-end conversion check:

```text
ChartingToolkit.exe --smoke-test
```

This exports all three demos under `smoke-output`. `--demo` opens the guitar demo. `--demo-drums --demo-audio` opens the drum demo with a synthesized practice recording. `--capture` saves a preview screenshot using the application's own renderer and closes after capture. The optional native audio device check is `cargo test native_device_playback_clock_pause_and_seek -- --ignored` (silent playback).

## Licensing

Derived from Chart Starter by shytheprogrammer, under the MIT license retained in `LICENSE`. The alphaTab license is in `vendor/alphatab/LICENSE`; the Node license is in `vendor/NODE-LICENSE.txt`. The separate bundled FFmpeg executable is the Gyan.dev GPLv3 build `2025-08-25-git-1b62f9d3ae`; its license, source URL and build configuration are in `vendor/FFMPEG-LICENSE.txt` and `vendor/FFMPEG-BUILD-README.txt`. Rust dependency notices accompany the portable distribution under `licenses`, including Rodio/CPAL for native playback. The synthesized demo recordings is provided under this project's MIT license. The original help reference is retained as `ORIGINAL-GUIDE.md`; use the in-app Guide for behavior specific to this Rust edition.

## Preview controls (v1.2.2)

Extra media contains artwork, backgrounds, stems and other optional assets. All LRC file management, lyric editing and alignment lives in Lyrics. Attach the main recording in the pattern or lyric preview.

Both previews offer quarter-beat navigation and independent Zoom in, Zoom out and Reset zoom controls (1–64 beats). These viewing controls do not alter exported timing. The pattern preview also offers quarter-bar note shifts; these move every instrument and difficulty together and are included in exports.
