# Validation — v1.2.1

Validated on Windows, 9 October 2026.

- Stable Rust Windows GNU release build succeeded.
- Thirty-nine automated tests passed; one optional native-device playback test remains excluded by default.
- Standard LRC phrase ends were verified as exactly one tick before the next start across source tempo changes and long gaps.
- Enhanced backward runs were verified to preserve written word order and interpolate evenly between valid anchors, including anchors on a following source line. Missing anchors and insufficient interpolation intervals produce actionable messages.
- Export checks verified shared enhanced-line phrases, edited/repaired lyric timings, normal/enhanced LRC output, persistent repair history after editor normalization/project reload, LYRIC-REVIEW.txt and conversion report records, and no pitched-vocal MIDI/difficulty output.
- An actual rendered UI pointer test clicked Align first lyric to playhead and +100 ms. The first lyric moved to the requested playback time; charted note positions and recording alignment did not change. Repair notifications were rendered and retained.
- Existing drum mapping/collision, guitar/bass/rhythm rulebook, tempo override, shared note-shift, audio padding/trimming, waveform, lyric offset and saved-project checks continue to pass.
- Compiled demo exports and the new Lyrics screen were additionally checked using the native Windows build.

Clone Hero/Moonscraper in-game playtesting and audible listening were not performed. Review repaired lyrics and chart timing against your recording before final use.
