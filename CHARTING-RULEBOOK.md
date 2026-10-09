# Clone Hero Guitar Charting Rulebook

**Version:** 1.0  
**Scope:** Converting six-string guitar tablature and recordings into five-fret Clone Hero guitar charts, including Easy, Medium, Hard, and Expert.  
**Status:** Community-informed **recommended authoring standard**, not an official Clone Hero ruleset. All numerical spacing targets below are *design recommendations*, **not engine-enforced minimums**.

## 1. Guiding principles

1. **Time the audio, not the appearance of ASCII tab.** A plain-text tab usually lacks trustworthy rhythmic durations.
2. Preserve audible attacks, rests, phrase structure, and distinctive riffs.
3. Keep repeated pitches, riffs, and chord shapes consistent within their musical context.
4. Map **relative pitch** and melodic contour to five lanes; do not directly map guitar fret numbers to lanes.
5. Represent picking versus legato using strums, HOPOs, and taps as appropriate.
6. Do not invent notes to fill empty space or inflate difficulty.
7. Make lower difficulties genuine, progressively simpler arrangements, not merely randomly deleted notes.
8. Check feasibility at the **actual tempo**; a 1/16-note gap is very different at 90 BPM and 240 BPM.
9. Playtest every difficulty, especially fast same-lane strums, wide jumps, chord changes, and sustain overlaps.
10. When faithfulness and accessibility conflict, retain musical identity and simplify *only as needed for the selected difficulty*.

## 2. Understanding guitar tab and pitch

Standard tuning (low to high): E2 (MIDI 40), A2 (45), D3 (50), G3 (55), B3 (59), E4 (64). A fretted pitch is `open-string MIDI note + fret number`. Adjust open pitches for alternate tunings (e.g., Drop D: low D2 = 38).

```text
 e|--5--7--8--10--12--|
 B|--------------------|
 G|--------------------|
 D|--------------------|
 A|--------------------|
 E|--------------------|
```

An ascending five-note phrase can map `G R Y B O`. However, a fixed guitar pitch is **not** permanently assigned to a fixed lane: the phrase, surrounding register, and repeated motifs matter. A note at fret 5 on a high string can be much higher than a note at fret 7 on a low string.

**Tab parsing:** Detect simultaneous notes in aligned columns as potential chords; parse `h`, `p`, `/`, `\`, `b`, `r`, `~`, `x`, `PM`, and `let ring` as technique clues. Confirm what is actually audible. Guitar tablature often omits precise attacks and release times.

## 3. Fret lanes and note types

| Symbol | Meaning | Charting guidance |
|---|---|---|
| `G` | Green | Lowest colored lane |
| `R` | Red | Second lane |
| `Y` | Yellow | Middle lane |
| `B` | Blue | Fourth lane |
| `O` | Orange | Highest colored lane |
| `P` | Open / purple | Optional gameplay note type; **not** the same thing as an open guitar string |
| `G+R` | Chord | Simultaneous fretted lanes |
| `G~` | Sustain | Hold the note for its audible duration |
| `R(h)` | HOPO | Hammer-on/pull-off playable without a new strum when eligible |
| `Y(t)` | Tap | Tap note, generally does not require strumming |
| `B(s)` | Strum | New pick attack / strum-required note |

**Implementation caution:** Automatic HOPO classification is affected by chart timing and settings. Force HOPO/strum where the desired behavior is not obtained automatically. Tap, open, open-chord, and extended-sustain support can vary by game version and chart format; test the exported file in the intended game version. Open guitar strings may be mapped to normal colored frets.

## 4. Rhythmic timing, grids, and spacing

### 4.1 The key distinction: engine validity vs. playability

**There is no verified universal Clone Hero engine minimum of, for example, 50 ms or 1/16 beat between all notes.** Very dense charts can exist; whether a pattern is practical depends on tempo, strumming technique, lane changes, controller, and player skill. Do **not** treat the recommended gaps below as parser limits, hit-window values, or official requirements.

For two consecutive **distinct attacks**, define:

- `delta_beats = beat_position(next) - beat_position(current)`
- `delta_ms = delta_beats × 60,000 / BPM` for a constant-tempo segment.
- At a tempo change, compute time by integrating over the tempo map, not by applying one BPM to the entire interval.
- Simultaneous notes with the same onset form **one chord event**: they are **not** a zero-gap collision.

At constant tempo, the interval between subdivisions is `60,000 × 4 / (BPM × denominator)` milliseconds for a `1/denominator` whole-note grid.

| BPM | 1/4 note | 1/8 note | 1/16 note | 1/32 note | 1/64 note |
|---:|---:|---:|---:|---:|---:|
| 90 | 667 ms | 333 ms | 167 ms | 83 ms | 42 ms |
| 120 | 500 ms | 250 ms | 125 ms | 63 ms | 31 ms |
| 160 | 375 ms | 188 ms | 94 ms | 47 ms | 23 ms |
| 200 | 300 ms | 150 ms | 75 ms | 38 ms | 19 ms |
| 240 | 250 ms | 125 ms | 63 ms | 31 ms | 16 ms |

Rounded to the nearest millisecond. These values are mathematical intervals, **not recommended minimums**.

### 4.2 Recommended *default* minimum gaps by difficulty

The following are **conservative authoring targets for ordinary accessible charts**, not absolute limits. They apply to adjacent **distinct attacks** (not notes inside the same chord). Preserve musically meaningful syncopation, triplets, and short bursts where appropriate.

| Difficulty | Typical minimum grid for sustained runs | Suggested default adjacent-onset floor | Exceptions / caveats |
|---|---|---:|---|
| **Easy** | Mostly quarter notes | **~250 ms** | Prefer >= one beat at moderate BPM; occasional eighths when slow and simple |
| **Medium** | Mostly eighth notes | **~150 ms** | Short quicker figures may be appropriate if no difficult jumps or chords |
| **Hard** | Eighths and sixteenths | **~90 ms** | Faster passages may be retained when manageable and musically central |
| **Expert** | As required by the recording | **No universal floor** | Preserve real attacks; scrutinize dense bursts below ~60 ms and do not invent extra notes |

**How to use the table:** These are *review thresholds*, not automatic quantization or deletion rules. For instance, at 200 BPM an eighth note is 150 ms and a sixteenth is 75 ms. A Hard chart can reasonably contain 75 ms sixteenths in a manageable phrase; the ~90 ms threshold simply prompts a playability review. At 90 BPM, an Easy eighth is 333 ms and may be comfortable even though Easy normally emphasizes quarters.

### 4.3 Sustain-tail-to-next-note clearance (the important spacing rule)

**This is different from onset-to-onset spacing.** The purpose of a sustain-tail clearance is to ensure the player can hold the full scored sustain and still release or change frets in time to hit the following note.

Define:

- `next_onset`: beat/tick position of the following note or chord.
- `sustain_end`: beat/tick position where the previous sustain tail finishes.
- **`clearance = next_onset - sustain_end`** (measure using the tempo map; a negative value indicates overlap).
- `onset_gap = next_onset - previous_onset` is **not** a substitute for clearance.

**Default authoring recommendation:** For ordinary, non-overlapping sustains, leave **at least 1/16 of a beat** between the sustain tail and the next note, with **1/8 of a beat preferred** for clear, comfortable release/regrip. These are **fractions of a quarter-note beat**, not sixteenth/eighth *notes*: at 120 BPM, 1/16 beat = 31.25 ms and 1/8 beat = 62.5 ms. A 1/16-beat clearance is a tight visual/technical buffer and may be too small for an actual difficult hand change. No universal engine-enforced clearance is asserted here.

For **practical playability**, use the larger of a beat-relative clearance and an appropriate real-time release/regrip target:

| Difficulty | Suggested minimum clearance from sustain end to next onset | Preferred for harder chord/lane changes |
|---|---|---|
| Easy | **1/4 beat or ~100 ms**, whichever is greater | ~150 ms |
| Medium | **1/8 beat or ~70 ms**, whichever is greater | ~100 ms |
| Hard | **1/8 beat or ~50 ms**, whichever is greater | ~75 ms |
| Expert | **1/16 beat or ~30 ms**, whichever is greater | ~50–75 ms for awkward changes |

These are **proposed conservative chart-design targets**, not verified official Clone Hero scoring thresholds or mandatory gaps. The actual required physical clearance varies with controller, fretting, chord shape, player, and whether the following note can be prepared while holding the sustain. A chart can legitimately use a smaller gap or overlapping sustains when the intended mechanic is supported and playtested.

**Example at 120 BPM (one quarter-note beat = 500 ms):** A note starts at beat 1.0 and the next starts at beat 2.0. For Expert, ending the sustain at beat 1.9375 leaves 1/16 beat = 31.25 ms. Ending at beat 1.875 leaves 1/8 beat = 62.5 ms. The latter is generally safer for a required fret release.

**Rules for sustain clearance:**

1. For a **same-lane re-strum**, end the previous sustain before the new attack; do not force a hold through the next gem unless deliberately using a tested supported mechanic.
2. For **different-lane notes or chords requiring a grip change**, allow enough time to release and reposition; the default Expert 1/16-beat gap may be insufficient.
3. For **simultaneous new notes while a previous lane remains held** (extended/disjoint sustains), do not automatically shorten the sustain: verify that the overlap is intended, supported, and playable.
4. If the audio rings right up to the next attack but a full-length chart sustain would be awkward, shorten the **charted** tail slightly while preserving the note's attack and musical feel.
5. Do not add a sustain to a note that becomes so short after applying the gap that it no longer offers a meaningful hold; use a normal gem instead.
6. Quantize sustain ends to an appropriate tick/grid resolution and check the **actual exported end time**; rounding can accidentally erase the gap.
7. At tempo changes, compute the real-time clearance across the tempo map, not with one fixed BPM.
8. Playtest whether the full sustain can be held **until its tail ends**, followed by a clean next hit; if the player must release early, shorten the tail or revise the pattern.

**Automated audit:** For every sustained event and its next distinct onset, calculate `clearance_ms = time_ms(next_onset) - time_ms(sustain_end)`. Flag clearance below the chosen difficulty target; flag unintended negative clearance as overlap. Also flag chord changes requiring multiple finger releases even when the nominal gap passes. Treat intentionally supported extended sustains separately.

### 4.4 Additional spacing and collision guidelines

1. **Same-lane repeated strums:** Every audible pick attack gets a separate event on Expert. For Easy/Medium, thin rapid repeated attacks to a regular, musically meaningful pulse; do not force alternate strumming.
2. **Different-lane HOPOs:** Consecutive notes can be very close if the phrase is actually legato; still verify the note-type behavior and physical playability.
3. **Rapid chord changes:** Demand more recovery time than a comparable single-note run at lower difficulties. Do not stack difficult three-lane chord changes at a speed intended for simple singles.
4. **Large lane jumps:** At short gaps, reduce unnecessary `G↔O` leaps on lower difficulties or choose a more coherent position mapping.
5. **Open-to-fretted transitions:** Treat them as genuine release/regrip demands; do not assume they are as easy as neighboring colored notes.
6. **Simultaneous onset:** Multiple notes at exactly the same beat are one chord; never stagger a chord by a few milliseconds merely to avoid a fictitious minimum gap.
7. **Accidental near-duplicates:** If two intended-to-be-simultaneous notes are offset by a few ticks through conversion or editing, snap them to one chord onset after verifying the audio.
8. **Sustain overlaps:** Avoid a sustain running into a subsequent note on the same lane unless the target chart format/game supports the intended behavior. Do not let long tails obscure new attacks.
9. **Release gaps:** Use the sustain-end-to-next-onset clearance targets in §4.3. Do not confuse this with spacing between note attacks.
10. **Triplets and swing:** Preserve the proper rhythmic ratios; never round all triplets onto a straight sixteenth grid simply to satisfy a preferred subdivision.
11. **Tempo changes:** Recheck absolute milliseconds after each tempo marker; beat-based spacing alone can conceal abrupt changes in physical demand.
12. **Dense Expert passages:** Distinguish a genuinely fast recording from transcription artifacts, doubled attacks, delay echoes, or overlapping guitar parts.

### 4.5 Suggested automated spacing audit

For each difficulty, sort note **events** by onset, grouping all same-onset lanes into one chord. Calculate the gap to the next event in real milliseconds using the tempo map. Flag, but do not automatically delete, any event pair below the difficulty's suggested floor. Add warnings for:

- Two consecutive three-note chords on Medium/Easy (which should ordinarily not exist).
- Rapid wide fret jumps and open-to-chord transitions.
- Repeated same-lane notes incorrectly marked as effortless HOPOs.
- Two distinct events separated by only a tiny conversion offset when the source indicates one chord.
- Sustains overlapping incompatible later notes or crossing into silence.
- Short notes whose charted sustain has negligible meaningful hold time.

A good linter outputs `measure:beat`, `delta_ms`, `BPM`, `pattern`, `severity`, and `suggested fix`. The chart author makes the final decision.

## 5. Common Clone Hero note patterns

**Notation:** spaces indicate sequence, `+` means a simultaneous chord, `(h)` HOPO, `(t)` tap, `~` sustain, `P` open. The sequences below are **shapes**, not fixed timings or difficulty mandates.

| Pattern | Example | Musical/gameplay use | Charting caution |
|---|---|---|---|
| Repeated notes / jack | `G G G G` | Repeated picking | Each audible attack is separate; avoid accidental HOPO |
| Alternate strumming stream | `R R R R R R` | Rapid tremolo/palm-muted picking | Thin on lower tiers |
| Two-note trill | `R Y R Y R Y` | Repeated alternation | Often HOPO when legato; check strum articulation |
| Ascending run | `G R Y B O` | Rising scalar passage | Keep pitch direction |
| Descending run | `O B Y R G` | Falling scalar passage | Keep pitch direction |
| Three-note triplet figure | `G R Y` | Three-note run/grouping | 'Triplet' may mean rhythm or three-note grouping; confirm timing |
| Quad run | `G R Y B` | Four-note run | Do not assume all quads are rhythmically 1/16 |
| Quint run | `G R Y B O` | Five-note run | Re-map if phrase extends beyond five lanes |
| Zig / zigzag | `G R Y R G R Y` | Back-and-forth contour | Keep turning points consistent |
| Reverse zig | `Y R G R Y R G` | Inverted zig | Avoid unnecessary fret reset |
| Chimney | `Y R G R Y R G` | Zig-derived shape with high-end lead-in | Names vary by charting community |
| Reverse chimney | `G R Y R G R Y` | Zig-derived shape with low-end lead-in | Treat as descriptive, not a separate engine type |
| Ladder / staircase | `G R R Y Y B B O` | Repeated stepped ascent | Preserve repeated attacks |
| Anchor pattern | `G R G Y G B` | Low anchor alternating with upper notes | Verify whether the lower fret is held or re-attacked |
| Gallop | `G G G  G G G` | Long-short-short / grouped strumming | Time from recording, not text spacing |
| Reverse gallop | `G G G  G G G` | Short-short-long rhythm | Same symbols as gallop; **rhythmic durations differ** |
| Burst | `G R Y B` | Short fast run between slower notes | Leave correct rest/recovery afterward |
| Chord pulse | `G+R  G+R  G+R` | Repeated power chords | Each chord has one onset |
| Chord change | `G+R  R+Y  Y+B` | Moving harmony | Maintain coherent voicing shape |
| Chord-to-single | `G+R  Y  B` | Riff/chord accent | Don't turn arpeggio into chord |
| Single-to-chord | `G  R+Y` | Pickup into chord | Keep the true onset |
| Chord sustain | `G+R~` | Ringing power chord | Duration follows audio |
| Disjointed chord sustain | `G~ + R` | Different chord-note sustain lengths | Verify format/game support |
| Extended sustain | `G~  Y  B` | New notes while holding earlier note | Test game support and controller feasibility |
| HOPO chain | `G(s) R(h) Y(h) B(h)` | Legato run | Force note types if auto classification differs |
| Tap run | `G(t) Y(t) B(t)` | Two-hand tapping | Usually Expert-only by policy |
| Open-note riff | `P P R P Y` | Explicit open-note gameplay | Do not equate with open guitar strings |
| Open chord | `P+Y` | Open and fretted simultaneously | Version-dependent; verify target |
| Syncopation | `G  [rest] R  Y` | Off-beat accents | Preserve the real off-beat positions |
| Chromatic climb | `G R Y B O` | Consecutive semitones | Reposition logically when longer than five notes |
| Octave leap | `G O G O` | Large pitch movement | Avoid overusing full-span jumps |
| Tremolo picking | `Y Y Y Y Y Y` | Fast repeated pick attacks | Strum-required unless intended otherwise |
| Arpeggio | `G Y B Y` | Chord tones played separately | Do not collapse into a simultaneous chord |

**Terminology note:** The official Clone Hero Wiki documents common terms such as trills, triplets, quads, quints, zigs, chimneys, and ladders. Names can be used differently in different communities; use the examples to communicate the *shape*, and use audio/tempo to specify *timing*.

## 6. Techniques from guitar tab

| Tab marking | Meaning | Recommended chart representation |
|---|---|---|
| `5h7` | Hammer-on | Pick first note, HOPO second when appropriate |
| `7p5` | Pull-off | HOPO for second note when appropriate |
| `5/7` or `7\5` | Slide | Change lane with audible destination; avoid fabricated in-between attacks |
| `7b9` | Bend | Usually one held note; chart new event only for a distinct reattack |
| `9r7` | Bend release | May remain one sustain unless separately attacked |
| `~` | Vibrato | Sustain when audibly held; don't invent oscillating gems |
| `PM----` | Palm mute | Short, separated attacks; typically strummed |
| `x` | Muted/percussive note | Chart if an intentional distinct guitar attack |
| `t` | Tapping | Tap note when musically appropriate and target format supports it |
| `let ring` | Sustained ringing | Sustain to audible release, considering subsequent notes |
| `<12>` | Harmonic | Usually ordinary pitched note; map relative pitch |

**Picked vs. legato:** A fast line is not automatically a tap phrase. Same-pitch repeated notes normally require separate strums. Forced strums can be appropriate even when notes are close enough for automatic HOPOs.

## 7. Chord mapping rules

- Simultaneously struck guitar strings map to **one simultaneous Clone Hero event**, usually two or three colored lanes on Expert.
- A two- or three-string power chord often maps to a two-note shape such as `G+R` or `R+Y`.
- Keep recurring chord voicings and transpositions consistent; a chord change should look and feel intentional.
- An arpeggio is a sequence, not a chord, even if the tab shows notes from one harmony.
- Avoid forcing every six-string chord into five lanes; simplify according to the audible attack and difficulty.
- For Easy, prefer singles; for Medium, allow selected two-note chords; for Hard, allow two-note chords and occasional three-note chords; for Expert, use the voicing complexity that best represents the performance without arbitrary overcharting.
- Open notes and open chords are **optional special mechanics**, not direct translations of an open string or an ordinary open-position guitar chord.

## 8. Difficulty-by-difficulty charting guidelines

### 8.1 Easy — introductory play

**Lane set:** `G R Y` (three colors). **No Blue, Orange, or open notes** under this recommended traditional scheme.

- Use mainly single notes; chords should be exceptional or omitted.
- Prefer quarter-note beats and strong rhythmic anchors; occasional simple eighths are fine when comfortably spaced.
- Target a default minimum of **~250 ms** between distinct attacks, and more for complex transitions.
- Avoid rapid repeated strums, alternate-strumming requirements, dense HOPO chains, taps, trills, fast zigzags, and large jumps.
- Make repeated phrases visually and physically predictable.
- Preserve the signature rhythm by selecting essential downbeats and riff landmarks rather than shifting them.
- Prefer simple `G R Y` contour mapping and limited fret movement.
- Sustains can remain when they teach holding notes without interfering with following attacks.
- Playtest for one-handed novice-friendly fingering.

### 8.2 Medium — basic rhythmic and fret development

**Lane set:** `G R Y B` (four colors). **No Orange or open notes** under this recommended traditional scheme.

- Mostly singles, with occasional two-note chords; avoid three-note chords.
- Eighth notes are normal; preserve some syncopation and moderate runs.
- Target a default minimum of **~150 ms** between distinct attacks; review any denser figure.
- Do not require sustained fast alternate strumming; simplify gallops and tremolo streams.
- Introduce short, manageable HOPO passages if they clearly reflect legato playing; do not require tapping.
- Avoid repeated rapid full-range shifts or awkward chord-to-single changes.
- Keep familiar riffs recognizably similar to Hard/Expert, with notes removed rather than shifted off their real attacks.
- Prefer stable chord shapes and modest position changes.

### 8.3 Hard — near-complete musical representation

**Lane set:** `G R Y B O` (all five colored lanes). Prefer **no open notes or required taps** for a conventional Hard tier.

- Preserve most meaningful riff attacks, chord accents, and melody contours.
- Eighths and sixteenths are normal; thin especially demanding repeated-strum streams.
- Use **~90 ms** as a *review threshold*, not a strict floor: musically essential 75 ms passages can be appropriate.
- Two-note chords are common; three-note chords may occur but should not dominate dense runs.
- HOPOs should follow legato technique and remain predictable; simplify complex tapping figures into ordinary notes/HOPOs.
- Retain recognizable gallops, short trills, zigs, and ladders while reducing the most punishing density.
- Avoid stacked difficult transitions (e.g., full-span jump immediately into a fast three-note chord change) unless truly warranted.
- Test at song speed, not only at slowed practice speed.

### 8.4 Expert — complete, faithful chart

**Lane set:** All five colored lanes; optional open notes and taps when suitable and supported.

- Chart nearly all distinct audible guitar attacks from the selected part; do not add imaginary attacks.
- Keep the actual tempo map, subdivisions, syncopation, triplets, and articulated strums.
- Allow dense sixteenths, thirty-seconds, fast HOPOs, tapping, tremolo picking, and complex chords **only where the music supports them**.
- **No blanket minimum inter-note gap.** Inspect extremely short intervals (e.g., under ~60 ms) for transcription mistakes, stacked tracks, accidental duplicates, and impractical combinations.
- Same-pitch repeated picks remain distinct strum events; legato passages should not be over-strummed.
- Map fast solos by coherent phrase contour and sensible fret-position shifts.
- Use extended/disjointed sustains only when they reflect the music and work in the target game.
- Expert may be very difficult, but must not be artificially harder than the performance implies.

### 8.5 Side-by-side difficulty matrix

| Property | Easy | Medium | Hard | Expert |
|---|---|---|---|---|
| Recommended lanes | G/R/Y | G/R/Y/B | G/R/Y/B/O | All five + optional open |
| Typical note grid | Quarter | Eighth | Eighth/sixteenth | As recorded |
| Default gap review | ~250 ms | ~150 ms | ~90 ms | No fixed floor; review <~60 ms |
| Chords | Usually none | Up to two notes | Mostly two, occasional three | Musically appropriate |
| HOPOs | Rare/optional | Short simple passages | Common where appropriate | Faithful to technique |
| Taps | No | No | Normally no | When appropriate |
| Fast alternate strum | Avoid | Usually simplify | Selectively retain | Preserve real attacks |
| Wide jumps | Minimize | Moderate | Allowed | As musically justified |
| Primary goal | Accessibility | Development | Near-full arrangement | Accuracy |

**Not official requirements:** This matrix is a deliberately conservative, conventional authoring policy. Other custom charting styles may differ.

## 9. Sustain rules

1. Start sustains at the true attack and end them near the audible release, not the end of the ASCII-tab line.
2. Avoid meaningless micro-sustains. An eighth note is a useful starting *musical* threshold, not a technical cutoff.
3. Short palm-muted and staccato attacks normally have no sustain.
4. Do not extend sustains across silence or into conflicting same-lane attacks.
5. Chord notes can have different audible lengths; disjointed sustains may be appropriate if supported.
6. Extended sustains can represent ringing notes under later attacks, but should not introduce unintended impossible holds.
7. Review sustain behavior in the game because engine version and chart format can affect interpretation.
8. **Before each subsequent note, apply the sustain-tail clearance guidance in §4.3** so the player can hold the entire charted sustain without releasing prematurely.

## 10. Building and validating a tempo map

1. Find the actual first downbeat and account for any intro silence or pickup.
2. Estimate BPM from multiple measures, not one attack.
3. Add time-signature and tempo markers as needed; live performances can drift.
4. Align measure lines to audio transients and phrase boundaries.
5. Use subdivisions that match the music (including triplets, swing, and tuplets).
6. Re-check timing after any BPM edit; moving the tempo map can shift apparent note placement.
7. Listen with a metronome and note-clap playback, then playtest.

## 11. Recommended tab-to-chart conversion algorithm

1. **Input:** recording, tab, tuning, intended guitar part, target game version, and difficulty policy.
2. **Parse tab:** extract string/fret positions, simultaneous note groups, and technique annotations.
3. **Calculate pitches:** use tuned open-string MIDI pitches plus fret numbers.
4. **Align to audio:** obtain note onsets, releases, rests, and tempo map from the recording or a rhythmically reliable source.
5. **Segment phrases:** group repeated riffs, melodic runs, chord progressions, and transitions.
6. **Assign Expert lanes:** optimize each phrase for contour, repeated-note consistency, interval relationships, chord shapes, and hand movement.
7. **Assign note types:** picked/strum, HOPO, tap, open, and sustains; verify engine behavior.
8. **Derive Hard:** retain essential musical attacks; thin punishing streams, taps, and dense chords.
9. **Derive Medium:** reduce to four lanes, simpler rhythms, and at most two-note chords.
10. **Derive Easy:** reduce to three lanes, strong beats, predominantly single notes, and straightforward movement.
11. **Run spacing audit:** calculate both onset-to-onset gaps and **sustain-end-to-next-onset clearance**; flag insufficient release/regrip time and awkward transitions.
12. **Human review:** accept justified exceptions rather than deleting notes mechanically.
13. **Playtest:** all difficulties, full speed, representative difficult sections.
14. **Export and validate:** verify chart loads, sync stays correct, sections exist, and notes behave as intended.

### 11.1 Pseudocode for spacing audit

```text
for difficulty in [Easy, Medium, Hard, Expert]:
    events = group_notes_by_exact_onset(chart[difficulty])
    events = sort_by_time(events, tempo_map)
    for previous, current in adjacent_pairs(events):
        gap_ms = time_ms(current.onset, tempo_map) - time_ms(previous.onset, tempo_map)
        if gap_ms <= 0:
            report_error("non-increasing event times or duplicate event")
        if difficulty != Expert and gap_ms < recommended_review_floor[difficulty]:
            report_warning("dense event spacing; manually review")
        if difficulty == Expert and gap_ms < 60:
            report_info("very dense Expert interval; verify against audio")
        check_transition_difficulty(previous, current)
        if previous.has_sustain:
            tail_gap_ms = time_ms(current.onset, tempo_map) - time_ms(previous.sustain_end, tempo_map)
            if not intentional_supported_extended_sustain(previous, current):
                if tail_gap_ms < recommended_sustain_clearance_ms(difficulty, current, tempo_map):
                    report_warning("sustain tail too close to next note; full hold may be impractical")
        check_sustain_overlap(previous, current)
```

**Important:** The sustain-clearance check is the primary test for whether the **full sustain can be played**; onset-gap thresholds alone cannot establish this. Do not collapse a simultaneous chord into one note when grouping onsets; it remains a single **event containing multiple lanes**. Do not equate the 60 ms Expert informational flag with an engine limitation.

## 12. Worked riff example

```text
 e|-------------------------|
 B|-------------------------|
 G|-------------------------|
 D|-------------------------|
 A|---------3-5-3-----------|
 E|-0-0-0-0-------5-3-0-----|
```

Standard-tuning pitches: `E2 E2 E2 E2 C3 D3 C3 A2 G2 E2`.

A possible Expert lane sequence is `G G G G Y B Y R G G`. The first four repeated low E notes should generally be strummed separately. Whether the ascending and descending notes are HOPOs depends on the actual picking/legato heard in the recording. The tab alone does **not** establish exact timing.

For Hard, retain the recognizable rising and falling line while simplifying very fast repetitions if needed. For Medium, use only G/R/Y/B and thin any run that exceeds its comfortable density. For Easy, preserve the main rhythmic accents and melodic rise/fall on G/R/Y.

## 13. Release checklist

- [ ] Correct tuning and target guitar part identified.
- [ ] Notes aligned to the recording, not ASCII tab spacing.
- [ ] Tempo map and time signatures checked throughout.
- [ ] Repeated pitches and repeated riffs consistently mapped.
- [ ] Chords represent simultaneous audible attacks.
- [ ] No accidental note duplicates or unintended staggered chords.
- [ ] Picked repetitions, HOPOs, and taps behave correctly.
- [ ] Sustain ends and overlaps are intentional.
- [ ] Easy uses G/R/Y and has accessible rhythm.
- [ ] Medium uses G/R/Y/B and avoids excessive chord/strum demands.
- [ ] Hard retains identity without Expert-only gimmicks.
- [ ] Expert includes real attacks without overcharting.
- [ ] Note-gap warnings reviewed in milliseconds at local BPM.
- [ ] Difficult transitions, chord changes, and fast streams playtested.
- [ ] Export tested in intended Clone Hero version.

## 14. Sources and further reading

The references below support terminology, game capabilities, and examples of community difficulty-reduction conventions. **The suggested numerical spacing floors in this rulebook are original editorial recommendations, not numbers attributed to these sources.**

1. [Clone Hero Wiki — Dictionary](https://wiki.clonehero.net/books/general-info/page/dictionary) — terminology for trills, zigs, chimneys, ladders, quads, quints, extended sustains, and more.
2. [Clone Hero — v1.1 announcement](https://clonehero.net/2024/11/10/clonehero-leaderboards.html) — open chords and chart parsing settings, including sustain and chord-snap handling.
3. [Clone Hero community discussion — lower difficulty charting](https://www.reddit.com/r/CloneHero/comments/wr46p5) — community conventions for reducing difficulty; not official standards.
4. [Clone Hero Difficulty Creator](https://github.com/lililwavezlilil/Clone-Hero-Difficulty-Creator) — example of traditional lane restrictions for Medium and Easy.
5. [Clone Hero chart format example](https://github.com/kolt2050/clonehero-generator/blob/main/specs/CHART_FORMAT.md) — illustrative `.chart` representation and note type markers; verify against the target editor/engine.
6. [Guitar difficulty downchart tool](https://cloneherocharts.vercel.app/guitar-difficulties) — example of retaining note times and simplifying shapes when generating lower difficulties.

---

**Final rule:** A chart is successful when it tracks the audible guitar part accurately, expresses its musical shape consistently, and remains intentionally playable at its selected difficulty. There is no single universal time gap that guarantees every possible pattern is playable.
