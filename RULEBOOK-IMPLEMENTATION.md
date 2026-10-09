# Rulebook v1.0 implementation

The supplied community authoring standard is applied to Guitar, Rhythm and Bass. Its spacing numbers are review recommendations, not game validity limits. Drum mapping retains its existing policy.

## Attacks, phrases and lanes

Guitar Pro playback provides real MIDI pitch, expanded repeat timing, tuplets, durations and technique flags. Tuning is handled by the importer, rather than treating a guitar fret number as a lane. Same-tick notes form one chord event; arpeggios and grace/brush attacks retain separate times.

Expert retains all distinct source attacks, including dense bursts. Phrases split at sections, actual rests and bounded musical windows. Small pitch palettes map consistently by rank; larger phrases use a look-ahead position optimizer to preserve direction, repeated attacks and manageable movement. Identical phrases with matching pitch/rhythm signatures share mappings. Power-chord octaves collapse to two-lane shapes; other full voicings use up to three lanes.

Lower difficulties are derived hierarchically without adding or moving attacks. Easy uses G/R/Y singles, choosing strong rhythmic anchors, accents and contour landmarks per beat. Medium uses G/R/Y/B and at most two-note chords, selecting eighth-note anchors. Hard keeps short fast figures and most attacks, selectively thinning sustained fast picked repetitions and chord streams of eight or more attacks. Hard uses at most two lanes per chord as a conservative default. Neither the 90 ms Hard review target nor the 60 ms Expert notice is a blanket deletion rule.

## Note types and sustains

Picked notes stay strums even when the engine would automatically turn them into HOPOs. `N 5` is emitted only to flip the calculated natural type; adding it to an already-natural HOPO would incorrectly force a strum. Explicit legato can remain HOPO on Medium/Hard/Expert when a different playable lane is available. Repeated source pitches stay strums. Written tapping is retained on Expert and simplified on lower tiers; fast timing alone never creates taps.

Open guitar/bass strings do not imply purple gameplay. Normal GP imports use colored notes. The optional Expert bass open mechanic requires an explicit source `chart_open` annotation. All lower tiers exclude open notes.

Audible release is approximated from source playback duration. Short muted/dead/staccato voices have no sustain. Individual chord voices retain different release lengths. Tails are capped before the next **retained** attack using the larger of the rulebook's beat-relative and real-time clearance. Awkward chord/lane changes receive preferred extra clearance. Tempo integration and conservative tick rounding determine the actual exported gap. A tail shorter than half a beat after trimming becomes an ordinary gem. Intentional overlapping extended sustains/open chords require manual authoring and playtesting.

## Timing and review

By default, source tempo changes and time signatures are exported in chronological order. Overall BPM scales the entire tempo map relative to its first tempo. The optional Constant BPM checkbox overrides those tempo changes with exactly one BPM marker at tick zero, using the entered value; time signatures remain intact. Chart shifts translate note events and source tempo/meter changes together; the constant BPM marker always stays at zero. Bar arrows use the source starting meter. Playback waveform positions, playhead seeking, LRC timing, release clearance, ratings and song length use the selected tempo policy.

The preview displays review findings for the selected instrument/difficulty. Export writes complete findings for all enabled pitched parts to `CHARTING-REVIEW.txt` and `conversion-report.json`, including measure:beat, tick, local BPM, milliseconds, pattern, severity and suggested fix. Findings cover dense attacks, possible staggered chords, fast wide/open/chord transitions and sustain clearance. Chord notes are grouped as one event during audits.

The application cannot establish audible attacks, correct transcription, tuning accuracy or intentional ringing solely by inspecting tablature. It does not infer a recording's tempo map or automatically snap ambiguous nearby notes. Listen against the recording, review flagged exceptions and playtest every difficulty in the intended game version, as the rulebook recommends.

Format reference used for HOPO flip semantics: [5-fret .chart format](https://github.com/TheNathannator/GuitarGame_ChartFormats/blob/main/docs/Chart-File-Formats/chart-format/Tracks/5-Fret-Guitar.md).
