# Guitar Hero Live (6-Fret) — All Note Combinations and Common Patterns

A practical reference for **Clone Hero 6-fret charting**. This distinguishes *mathematically possible button combinations* from *recommended, playable GHL-style charting*.

## Legend

The GHL controller has three columns, each with a black (upper) and white (lower) button.

| Column | Upper | Lower |
|---|---|---|
| Left | `B1` | `W1` |
| Middle | `B2` | `W2` |
| Right | `B3` | `W3` |

`O` = open strum (no buttons). `+` = simultaneous notes; `→` = successive notes; `×n` = repeat n times. `—` = sustain. `B1+W1` is a **barre** (both buttons in one column), typically shown as a square note.

## 1. Every button combination (64 states)

Six independent buttons produce **2⁶ = 64** possible on/off states: **63 nonempty fretted combinations plus one empty state (`O`)**. The 63 combinations below are an exhaustive mathematical enumeration, **not** a claim that every combination is ergonomic, conventional, or supported as a charted chord in every game/editor.

### Open (1)

- `O`

### Singles (6)

- `B1`, `B2`, `B3`, `W1`, `W2`, `W3`

### Two-button chords (15)

- `B1+B2`, `B1+B3`, `B1+W1`, `B1+W2`, `B1+W3`
- `B2+B3`, `B2+W1`, `B2+W2`, `B2+W3`
- `B3+W1`, `B3+W2`, `B3+W3`
- `W1+W2`, `W1+W3`, `W2+W3`

### Three-button chords (20)

- `B1+B2+B3`
- `B1+B2+W1`, `B1+B2+W2`, `B1+B2+W3`
- `B1+B3+W1`, `B1+B3+W2`, `B1+B3+W3`
- `B1+W1+W2`, `B1+W1+W3`, `B1+W2+W3`
- `B2+B3+W1`, `B2+B3+W2`, `B2+B3+W3`
- `B2+W1+W2`, `B2+W1+W3`, `B2+W2+W3`
- `B3+W1+W2`, `B3+W1+W3`, `B3+W2+W3`
- `W1+W2+W3`

### Four-button chords (15)

- `B1+B2+B3+W1`, `B1+B2+B3+W2`, `B1+B2+B3+W3`
- `B1+B2+W1+W2`, `B1+B2+W1+W3`, `B1+B2+W2+W3`
- `B1+B3+W1+W2`, `B1+B3+W1+W3`, `B1+B3+W2+W3`
- `B1+W1+W2+W3`
- `B2+B3+W1+W2`, `B2+B3+W1+W3`, `B2+B3+W2+W3`
- `B2+W1+W2+W3`, `B3+W1+W2+W3`

### Five-button chords (6)

- `B1+B2+B3+W1+W2`
- `B1+B2+B3+W1+W3`
- `B1+B2+B3+W2+W3`
- `B1+B2+W1+W2+W3`
- `B1+B3+W1+W2+W3`
- `B2+B3+W1+W2+W3`

### Six-button chord (1)

- `B1+B2+B3+W1+W2+W3`

> **Important:** A combination being enumerable does not make it a good GHL chart. Dense 4–6-button chords and awkward overlapping barres are generally unsuitable for normal play. Validate anything unusual in Clone Hero itself.

## 2. Useful chord families

| Family | Examples | Charting guidance |
|---|---|---|
| Same-row neighbors | `B1+B2`, `B2+B3`, `W1+W2`, `W2+W3` | Natural, readable power-chord-like shapes |
| Same-row skips | `B1+B3`, `W1+W3` | Wider spread; use deliberately |
| Cross-row diagonals | `B1+W2`, `B2+W3`, `W1+B2`, `W2+B3` | Common sources of movement and variety |
| Cross-row outer | `B1+W3`, `W1+B3` | Large jump; moderate use |
| Single-column barre | `B1+W1`, `B2+W2`, `B3+W3` | Distinctive GHL square note |
| Three black | `B1+B2+B3` | Dense full-row grip |
| Three white | `W1+W2+W3` | Dense full-row grip |
| Mixed triples | `B1+B2+W3`, `B1+W2+B3`, `W1+B2+W3` | Test ergonomics and musical justification |
| Barre + other fret | `B1+W1+B2`, `B2+W2+W3` | Advanced, often awkward |

## 3. Common single-note patterns

| Pattern | Example | Musical use |
|---|---|---|
| Repeated note / tremolo | `B1 → B1 → B1 → B1` | Repeated picking |
| Black ascending | `B1 → B2 → B3` | Rising melodic line |
| Black descending | `B3 → B2 → B1` | Falling melodic line |
| White ascending | `W1 → W2 → W3` | Rising phrase on lower row |
| White descending | `W3 → W2 → W1` | Falling phrase on lower row |
| Alternating rows | `B1 → W1 → B1 → W1` | Same-column flip |
| Alternating columns | `B1 → B2 → B1 → B2` | Two-fret trill |
| Zigzag | `B1 → W2 → B3 → W2` | Cross-row motion |
| Staircase | `B1 → W1 → B2 → W2 → B3 → W3` | Full-controller run |
| Reverse staircase | `W3 → B3 → W2 → B2 → W1 → B1` | Descending full-controller run |
| Skip run | `B1 → B3 → B2 → B1` | Intervallic melody |
| Pedal tone | `B1 → B2 → B1 → B3 → B1` | Recurring anchor pitch |
| Open alternation | `O → B1 → O → B2 → O → B3` | Open-string riff |
| Gallop | `B1 → B1 → B1` with long-short-short timing | Metal rhythm |
| Reverse gallop | `B1 → B1 → B1` with short-short-long timing | Rhythm variation |
| Triplet run | `B1 → B2 → B3` as three evenly spaced notes | Triplet lead |
| Chromatic-like climb | `B1 → W1 → B2 → W2` | Six-fret-shaped melodic movement; not literal guitar pitch mapping |

## 4. Common chord and mixed patterns

| Pattern | Example |
|---|---|
| Chord repetition | `(B1+B2) → (B1+B2) → (B1+B2)` |
| Chord shift | `(B1+B2) → (B2+B3)` |
| Row shift | `(B1+B2) → (W1+W2)` |
| Chord-to-single | `(B1+B2) → B3 → (B1+B2)` |
| Single-to-chord build | `B1 → B2 → (B1+B2)` |
| Barre repetition | `(B1+W1) → (B1+W1)` |
| Barre migration | `(B1+W1) → (B2+W2) → (B3+W3)` |
| Barre to single | `(B2+W2) → B2 → W2` |
| Cross-row chord alternation | `(B1+W2) → (W1+B2)` |
| Open-chord alternation (not simultaneous) | `O → (B1+B2) → O → (B1+B2)` |
| Chord staircase | `(B1+B2) → (B2+B3) → (W2+W3)` |
| Repeated riff with turnaround | `B1 → B2 → (B1+B2) → O` |

## 5. Note behavior / modifiers

Any *supported* note or chord may appear in different charting contexts. These are **not extra fret combinations**.

- **Strum:** hit with a strum.
- **HOPO (hammer-on/pull-off):** may be hit without strumming when combo conditions are met; otherwise strum to start/recover.
- **Tap:** may be hit without a strum or prior combo.
- **Sustain:** hold the indicated fret combination for the charted duration.
- **Star Power phrase:** special scoring phrase overlay, not a new fret.
- **Open strum:** `O`, strum while holding no frets.
- **Open HOPO / open tap:** special open-note behavior; release frets at the right time. Check the chart's actual rendering and game version.

**Do not confuse** `O → B1` (two successive notes) with `O+B1` (a simultaneous open/fretted chord). The latter is **not a standard GHL six-fret chord**; support and semantics for open chords differ across game versions and engines, so avoid it in a Clone Hero-targeted GHL chart unless independently verified.

## 6. Recommended charting practices

1. Follow the audible rhythm and phrase structure, not arbitrary finger gymnastics.
2. Prefer singles, natural two-note shapes, and occasional barres as the foundation.
3. Use cross-row transitions when they reflect the musical phrase and remain readable.
4. Avoid excessive three-note grips and especially four-to-six-button clusters.
5. Make HOPOs/taps intentional; verify the engine's auto-HOPO behavior.
6. Use open notes for appropriate rhythmic/musical events, not as filler.
7. Sustain only for the actual audible held duration; avoid micro-sustains.
8. Test at full speed with an actual six-fret controller, not only in the editor.
9. Verify all unusual chords and modifiers in the target Clone Hero version.
10. Separate **physically possible** patterns from **fun and fair** patterns.

## 7. `.chart` lane reference

For six-fret `.chart` tracks, the common note IDs are:

| ID | Note |
|---|---|
| `0` | `W1` |
| `1` | `W2` |
| `2` | `W3` |
| `3` | `B1` |
| `4` | `B2` |
| `8` | `B3` |
| `7` | `O` |
| `5` | Strum/HOPO flip modifier |
| `6` | Tap modifier |

Instrument tracks include `ExpertGHLGuitar`, `HardGHLGuitar`, `MediumGHLGuitar`, and `EasyGHLGuitar` (and corresponding GHL bass/rhythm variants, where supported). Consult format documentation before editing raw files.

## Sources and further reading

- [Clone Hero Wiki — How to Play](https://wiki.clonehero.net/books/clone-hero-manual/page/how-to-play)
- [Guitar Game Chart Formats — 6-Fret Guitar](https://solamint.github.io/GuitarGame_ChartFormats/Chart-File-Formats/chart-format/Tracks/6-Fret-Guitar/)
- [Clone Hero 1.1 release notes](https://clonehero.net/2024/11/10/clonehero-leaderboards.html)

*This is an exhaustive list of the six-button subsets and a practical, non-exhaustive catalog of common **sequences**. Arbitrarily long rhythmic sequences have no finite exhaustive list.*
