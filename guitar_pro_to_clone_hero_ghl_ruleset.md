# Guitar Pro → Clone Hero: Authentic Guitar Hero Live Charting Specification

**Version 2.0 | Deterministic conversion | Six-fret GHL | Expert-first**

## 1. Design philosophy

Convert Guitar Pro tablature into charts that feel designed for Guitar Hero Live rather than mechanically translating guitar frets into buttons.

1. **Musical accuracy:** Preserve timing, articulation, rhythmic emphasis, and recognizable guitar parts.
2. **Six-fret ergonomics:** Favor natural three-finger shapes, intentional black/white transitions, and coherent patterns.
3. **Determinism:** Identical inputs and settings must produce identical outputs.
4. **Riff consistency:** Identical riffs must map identically; transposed riffs should generally retain recognizable patterns.
5. **Controlled complexity:** Retain meaningful difficulty without arbitrary or unreasonable finger movements.

Prioritize phrase recognition and chord-shape mapping over global pitch-to-lane lookup.

## 2. Controller model

The controller has three finger columns, each with an upper black and lower white button.

| Internal ID | Lane | Finger column |
|---|---|---|
| 0 | OPEN | None |
| 1 | W1 | Index |
| 2 | W2 | Middle |
| 3 | W3 | Ring |
| 4 | B1 | Index |
| 5 | B2 | Middle |
| 6 | B3 | Ring |

These are internal IDs, **not** exported chart note numbers.

**Default chord constraints:** Maximum three fretted buttons; at most one button per column; OPEN cannot coexist with a fretted button; prefer two-button shapes where musically adequate. Mixed black/white chords across different columns are allowed. Barre chords (black and white within one column) are forbidden by default but may be explicitly enabled and play-tested.

## 3. `.chart` six-fret lane encoding

Use the `[ExpertGHLGuitar]` section for Expert GHL guitar.

| Lane/modifier | `.chart` note number |
|---|---:|
| W1 | 0 |
| W2 | 1 |
| W3 | 2 |
| B1 | 3 |
| B2 | 4 |
| B3 | 8 |
| OPEN | 7 |
| HOPO/strum flip | 5 |
| Tap modifier | 6 |

**Important:** B3 uses `8`, not `5`. Note values `5` and `6` are modifiers. Verify the target chart format and player version before export.

## 4. Deterministic single-note mapping

### 4.1 Phrase segmentation

Split phrases using, in order:

1. Guitar Pro rehearsal/section markers.
2. Rests of at least one quarter-note beat.
3. Repeated-riff boundaries detected by rhythm and pitch similarity.
4. Four-measure groups as a fallback.

Never split tied notes, sustained bends, or continuous legato phrases arbitrarily.

### 4.2 Extracted event fields

```text
pitch_midi
string_number
fret_number
start_tick
duration_ticks
velocity
is_palm_muted
is_hammer_on
is_pull_off
is_slide
is_bend
is_tapped
is_dead_note
is_tied
```

Also calculate phrase-relative pitch, preceding melodic interval, and rhythmic position.

### 4.3 Fixed lane template library

| Template | Ascending lane sequence | Intended use |
|---|---|---|
| T1 | W1, W2, W3 | Simple three-note riffs |
| T2 | B1, B2, B3 | Upper-row riffs |
| T3 | W1, W2, W3, B3 | Four-note melodic phrases |
| T4 | W1, B1, W2, B2, W3, B3 | Six-note scalar patterns |
| T5 | W1, B2, W3 | Wide melodic jumps |
| T6 | W1, OPEN, W2, W3 | Pedal-tone riffs (special-case template) |
| T7 | W1, B1, W2, B2 | Alternating-row patterns |
| T8 | W1, W2, B2, B3 | Four-note ascending phrases |

Templates are proposed house-style conventions, not official GHL note mappings. T6 is an articulation/pedal-tone template and should not be interpreted as a strictly pitch-ordered sequence.

For each phrase, enumerate order-preserving assignments of distinct pitches to compatible templates. Reject templates with too few positions unless octave folding is enabled. Repeated pitches retain their lane within the phrase.

### 4.4 Candidate scoring

Minimize:

`C = 4M + 3R + 2J + 2E + 5I`

- `M`: Melodic contour violations.
- `R`: Unnecessary row transitions.
- `J`: Finger-column jumps exceeding one column.
- `E`: Ergonomic penalties such as rapid reversals.
- `I`: Inconsistency with previously mapped riff families.

Deterministic tie-breaking: (1) existing riff-family template, (2) fewer distinct lanes, (3) fewer row transitions, (4) lower template ID, (5) lexicographically smaller lane-ID sequence.

Use whole-phrase dynamic programming rather than independent note-by-note greedy selection.

## 5. Chord mapping

### 5.1 Normalize each source chord

1. Remove duplicate sounding pitches.
2. Identify bass and highest sounding pitch.
3. Compute intervals/pitch classes relative to the bass.
4. Classify power, major, minor, suspended, seventh, octave, or unknown.
5. Preserve inversion and voice-leading metadata.

### 5.2 Default chord dictionary

| Source chord class | Preferred GHL shape |
|---|---|
| Root + fifth | W1 + W2 |
| Root + octave | W1 + W3 |
| Major triad | W1 + W2 + W3 |
| Minor triad | B1 + W2 + W3 |
| Suspended | W1 + B2 + W3 |
| Seventh | W1 + W2 + B3 |
| Unknown dyad | W1 + W2 |
| Unknown triad or larger | W1 + W2 + W3 |

These are deterministic defaults, **not** official mappings. Apply the dictionary only where surrounding musical context does not suggest a better progression shape.

### 5.3 Progression-aware mapping

Do not map every power chord to the same shape when pitch movement matters. Example E5 → G5 → A5 → E5 could map to `W1+W2 → W2+W3 → B1+B2 → W1+W2` when appropriate.

Generate candidate shapes containing one to three buttons subject to ergonomic/barre policy. Minimize:

`C_chord = 5V + 4F + 3H + 2S + 6R`

- `V`: Voice-leading mismatch.
- `F`: Finger movement/shape-change difficulty.
- `H`: Harmonic identity mismatch.
- `S`: Unnecessary shape complexity.
- `R`: Inconsistency with previous occurrences.

Optimize across the chord progression rather than one chord at a time.

## 6. Repeated riffs

### 6.1 Fingerprinting

Fingerprint each phrase using:

```text
relative_note_onsets
note_durations
relative_pitch_intervals
chord_quality_sequence
articulation_flags
```

Maintain both an **exact fingerprint** (absolute pitches and original articulations) and a **transposition-invariant fingerprint** (relative intervals and rhythm).

### 6.2 Matching behavior

| Match | Action |
|---|---|
| Exact riff | Reuse lanes, chords, HOPOs, and sustain decisions |
| Transposed riff | Prefer existing shape; allow context-sensitive adjustment |
| Same riff, different ending | Preserve shared prefix; remap ending |
| Same rhythm, different melody | Preserve rhythmic feel; remap pitches |
| Same chord progression | Reuse established chord shapes |
| Repeated palm-muted riff | Preserve open-note/strum pattern |

Cache the canonical result:

```text
riff_cache[canonical_riff_fingerprint] = {
    lane_sequence,
    chord_shapes,
    articulation_mapping,
    sustain_policy
}
```

Find riff families **before** mapping, select the earliest occurrence as the canonical representative, and map it once. Exact matches reuse it; transposed matches treat it as a preferred candidate.

Priority: **Exact match → explicit override → riff family → local optimizer → default template**. Explicit overrides should be reconciled with exact-match locking by applying the override to the entire intended riff family, or marking the affected occurrence as a deliberate exception.

## 7. HOPO, strum, and tap conversion

| Guitar Pro event | Default output |
|---|---|
| Picked single note | STRUM |
| Hammer-on | HOPO |
| Pull-off | HOPO |
| Legato slide | HOPO if distinct target note |
| Picked slide | STRUM unless marked legato |
| Two-hand tapping | TAP |
| Tremolo picking | STRUM for each attack |
| Repeated same pitch/lane | STRUM |
| Palm-muted note | STRUM |
| Chord change | STRUM |
| Grace note | HOPO if clearly legato |
| Trill | Alternating HOPOs if appropriate |

### 7.1 Natural HOPO eligibility

```text
previous_note_exists
AND current_event_is_single_note
AND previous_event_is_single_note
AND current_lane != previous_lane
AND timing_gap <= hopo_threshold
AND no_explicit_repick
```

Default `hopo_threshold_beats = 0.5` quarter-note beats, as a converter policy, not an assertion about the game's universal HOPO threshold.

### 7.2 Articulation precedence

1. Manual override.
2. Explicit tapping.
3. Explicit hammer-on, pull-off, or legato slide.
4. Explicit picking/tremolo.
5. Natural HOPO eligibility.
6. STRUM fallback.

Determine natural engine behavior before emitting a HOPO/strum flip modifier. OPEN notes can be strums, HOPOs, or taps; repeated open pedal tones default to STRUM.

## 8. Sustain conversion

| Setting | Default |
|---|---:|
| Minimum sustain | 0.5 quarter-note beats |
| Release gap | 0.125 beats |
| Incompatible overlap allowed | 0 beats |
| Short-note cutoff | 0.25 beats |

Algorithm:

1. Combine tied notes.
2. Determine sounding duration.
3. Detect staccato, palm mute, and repicking.
4. Suppress sustains shorter than the minimum.
5. Find the next incompatible event.
6. Trim before that event by the release gap.
7. Preserve the sustain only if enough duration remains.

For incompatible next events:

`L_sustain = max(0, min(D, T_next - T_start - G))`

Where `D` is original sounding duration and `G` is release gap. Preserve compatible anchored sustains only where the target format supports them.

Special cases: bends/vibrato become sustained notes; palm-muted chugs and tremolo picking remain separate strums; tied chords become continuous chord sustains; incompatible chord changes terminate held notes.

## 9. Open-note selection

Open guitar strings do **not** automatically become GHL OPEN notes.

Choose OPEN for an event when:

```text
event_is_single_note
AND phrase_contains_repeated_low_pedal_tone
AND note_is_phrase_pedal_pitch
AND note_is_explicitly_picked
AND no_required_simultaneous_fretted_note
```

Qualifying pedal tones must:

- Occur at least three times in the phrase.
- Account for at least 30% of note attacks.
- Be the lowest recurring pitch.
- Alternate with higher pitches or form a repeated chugging figure.

Example: `E2 E2 G2 E2 A2 E2 G2 A2` → `OPEN OPEN W1 OPEN W2 OPEN W1 W2`.

An explicit user override can require a fretted pedal tone instead.

## 10. Conversion architecture

```python
def convert_guitar_pro_to_ghl(gp_file, config):
    song = parse_guitar_pro(gp_file)
    tempo_map = extract_tempo_map(song)
    events = extract_guitar_events(song, config.track)

    events = normalize_ties(events)
    events = group_simultaneous_notes(events)
    events = normalize_articulations(events)

    phrases = segment_phrases(events)
    riff_families = detect_riff_families(phrases)
    representatives = select_canonical_riffs(riff_families)

    mapped_riffs = {}
    for riff in representatives:
        candidates = generate_phrase_candidates(
            riff, config.templates, config.barre_policy
        )
        mapped_riffs[riff.id] = optimize_phrase(
            candidates,
            chord_dictionary=config.chord_dictionary,
            cost_weights=config.cost_weights,
            deterministic_tiebreak=True
        )

    chart_events = []
    for phrase in phrases:
        mapping = apply_riff_family_mapping(
            phrase, riff_families, mapped_riffs
        )
        mapping = apply_explicit_overrides(mapping, config.overrides)
        chart_events.extend(mapping)

    chart_events = assign_open_notes(chart_events, config)
    chart_events = resolve_hopo_and_taps(chart_events, config)
    chart_events = calculate_sustains(chart_events, config)

    validate_timing(chart_events, tempo_map)
    validate_ergonomics(chart_events, config.barre_policy)
    validate_repeated_riffs(chart_events, riff_families)

    return export_ghl_chart(
        chart_events, tempo_map, resolution=config.resolution
    )
```

This is architecture-level pseudocode; helper functions must be implemented. Open-note decisions should be integrated into phrase candidate evaluation or followed by a consistency pass so they cannot silently invalidate cached riff mappings.

### 10.1 Recommended JSON configuration

```json
{
  "ruleset": "authentic_ghl_v2",
  "difficulty": "expert",
  "track": "lead_guitar",
  "resolution": 192,
  "barre_policy": "forbid",
  "max_fretted_buttons": 3,
  "phrase_fallback_measures": 4,
  "rest_split_beats": 1.0,
  "hopo_threshold_beats": 0.5,
  "sustain_min_beats": 0.5,
  "sustain_release_gap_beats": 0.125,
  "short_note_cutoff_beats": 0.25,
  "open_pedal_min_occurrences": 3,
  "open_pedal_min_ratio": 0.3,
  "preserve_exact_riffs": true,
  "reuse_transposed_riffs": true,
  "deterministic_tiebreak": true,
  "random_seed": null,
  "cost_weights": {
    "melodic_contour": 4,
    "row_transition": 3,
    "finger_jump": 2,
    "ergonomic_penalty": 2,
    "riff_inconsistency": 5
  }
}
```

## 11. `.chart` serialization example

```ini
[Song]
{
  Name = "Example"
  Artist = "Example Artist"
  Charter = "Guitar Pro GHL Converter"
  Resolution = 192
}
[SyncTrack]
{
  0 = TS 4
  0 = B 120000
}
[Events]
{
}
[ExpertGHLGuitar]
{
  0 = N 7 0
  96 = N 0 0
  192 = N 1 96
  384 = N 0 0
  384 = N 1 0
  576 = N 8 0
}
```

This illustrates OPEN, W1, W2, a two-button chord, and B3. `B 120000` means 120 BPM. Real song packages also require suitable audio and metadata.

## 12. Validation and acceptance tests

| Test | Expected result |
|---|---|
| Convert same input twice | Byte-identical output |
| Exact repeated riff | Identical lanes and modifiers |
| Transposed riff | Consistent recognizable shape |
| Six-note ascending run | Predictable ordered lane sequence |
| Power-chord progression | Playable coherent shapes |
| Rapid repeated note | Separate strums |
| Hammer-on sequence | HOPOs as intended |
| Tapping passage | Tap modifiers |
| Long tied note | One sustain |
| Tempo change | No synchronization drift |
| Barre forbidden | No simultaneous same-column buttons |
| Export round-trip | Preserved note positions, durations, and lanes |

Compare the re-parsed exported chart against the intermediate event representation, not only the output text.

## 13. Final rule precedence

1. Timing and synchronization.
2. Correct note and chord rhythm.
3. Repeated-riff consistency.
4. Physical controller playability.
5. Musical contour and harmonic identity.
6. HOPO and articulation authenticity.
7. Visual pattern variety.

**Implementation recommendation:** Separate the Guitar Pro parser, intermediate note-event model, deterministic phrase optimizer, and six-fret `.chart` serializer.

This specification is an **authentic-style house standard**, not an assertion that its musical heuristics reproduce the proprietary charting decisions of the original game.

## References

- [Guitar Game Chart Formats](https://github.com/TheNathannator/GuitarGame_ChartFormats) — format reference for `.chart` and MIDI conventions.
- [Clone Hero](https://clonehero.net/) — target game and compatibility context.
