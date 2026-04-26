# VirtualDJ Lyrics AI And Skins Reference

Focused notes for VirtualDJ 2026's AI lyric detection and the parts that are useful from skins, pad pages, and VDJScript.

## Status

- `Official`: current VirtualDJ manual, options list, or VDJScript verbs appendix.
- `Official forum`: VirtualDJ staff, Development Manager, CTO, CEO, moderator, or support staff guidance.
- `Community`: non-staff forum observations worth testing before relying on them.
- `Inference`: repo-level conclusion from the sources above.

## Short Version

VirtualDJ's new lyrics feature is mostly an engine feature, not a skin-rendering API.

What skins can reliably use:

- `has_lyrics`: boolean query for the loaded deck.
- `get_lyrics_language`: text query for the loaded deck's detected lyric language.
- `edit_lyrics`: action that opens the Lyrics Editor for the loaded track.
- `setting 'showLyrics'`: toggle/read the waveform lyric overlay option.
- `setting 'lyricsWaveformSize'`: read/write the waveform lyric overlay size.
- `setting 'getLyrics'`: read/write whether VirtualDJ analyzes new tracks for lyrics.
- `get_time 'to_lyrics'`: official `get_time` target. Treat as useful but verify in the exact skin context because the docs do not expand its semantics.

What skins do not appear to expose directly:

- the current lyric line or word text
- per-word timing, highlight, confidence, or censor state
- whether lyrics came from a cached server result, a fresh AI analysis, or a local edit
- whether a track is currently queued for lyric analysis
- the AI extraction progress, error reason, or server/cache status
- independent font/color styling for the built-in lyric words on waveform/video output

Practical implication: style the presence, absence, language, and controls around lyrics. Do not design as if the skin can restyle the lyric renderer itself.

## How AI Lyric Detection Works

`Official`: The Lyrics Editor edits auto-generated lyrics for any track. It can adjust both timing and text, and its `Re-Analyze` control re-runs extraction. The editor also exposes global automatic censoring; censored words are stored in `lyricsCensoredWords`, and video/audio censoring can replace lyric text on output or mute audio.

Sources:

- [Lyrics Editor manual](https://www.virtualdj.com/manuals/virtualdj/editors/lyricseditor.html)
- [Options list](https://www.virtualdj.com/manuals/virtualdj/appendix/optionslist/)

`Official`: The `getLyrics` option controls whether new tracks are analyzed for lyrics. The current options list notes that setting it to `always` may upload some audio data to VirtualDJ's server for analysis.

`Official forum`: Staff explained that VirtualDJ calculates an audio signature for the song, that this signature requires stems, and that already-analyzed songs can be served from a cached result. If a result is wrong, users can reanalyze or edit locally; user edits are not reuploaded as the global AI result.

Source:

- [How Lyrics are analyzed???](https://virtualdj.com/forums/267223/VirtualDJ_Technical_Support/How_Lyrics_are_analyzed%3F%3F%3F.html)

`Official forum`: Early VirtualDJ 2026 forum guidance says lyric extraction requires a PRO user context and stems enabled. Some builds show the user-facing message that lyrics extraction requires Stems 2.0 if the machine is set to only use prepared stems and none exist for that song. Staff also clarified in the VirtualDJ 2026 release thread that very long tracks may not be attempted by the lyric engine.

Sources:

- [Virtual DJ 2026](https://www.virtualdj.com/forums/266311/VirtualDJ_Technical_Support/Virtual_DJ_2026.html)
- [Stems 2.0](https://virtualdj.com/forums/266488/VirtualDJ_Technical_Support/Stems_2.0.html)

`Community`: Several early reports describe lyric analysis as serialized and occasionally needing manual re-analysis after a stalled or interrupted extraction. Use this as troubleshooting context, not guaranteed engine behavior.

Source:

- [Lyrics issues with stems VDJ 2026](https://virtualdj.com/forums/266347/VirtualDJ_Technical_Support/Lyrics_issues_with_stems_VDJ_2026.html)

## Skin Styling Surface

### Availability Badge

Use `has_lyrics` for a binary state. It is the cleanest styling signal currently exposed to skins.

```xml
<panel visibility="has_lyrics">
  <text text="LYRICS" color="#f4f4f4" fontsize="11" align="center"/>
</panel>

<panel visibility="not has_lyrics">
  <text text="NO LYRICS" color="#777777" fontsize="11" align="center"/>
</panel>
```

### Language Chip

Use `get_lyrics_language` for text, or compare it with `param_equal` if you want language-specific colors or icons.

```xml
<text action="has_lyrics ? get_lyrics_language : get_text ''"
      color="#d8d8d8"
      fontsize="11"
      align="center"/>
```

Example conditional color:

```xml
<text action="get_lyrics_language"
      visibility="param_equal `get_lyrics_language` 'en'"
      color="#9bd67d"
      fontsize="11"
      align="center"/>

<text action="get_lyrics_language"
      visibility="not param_equal `get_lyrics_language` 'en'"
      color="#d8d8d8"
      fontsize="11"
      align="center"/>
```

### Open Editor

Use `edit_lyrics` for a skin button that opens the Lyrics Editor for the loaded deck.

```xml
<button action="edit_lyrics" query="has_lyrics">
  <size width="64" height="24"/>
  <off color="#222222" border="#444444" radius="4"/>
  <on color="#2f3a26" border="#9bd67d" radius="4"/>
  <text text="EDIT" color="#eeeeee" fontsize="11" align="center"/>
</button>
```

`Inference`: The editor can still be useful when lyrics are missing or wrong, so you may prefer not to hide the button entirely when `has_lyrics` is false.

### Waveform Overlay Toggle

The built-in waveform lyric overlay is controlled by options, not by a skin-owned lyric text element.

```xml
<button action="setting 'showLyrics'" query="setting 'showLyrics'">
  <size width="70" height="24"/>
  <off color="#181818" border="#333333" radius="4"/>
  <on color="#243246" border="#78a8ff" radius="4"/>
  <text text="LYRICS" color="#eeeeee" fontsize="11" align="center"/>
</button>
```

`Official`: Relevant skin-facing settings from the options list:

- `showLyrics`: show lyrics on the waveform when available.
- `lyricsWaveformSize`: size multiplier for lyrics on the waveform.
- `getLyrics`: whether VirtualDJ analyzes new tracks for lyrics.

## Browser And Filter Behavior

`Official`: The VDJScript verbs appendix exposes `has_lyrics` for the loaded deck, not the browsed track. For browsed-track filtering, use the browser/filter field rather than assuming `has_lyrics browsed` exists.

`Official forum`: A February 2026 thread reported that the instant filter `Has Lyrics > list has Lyrics` returned incomplete states. A moderator suggested using an "is not blank" condition, which returned both `yes` and `edited` in their test.

Source:

- [Bug in Instant Filters > Has Lyrics](https://virtualdj.com/forums/267592/VirtualDJ_Technical_Support/Bug_in_Instant_Filters_%3E_Has_Lyrics.html)

Practical filter notes:

- Prefer `Has Lyrics is not ""` or the equivalent UI "is not blank" condition when trying to catch both AI-detected and edited lyric states.
- Treat `Has Lyrics = yes` as too narrow until verified on the target build.
- If you need a skin button for browser filtering, route through `quick_filter` with the filter expression you have verified in VirtualDJ.

## Known Quirks To Design Around

- Re-analysis can improve results, but it is not guaranteed to fix wrong or foreign-language output.
- A wrong/blank result may still leave the database in a state where the browser's "has lyrics" field is not simply `no`.
- Deleting lyrics in the editor has been reported to mark the field as edited rather than clearing the state; verify current build behavior before depending on `no`.
- Long tracks may not be sent through lyric extraction.
- Stems settings matter. For lyric-related controls, it is useful to provide a clear "open editor" or "prepare stems" path instead of only a passive status light.
- There is no documented batch lyric-analysis verb as of the checked sources. `reanalyze` and `browsed_file_analyze` are general analysis verbs; do not assume they perform lyrics extraction.

## Censoring Notes

`Official`: Relevant options:

- `lyricsCensoredWords`: words to censor.
- `lyricsCensorMatching`: `Exact`, `Contains`, or `Wildcard`.
- `lyricsCensorVideo`: replace detected censored lyric words on lyrics/karaoke video output.
- `lyricsCensorAudio`: mute audio for detected censored lyric words.

`Official`: The options list documents wildcard examples such as matching a base word exactly unless a trailing wildcard is used for word starts.

`Official forum`: Early VirtualDJ 2026 censor matching had punctuation edge cases around quote marks. Staff said the matching was enhanced for a following build. Keep this in mind if a field report involves older builds.

Source:

- [2026 - Lyric censoring can fail if there's speechmarks](https://www.virtualdj.com/forums/266499/VirtualDJ_Technical_Support/2026_-_Lyric_censoring_can_fail_if_there%27s_speechmarks.html)

## Esoteric Script Behavior Worth Remembering

This section is for low-documentation or easy-to-miss behavior that may help skin work. Treat each item as a candidate to test in the target build.

| Verb or behavior | Why it matters |
| ---------------- | -------------- |
| `get_time 'to_lyrics'` | Official `get_time` target. Potentially useful for a "next lyric soon" indicator, but not expanded in the docs. |
| `edit_lyrics` | Opens the Lyrics Editor from a skin/control. Better than sending users through cover-art or browser context menus. |
| `get_status` | Returns background task information. Probe whether the current build exposes stems/lyrics extraction text usefully before displaying it. |
| `var_list` | Debugging aid. Forum advice suggests scattering `set` calls and inspecting `var_list` when a script path is complex. |
| `skin_panelgroup_available` | Lets a skin remove panels from group cycling without rebuilding the group. Useful for layouts with optional lyric/censor panels. |
| `skin_empty_buttons` | Used by the SDK custom-browser example to reveal empty custom-button areas. Useful when preserving user-programmable space. |
| `load_pulse` / `load_pulse_active` | Trigger brief UI changes after a new track is loaded or becomes audible. Useful for refreshing transient badges without timers. |
| `browsed_file_prepare_stems` | Browser action to prepare stems for selected files. Useful companion action because lyric extraction depends on stems. |
| `browsed_file_reveal` / `browsed_file_reload_tag` | Useful for prep/debug views: jump to the file in the OS, or force VirtualDJ to reload tags from the source file. |
| `browser_padding` / `font_size` | Browser density controls that are easy to miss when making lyric-heavy prep views. |
| `setting_setsession` / `setting_setsession_deck` | Temporarily override settings for the current session. Potentially useful for skin modes that should not permanently change user preferences. |
| `setting_ismodified` | Detect whether a setting differs from default. Useful for "skin setup" diagnostics. |
| `has_linked_tracks` / `mark_linked_tracks` | Build UI around VirtualDJ's linked/remix track relation instead of maintaining a parallel tag convention. |
| `get_spectrum_band 1 32 vocals` | The official example shows a stem-aware spectrum query. Interesting for vocal-reactive indicators, though unrelated to lyric text itself. |
| `bpm_stabilizer` | Fluid-grid helper that can lock a fluid track to the current BPM so it can stay synchronized. Useful around karaoke/lyric tracks with drifting tempo. |
| `get_vdj_folder` | Returns the VirtualDJ home folder. Useful for diagnostics and skin setup helpers. |
| `show_text` | Controller display helper; useful for hardware feedback, not a normal skin text element. |
| `effect X active` | Old v7-compatible syntax. Staff says prefer `effect_active X` in v8+ code. |
| `var 'name' X ?` | Staff says this is equivalent to `var_equal 'name' X ?`. Prefer `var_equal` for clarity. |

Sources:

- [VDJScript verbs appendix](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html)
- [Undocumented scripts](https://virtualdj.com/forums/213099/VirtualDJ_Technical_Support/Undocumented_scripts.html)
- [Where can I find a VDJ script reference?](https://virtualdj.com/forums/264082/VirtualDJ_Technical_Support/Where_can_I_find_a_VDJ_script_reference%253F.html)

## Source-Hunting Notes

`Official forum`: Staff has said VDJScript verbs are shared by skins and controller mappings, and that the in-app controller/custom-button editors are the most complete source when the public web page lags. Some special verbs without descriptions are intended for narrow hardware/controller cases and should not be copied casually into skins.

Source:

- [VDJ Script Verbs update](https://www.virtualdj.com/forums/205590/VirtualDJ_Skins/VDJ_Script_Verbs_update.html)

`Official forum`: A support reply from the VirtualDJ 7 era also notes that action definitions can be inspected in `Documents/VirtualDJ/Languages/English.xml`. Treat that as a discovery aid, then verify behavior in the current in-app editor and current manual.

Source:

- [VDJ Script](https://www.virtualdj.com/forums/122894/General_Discussion/VDJ_Script.html)
