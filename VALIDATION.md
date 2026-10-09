# Validation

Validated on Windows, 9 October 2026.

- Release build succeeded with the stable Rust Windows GNU toolchain.
- Thirty-one automated tests passed, covering the previous conversion/mapping/BPM checks plus waveform bounds, stereo playback padding/trimming/seeking, sample-accurate FFmpeg audio output, identical note/marker shifts across all four instruments and difficulties, section and Star Power translation, time-zero limits, and saved-project compatibility.
- Constant BPM checks verify the checkbox beside the number toggles both ways, exactly one BPM flag is written at tick zero with the entered value (including after shifting notes), unchecked restores source tempo changes, signatures remain intact, playback/lyrics/ratings use the chosen mode, and saved projects retain it while older projects default to unchecked.
- Rulebook tests verified dense Expert attack preservation, repeated-pitch strums, correct HOPO flip semantics in both directions, no automatic opens from zero-fret strings, Expert-only explicit opens/taps, tier lane/chord restrictions, hierarchical attack subsets, short Hard bursts versus long picked streams, repeated phrases/chord shapes, position resets for long pitch runs, individual chord release lengths, retained-next-attack sustain trimming, and real-time clearance across tempo changes.
- Timing/export tests verified source tempo and time-signature changes, proportional overall BPM scaling, integrated lyric timing, chronological sync markers, shared shifts in a 3/4 source, and complete audit sections for all pitched instruments/difficulties. The rendered pitched preview test confirms dense Expert notes remain present while review notices are shown.
- The rendered preview test also exercises actual pointer input: a waveform drag across multiple frames, holding the pointer still, releasing it, and clicking the one-bar arrow. Alignment remains stable and the preview notes move by the shared bar offset.
- Full export tests verified positive and negative audio alignment, original sample rate/channels, matching main/stem WAV duration, untouched preview excerpts and original files, aligned LRC lyrics, zero exported delay, conversion reports, and cleanup after an adjustment that trims away the recording.
- The optional native audio device test was run separately and passed: silent playback clock progression, pause stability, seeking with changed alignment, and stopping.
- Both bundled Guitar Pro scores imported and exported successfully through the compiled application.
- Demo export checks verified all enabled instrument/difficulty sections and valid, nonempty WAV audio.
- The guitar demo additionally exported standard/enhanced LRC content and chart lyric events.
- A repeated export created a uniquely named folder rather than replacing the previous export.
- All five interface pages were previously rendered successfully using the compiled native application. Updated guitar/review and drum/waveform previews were additionally rendered and visually inspected. Demo exports include `CHARTING-REVIEW.txt` and structured findings for every enabled pitched part/difficulty.

Clone Hero and Moonscraper playtesting was not performed in this environment. Audio device testing used silent samples; audible listening was not performed. Exported charts require musical review, as explained in the README and in-app Guide.
