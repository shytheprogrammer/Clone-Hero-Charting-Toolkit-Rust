# v1.2.1

- Replaced the pitched YARG vocal screen with a Clone Hero LRC manager; removed vocal MIDI export.
- Added timestamp/text editing, waveform playback, seek controls and aligned LRC copies.
- Extended normal LRC phrase ends to one tick before the next phrase start.
- Repaired enhanced backward timestamps by evenly spacing affected words between surrounding valid lyrics, preserving written order.
- Added persistent repair notices, original/repaired times and LYRIC-REVIEW.txt export records.
- Added first-lyric alignment to the playhead, an exact starting-time control and ±100 ms nudges.
- Kept optional Guitar Pro lyric import as editable LRC, without pitched vocal gameplay.

# v1.2.0

- Added a Vocals screen with Guitar Pro melody/verse selection, pitch and lyric preview, shared recording playback, transposition, syllable edits and phrase breaks.
- Exported Rock Band-style solo vocals for YARG in a complete-band notes.mid, preserving the selected instruments and difficulties.
- Imported beat-attached Guitar Pro lyrics, including repeated sections and separate verses, for Clone Hero chart lyric events and synchronized LRC output.
- Kept vocals, lyrics and phrase markers on the shared BPM and bar-shift timeline; audio alignment remains baked into the exported recordings.
- Added validation for polyphonic melodies, missing syllables, unsupported pitches and lyrics without note attacks.
- Included a repeated vocal demo with two verses and a synthesized melody recording.

Windows x64 portable release and GitHub source ZIP. Solo vocals only; phrase boundaries and Star Power require musical review.
