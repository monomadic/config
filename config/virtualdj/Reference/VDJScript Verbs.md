# VirtualDJ VDJScript API Reference

Curated API-oriented reference for VDJScript used in skins, pad pages, custom buttons, and controller mappings.

This file now has two layers:

- The curated API layer at the top: canonical names, aliases, scripting surfaces, and reliability notes.
- The broad catalog below: wide coverage tables that are still being normalized.

## Status

This is the first API pass, not the final exhaustive pass.

The highest-confidence material in this file is the curated layer at the top. It is intentionally biased toward the verbs and patterns that matter most for:

- skin XML
- pad page XML
- custom button actions
- controller mappings
- small inline query and text scripts

Current curated coverage includes:

- flow and timing verbs
- variables and deck targeting
- skin panels and settings
- browser, search, and sideview actions
- transport, cue, loop, and sync controls
- filter and FX selection
- sampler page-aware helpers
- deck assignment and crossfader routing

## Reliability Labels

- `Official`: current VirtualDJ manual or VDJPedia
- `Official forum`: VirtualDJ staff, Development Manager, or CTO forum guidance
- `Community`: forum guidance from non-staff users
- `Inference`: a conclusion drawn from official behavior plus repo usage

## Surface Legend

- `Map`: controller mapping
- `Button`: custom button action
- `Pad`: pad page action or query
- `SkinAction`: skin `action=""`, inline button body, or interactive element action
- `SkinQuery`: skin `query=""`, `visibility=""`, `condition=""`, or equivalent boolean/value query slot
- `Text`: skin `text`, `format`, color backticks, or other string/value-returning query usage

These are practical surfaces, not hard type-check guarantees. When a surface is listed, read it as "commonly useful and normally safe there", not "the only place the verb can run".

## Alias Policy

- Use the primary name shown in the current official verbs page as the canonical heading.
- List official aliases explicitly.
- Prefer canonical names in examples unless the alias is the name people are most likely to search for.
- If a synonym is only found in community posts and not in the official manual, label it as community-only rather than treating it as an official alias.

## High-Frequency Alias Index

| Canonical | Official aliases | Notes |
| --------- | ---------------- | ----- |
| `param_bigger` | `param_greater` | Same official entry |
| `skin_panel` | `skin_pannel` | Keep the official spelling as canonical |
| `skin_panelgroup` | `skin_pannelgroup` | Same note as above |
| `lock_panel` | `lock_pannel` | Acts on `<split>`, not `<panel>` |
| `settings` | `config` | Opens the configuration window |
| `filter` | `filter_slider` | Main deck filter / ColorFX amount control |
| `pad_page` | `pad_pages` | Page activation / page menu |
| `pad_page_select` | `pad_page_favorite_select` | Favorite pad-page slot selection |
| `effect_active` | `effect_activate` | Slot activation |
| `effect_slider` | `effect_slider_slider` | Slot slider control |
| `play_button` | `play_3button` | Behavior depends on `playMode` |
| `stop_button` | `stop_3button` | Behavior depends on `playMode` |
| `cue_button` | `cue_3button` | Behavior depends on `cueMode` |
| `smart_play` | `auto_sync` | Startup auto-sync behavior, not the same as `play_sync` |
| `play_sync_onbeat` | `sync_nocbg` | Local-beat sync variant |
| `is_fluid` | `has_variable_bpm` | Fluid-grid query |
| `set_fluid` | `set_variable_bpm` | Fluid-grid toggle |
| `get_sample_name` | `get_sample_slot_name` | Absolute sample-slot label |
| `add_list` | `add_virtualfolder` | Virtual folder creation |
| `info_options` | `infos_options` | Browser info-panel context menu |
| `browser_zoom` | `browser` | Browser zoom control |

## Core Execution Model

### Action vs Query vs Dual Verbs

- `Action` verbs primarily do something: `play_pause`, `load`, `skin_panel`
- `Query` verbs primarily return information: `get_browsed_song`, `get_time`, `sampler_loaded`
- `Dual` verbs are often used both ways: `filter`, `setting`, `var`

When a verb is documented here as `Dual`, that means it is commonly used in both action chains and value/query contexts.

### Deck Scoping

VDJScript actions can be prefixed with deck context:

```vdjscript
deck 1 play
deck 2 volume 50%
deck master get_level
```

Use deck scoping whenever the result should be explicit instead of depending on the current focused deck.

Notes:

- `deck master` means "run this in the current master deck context".
- In sampler title and text paths, explicit `deck 1 ... : deck 2 ...` resolution can be more reliable than raw `deck master ...`.

Sources:

- `Official`: current VDJScript verbs appendix, deck examples
- `Official forum`: sampler sync/build-specific discussion around master-deck sampler routing

## Curated High-Frequency Entries

### `up`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Branch on press versus release

Typical forms:

```vdjscript
up ? action_on_press : action_on_release
```

Use when:

- you need separate press/release logic in controller mappings
- you want momentary behavior without relying on vars

Sources:

- `Official`: VDJScript verbs appendix

### `down`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Branch on press versus release

Typical forms:

```vdjscript
down ? action_on_press : action_on_release
```

Preferred usage:

- use for hold-style momentary effects and pad behaviors
- pair with `filter 50%` or another neutral reset on the release side when the control is momentary

Sources:

- `Official`: VDJScript verbs appendix

### `holding`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Branch based on whether the control was held long enough

Typical forms:

```vdjscript
holding ? long_press_action : short_press_action
holding 1000ms ? long_press_action : short_press_action
```

Notes:

- The default threshold is documented as `500ms`.
- Prefer this over hand-rolled timer vars when you only need short-press versus long-press behavior.

Sources:

- `Official`: VDJScript verbs appendix

### `doubleclick`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Branch based on whether the control was pressed twice within the allowed window

Typical forms:

```vdjscript
doubleclick ? double_action : single_action
doubleclick 1000ms ? double_action : single_action
```

Notes:

- The default interval is documented as `300ms`.

Sources:

- `Official`: VDJScript verbs appendix

### `repeat_start`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Start a named repeating task after the first interval passes

Typical forms:

```vdjscript
repeat_start 'scroll' 1000ms & browser_scroll +1
repeat_start 'pulse' 1bt
repeat_start 'name' `get_var interval_ms`
```

Preferred usage:

- use named repeats for background animation, repeated browser movement, and timed FX patterns
- always pair long-lived repeats with a clear `repeat_stop`

Sources:

- `Official`: VDJScript verbs appendix

### `repeat_start_instant`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Start a named repeating task immediately, then keep repeating on the chosen interval

Typical forms:

```vdjscript
repeat_start_instant 'scroll' 250ms & browser_scroll +1
```

Preferred usage:

- use when the first action should happen right away rather than after the first delay

Sources:

- `Official`: VDJScript verbs appendix

### `repeat_stop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Stop a repeat created with `repeat_start` or `repeat_start_instant`

Typical forms:

```vdjscript
repeat_stop 'scroll'
```

Sources:

- `Official`: VDJScript verbs appendix

### `wait`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Delay the next action in a chain

Typical forms:

```vdjscript
wait 1bt & pause
wait 500ms & play
```

Notes:

- Use it for simple timed chains.
- If you need a persistent process, prefer named repeats instead of stacking many waits.

Sources:

- `Official`: VDJScript verbs appendix

### `blink`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Pad`, `SkinQuery`

Official summary:

- Toggle LED or visual state on and off at a configurable rate

Typical forms:

```vdjscript
blink
blink 1000ms
blink 1bt
blink 1bt 25%
```

Preferred usage:

- use as the true branch in queries: `effect_active 1 ? blink 1bt : off`

Sources:

- `Official`: VDJScript verbs appendix

### `param_bigger`

Aliases: `param_greater`

Kind: `Dual`

Typical surfaces: `Map`, `Pad`, `SkinQuery`, `Text`

Official summary:

- Compare the caller value against a value or another action result

Typical forms:

```vdjscript
param_bigger 0 ? action1 : action2
param_bigger pitch pitch_slider
```

Preferred usage:

- use the canonical manual name `param_bigger` in docs
- mention `param_greater` when helping users search or migrate older scripts

Sources:

- `Official`: VDJScript verbs appendix

### `param_equal`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Pad`, `SkinQuery`, `Text`

Official summary:

- Compare the caller value or the first parameter value against another value

Typical forms:

```vdjscript
param_equal `get_browsed_song 'type'` "audio"
param_equal 0.5 filter ? on : off
```

Preferred usage:

- use backticks when the compared value comes from another action
- prefer this over brittle string slicing or var mirrors when the source action already returns the value you need

Sources:

- `Official`: VDJScript verbs appendix

### `var`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinQuery`, `Text`

Official summary:

- Test whether a variable is true

Typical forms:

```vdjscript
var "my_var" ? action1 : action2
```

Notes:

- Use vars when you truly need stored state across events.
- Do not reach for vars when a built-in query verb already answers the question directly.

Sources:

- `Official`: VDJScript verbs appendix
- `Community`: experienced skin scripters repeatedly caution against overusing vars when a built-in UI/query path already exists

### `set`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Set a variable to a value

Typical forms:

```vdjscript
set 'varname' 5
set 'page_fx_active' 1
set 'remembered_filter' `filter`
```

Notes:

- Use for stored state, not as a substitute for native UI or deck state queries.

Sources:

- `Official`: VDJScript verbs appendix, variable section

### `toggle`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Toggle a variable between true and false

Typical forms:

```vdjscript
toggle "my_var"
```

Preferred usage:

- good for explicit user modes
- avoid using it when the target should mirror a built-in state such as `play`, `loop`, or `masterdeck`

Sources:

- `Official`: VDJScript verbs appendix

### `get_var`

Aliases: none

Kind: `Query`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinQuery`, `Text`

Official summary:

- Return the value of a named variable

Typical forms:

```vdjscript
get_var 'varname'
```

Sources:

- `Official`: VDJScript verbs appendix

### `set_deck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Use a script result or implicit value to decide which deck the following action applies to

Typical forms:

```vdjscript
set_deck `get_var target_deck` & play
```

Preferred usage:

- use in mappings when the target deck is computed at runtime
- prefer explicit `deck 1 ...`, `deck 2 ...`, or `deck master ...` when the target is already known

Sources:

- `Official`: VDJScript verbs appendix

### `skin_panel`

Aliases: `skin_pannel`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Show or hide a named panel

Typical forms:

```vdjscript
skin_panel 'my_panel' on
skin_panel 'my_panel' off
```

Preferred usage:

- use for explicit panel toggles
- if panel visibility should simply follow a live condition, prefer panel `visibility=""` or other query-driven skin logic instead of setting extra vars only to call `skin_panel`

Sources:

- `Official`: VDJScript verbs appendix
- `Official`: Skin SDK panel documentation

### `skin_panelgroup`

Aliases: `skin_pannelgroup`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Switch which panel in a named group is shown

Typical forms:

```vdjscript
skin_panelgroup 'rack' 'fx'
skin_panelgroup 'rack' +1
skin_panelgroup 'rack' 0.75
```

Preferred usage:

- use when the user is deliberately switching between remembered panel modes
- use `skin_panelgroup_available` to keep unavailable panels out of cycles

Sources:

- `Official`: VDJScript verbs appendix
- `Official`: Skin SDK panel documentation

### `lock_panel`

Aliases: `lock_pannel`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Despite the name, this acts on `<split>` elements rather than `<panel>`

Notes:

- Document this quirk explicitly wherever the verb is mentioned.

Sources:

- `Official`: VDJScript verbs appendix

### `setting`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`, `SkinQuery`, `Text`

Official summary:

- Read or write a specific VirtualDJ setting

Typical forms:

```vdjscript
setting "jogSensitivityScratch" 80%
setting "videoRandomTransition" on
setting "filterDefaultResonance"
```

Preferred usage:

- prefer it for actual program settings such as `filterDefaultResonance`
- do not use settings as a substitute for temporary UI state that belongs in a script variable or a panel selection

Sources:

- `Official`: VDJScript verbs appendix
- `Official forum`: `setting filterDefaultResonance` recommended by CTO Adion for filter resonance scripting

### `settings`

Aliases: `config`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Open the configuration window

Sources:

- `Official`: VDJScript verbs appendix

### `display_time`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Set the displayed time mode to `total`, `remain`, `elapsed`, `+1`, or `-1`

Typical forms:

```vdjscript
display_time 'remain'
display_time 'remain,elapsed'
display_time +1
```

Preferred usage:

- prefer this over custom elapsed/remain vars when the goal is simply to switch the time-display mode

Sources:

- `Official`: VDJScript verbs appendix
- `Inference`: prefer the built-in time-display mode before inventing a parallel time-mode variable

### `get_time`

Aliases: none

Kind: `Query`

Typical surfaces: `SkinQuery`, `Text`, `Map`, `Button`

Official summary:

- Return elapsed, remaining, or total time, with optional unit and target-point arguments

Typical forms:

```vdjscript
get_time
get_time 'remain'
get_time 'remain' 'short'
get_time 1000
get_time 'absolute'
```

Notes:

- `get_time` follows the current `display_time` mode unless you explicitly pass `elapsed`, `remain`, or `total`.

Sources:

- `Official`: VDJScript verbs appendix

### `get_browsed_song`

Aliases: none

Kind: `Query`

Typical surfaces: `Button`, `Pad`, `SkinQuery`, `Text`

Official summary:

- Return a property from the currently browsed file

Typical forms:

```vdjscript
get_browsed_song 'title'
get_browsed_song 'type'
```

Preferred usage:

- pair with `param_equal` when branching on browsed-file metadata

Sources:

- `Official`: VDJScript verbs appendix

### `load`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Load the selected song, or a specified full path, onto the deck

Typical forms:

```vdjscript
load
load "/path/to/file.mp3"
```

Sources:

- `Official`: VDJScript verbs appendix

### `play`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Start the deck

Typical forms:

```vdjscript
play
```

Sources:

- `Official`: VDJScript verbs appendix

### `play_stutter`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- If paused, start the deck. If playing, restart from the last stutter point.

Typical forms:

```vdjscript
play_stutter
```

Preferred usage:

- use when the control should deliberately retrigger from the stutter point
- for ordinary transport start behavior, prefer plain `play` or `play_pause`

Sources:

- `Official`: VDJScript verbs appendix

### `play_pause`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Start the deck if paused; pause it if playing

Typical forms:

```vdjscript
play_pause
```

Sources:

- `Official`: VDJScript verbs appendix

### `pause`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Pause the deck

Typical forms:

```vdjscript
pause
```

Sources:

- `Official`: VDJScript verbs appendix

### `stop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Stop to the last cue point, then on second press to the beginning of the song, then cycle through the cue points

Typical forms:

```vdjscript
stop
```

Important note:

- This is not a simple synonym for `pause`.
- For deterministic documentation, spell out `stop`, `pause`, or `pause_stop` based on the behavior you actually want.

Sources:

- `Official`: VDJScript verbs appendix

### `cue_stop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- If playing, pause and go to the last cue point. If paused, set the current position as cue and preview while pressed.

Typical forms:

```vdjscript
cue_stop
cue_stop 1
cue_stop 57
```

Sources:

- `Official`: VDJScript verbs appendix

### `pad_page`

Aliases: `pad_pages`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Activate a pad page, override a page slot, or show the pad-page selector menu

Typical forms:

```vdjscript
pad_page 1
pad_page 1 hotcues
pad_page btn1
pad_page
```

Preferred usage:

- use the canonical singular name `pad_page` in docs
- call out `pad_pages` as the official alias for searchability

Sources:

- `Official`: VDJScript verbs appendix

### `filter`

Aliases: `filter_slider`

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`, `SkinQuery`, `Text`

Official summary:

- Apply the selected ColorFX to the sound; nothing is applied at `50%`, and more is applied the farther from center

Typical forms:

```vdjscript
filter
filter 50%
filter 75%
down ? filter 75% : filter 50%
```

Preferred usage:

- for the main deck filter path, pair it with `filter_selectcolorfx`
- document `50%` as neutral; do not normalize docs around `0%` as if it were the center-off value

Sources:

- `Official`: VDJScript verbs appendix

### `filter_selectcolorfx`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Select the ColorFX controlled by the filter knob

Typical forms:

```vdjscript
filter_selectcolorfx 'Echo'
filter_selectcolorfx +1
filter_selectcolorfx -1
```

Preferred usage:

- use this as the main documented way to choose the deck's ColorFX
- prefer it over ad-hoc `effect_show_gui 'colorfx'` workflows when the goal is simply to set the selected ColorFX

Quirk:

- CTO Adion explicitly recommends `filter_selectcolorfx` to avoid duplicated filter states when switching ColorFX/filter behaviors.

Sources:

- `Official`: VDJScript verbs appendix
- `Official forum`: "Default filter and color fx filter", Adion, 2023-05-19

### `filter_label`

Aliases: none

Kind: `Query`

Typical surfaces: `SkinQuery`, `Text`, `Button`, `Pad`

Official summary:

- Return the label shown under the filter knob

Typical forms:

```vdjscript
filter_label
filter_label 'clean'
filter_label 'name'
```

Preferred usage:

- use `filter_label 'name'` when you specifically want the ColorFX name rather than the value-style label

Sources:

- `Official`: VDJScript verbs appendix

### `filter_resonance`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`, `SkinQuery`

Official summary:

- Change the filter resonance

Typical forms:

```vdjscript
filter_resonance 50%
filter_resonance +5%
filter_resonance -5%
```

Preferred usage:

- use for live control of resonance
- use `setting 'filterDefaultResonance' ...` when what you really want is the program setting, not a one-off movement

Sources:

- `Official`: VDJScript verbs appendix
- `Official`: options list
- `Official forum`: filter resonance discussion in the ColorFX/filter thread

### `effect_select`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Select an effect in a slot and deactivate the previous effect in that slot

Typical forms:

```vdjscript
effect_select 1 "echo"
effect_select 1 -1
effect_select +1
effect_select 1 0.2
```

Preferred usage:

- use slot-based effect selection for deterministic pad pages and skins
- prefer this over name-only global assumptions when you care which slot owns the effect

Sources:

- `Official`: VDJScript verbs appendix

### `effect_active`

Aliases: `effect_activate`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`, `SkinQuery`

Official summary:

- Activate or deactivate the effect on a specific slot

Typical forms:

```vdjscript
effect_active 1
effect_active 1 on
effect_active 1 off
effect_active 1 'flanger' on
```

Preferred usage:

- keep docs slot-centric when describing normal deck FX behavior
- mention the alias, but keep `effect_active` as canonical

Sources:

- `Official`: VDJScript verbs appendix

### `effect_slider`

Aliases: `effect_slider_slider`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Move a specific slider on the effect loaded in a given slot

Typical forms:

```vdjscript
effect_slider 1 2 50%
effect_slider 1 0%
```

Preferred usage:

- use explicit slot and slider numbers in docs and examples
- pair with `effect_select` and `effect_active` for deterministic FX presets

Sources:

- `Official`: VDJScript verbs appendix

### `effect_colorfx`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Select an effect for one of up to four custom ColorFX slots

Typical forms:

```vdjscript
effect_colorfx 1 "echo"
```

Preferred usage:

- use when building extra ColorFX-like controls that should not hijack the main deck filter path

Quirk:

- This is not the same thing as selecting the standard deck filter ColorFX. For that, prefer `filter_selectcolorfx`.

Sources:

- `Official`: VDJScript verbs appendix
- `Official forum`: ColorFX/filter guidance from staff and CTO posts

### `effect_colorslider`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Control an effect from a center-off position to full on left or right

Typical forms:

```vdjscript
effect_colorslider 1
```

Preferred usage:

- pair it with `effect_colorfx` for custom ColorFX-style controls

Sources:

- `Official`: VDJScript verbs appendix

### `effect_show_gui`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Show the control window for an effect

Typical forms:

```vdjscript
effect_show_gui 1
effect_show_gui 'colorfx'
```

Notes:

- Treat GUI access as separate from canonical selection logic. Opening a GUI does not make it the preferred API path for selection or activation.

Sources:

- `Official`: VDJScript verbs appendix

### `sampler_play`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Play the default sample or a specified absolute sample slot

Typical forms:

```vdjscript
sampler_play
sampler_play 4
```

Preferred usage:

- use for absolute-slot sampler control

Sources:

- `Official`: VDJScript verbs appendix

### `sampler_stop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Stop one sample or all playing samples

Typical forms:

```vdjscript
sampler_stop 4
sampler_stop all
```

Preferred usage:

- use for absolute-slot stop logic or global cleanup actions

Sources:

- `Official`: VDJScript verbs appendix

### `sampler_pad`

Aliases: none

Kind: `Dual`

Typical surfaces: `Pad`, `Button`, `SkinAction`, `SkinQuery`, `Text`

Official summary:

- Trigger the sample in the visible sampler pad position; in display contexts it can also return the visible pad label

Typical forms:

```vdjscript
sampler_pad 1
sampler_pad 1 "auto"
`sampler_pad 1`
```

Preferred usage:

- use this for page-aware sampler UI, not fixed absolute slot control

Quirk:

- In display contexts, `sampler_pad <n>` is often the safest way to show the visible sample label on the current page.

Sources:

- `Official`: VDJScript verbs appendix
- `Official`: pads manual
- `Inference`: current repo examples and current page-aware sampler usage patterns

### `sampler_pad_page`

Aliases: none

Kind: `Dual`

Typical surfaces: `Pad`, `Button`, `SkinAction`, `SkinQuery`, `Text`

Official summary:

- Change or query the current 8-pad sampler window

Typical forms:

```vdjscript
sampler_pad_page +1
sampler_pad_page -1
sampler_pad_page
```

Preferred usage:

- treat this as the official pager behind sampler `1-8`, `9-16`, `17-24`, and later windows

Sources:

- `Official`: VDJScript verbs appendix
- `Official`: pads manual

### `sampler_loaded`

Aliases: none

Kind: `Query`

Typical surfaces: `Pad`, `SkinQuery`, `Text`

Official summary:

- Return true when a sample is loaded in the target slot

Typical forms:

```vdjscript
sampler_loaded 1
sampler_loaded 1 "auto"
```

Quirk:

- The manual explicitly documents fixed-slot behavior.
- The page-aware `sampler_loaded <n> "auto"` pattern is widely used in practice and works with current custom sampler pad patterns, but it should still be labeled as build-sensitive rather than silently promoted to timeless official behavior.

Sources:

- `Official`: VDJScript verbs appendix
- `Inference`: repo usage plus current custom-page practice

### `sampler_color`

Aliases: none

Kind: `Query`

Typical surfaces: `Pad`, `SkinQuery`, `Text`

Official summary:

- Return the sample color for the visible sampler pad slot

Typical forms:

```vdjscript
sampler_color 1
sampler_color 1 "auto"
```

Important note:

- The official manual explicitly says the sample number takes `sampler_pad_page` into account, which makes `sampler_color` one of the safest documented page-aware helpers.

Sources:

- `Official`: VDJScript verbs appendix

### `get_sample_name`

Aliases: `get_sample_slot_name`

Kind: `Query`

Typical surfaces: `Text`, `Pad`, `SkinQuery`

Official summary:

- Return the name of a specified absolute sample slot

Typical forms:

```vdjscript
get_sample_name 9
```

Preferred usage:

- use this for absolute-slot sampler UI
- do not substitute it for `sampler_pad <n>` when the label should follow the currently visible page

Sources:

- `Official`: VDJScript verbs appendix

### `swap_decks`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Swap deck 1 and deck 2

Typical forms:

```vdjscript
swap_decks
```

Preferred usage:

- use as an explicit global deck-management command, not as a substitute for `leftdeck` or `rightdeck`

Sources:

- `Official`: VDJScript verbs appendix

### `clone_deck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Clone the current deck to the other deck, keeping song and position aligned

Typical forms:

```vdjscript
clone_deck
```

Preferred usage:

- useful for beat-juggling or quick A/B duplication

Sources:

- `Official`: VDJScript verbs appendix

### `clone_from_deck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Clone from the other deck into the current deck

Typical forms:

```vdjscript
clone_from_deck
```

Sources:

- `Official`: VDJScript verbs appendix

### `move_deck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Move the song from the called deck into the target deck and unload it from the caller

Typical forms:

```vdjscript
move_deck 2
```

Notes:

- treat this as a content-transfer action, not a cosmetic deck-side switch

Sources:

- `Official`: VDJScript verbs appendix

### `get_deck`

Aliases: none

Kind: `Query`

Typical surfaces: `Map`, `Button`, `SkinQuery`, `Text`

Official summary:

- Get the number of the deck

Typical forms:

```vdjscript
get_deck
```

Preferred usage:

- use for deck-aware text and conditions when the action should follow the current deck context instead of a hard-coded deck number

Sources:

- `Official`: VDJScript verbs appendix

### `masterdeck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Select or unselect this deck as the master deck

Typical forms:

```vdjscript
masterdeck
deck 3 masterdeck
```

Important note:

- When a master deck is set, synchronization operations use it as the reference deck.

Sources:

- `Official`: VDJScript verbs appendix

### `leftdeck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Select this deck to be the left deck

Typical forms:

```vdjscript
deck 3 leftdeck
leftdeck +1
```

Preferred usage:

- most useful in skins and mappings that expose more than two decks at once

Sources:

- `Official`: VDJScript verbs appendix

### `rightdeck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Select this deck to be the right deck

Typical forms:

```vdjscript
deck 3 rightdeck
rightdeck +1
```

Preferred usage:

- most useful in skins and mappings that expose more than two decks at once

Sources:

- `Official`: VDJScript verbs appendix

### `invert_deck`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Switch the left or right deck assignment

Typical forms:

```vdjscript
invert_deck
invert_deck 'left'
invert_deck 'right'
```

Sources:

- `Official`: VDJScript verbs appendix

### `leftcross`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Assign this deck to the left of the crossfader

Typical forms:

```vdjscript
deck 3 leftcross
deck 3 leftcross 'only'
leftcross 'none'
```

Sources:

- `Official`: VDJScript verbs appendix

### `rightcross`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Assign this deck to the right of the crossfader

Typical forms:

```vdjscript
deck 3 rightcross
```

Sources:

- `Official`: VDJScript verbs appendix

### `pfl`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `SkinAction`, `SkinQuery`

Official summary:

- Select whether this deck is sent to the headphones

Typical forms:

```vdjscript
pfl
pfl 75%
```

Important note:

- The official manual also documents slider or percent use for headphone level control.

Sources:

- `Official`: VDJScript verbs appendix

### `get_deck_color`

Aliases: none

Kind: `Query`

Typical surfaces: `SkinQuery`, `Text`, `Button`

Official summary:

- Return blue or red if the deck is the left or right deck, and gray otherwise

Typical forms:

```vdjscript
get_deck_color
get_deck_color 50%
get_deck_color "absolute"
get_deck_color "absolute" 50%
```

Important note:

- Use `"absolute"` when you want color based on the actual deck number rather than the current left/right assignment.

Sources:

- `Official`: VDJScript verbs appendix

### `play_button`

Aliases: `play_3button`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Act like `play_stutter` or `play_pause` depending on the `playMode` setting

Typical forms:

```vdjscript
play_button
```

Preferred usage:

- use only when you intentionally want behavior that follows the user's `playMode`
- for documentation and fixed examples, prefer `play_pause` or `play_stutter` explicitly

Sources:

- `Official`: VDJScript verbs appendix

### `pause_stop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- If playing, pause the deck. If already stopped, rewind to the beginning, then cycle cues on repeated presses.

Typical forms:

```vdjscript
pause_stop
```

Preferred usage:

- use when you intentionally want the classic Numark-style stop behavior
- do not document it as interchangeable with plain `stop`

Sources:

- `Official`: VDJScript verbs appendix

### `stop_button`

Aliases: `stop_3button`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Act like `pause_stop` or `stop` depending on the `playMode` setting

Typical forms:

```vdjscript
stop_button
```

Preferred usage:

- use only when the mapping should follow `playMode`
- for deterministic docs and examples, prefer `stop` or `pause_stop` explicitly

Sources:

- `Official`: VDJScript verbs appendix

### `cue_play`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Like `cue_stop`, but if held long enough it continues playing when released

Typical forms:

```vdjscript
cue_play
cue_play 1 1000ms
```

Notes:

- The manual documents the hold behavior and allows an explicit time argument.

Sources:

- `Official`: VDJScript verbs appendix

### `cue`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- If playing, jump to the last cue point and keep playing. If paused, set the cue and preview while pressed.

Typical forms:

```vdjscript
cue
cue 1
cue 57
```

Notes:

- In loops, the manual says `cue` changes `loop_in` to the cue point while keeping the loop length.

Sources:

- `Official`: VDJScript verbs appendix

### `cue_select`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Select the default cue point used by cue-related actions without jumping to it

Typical forms:

```vdjscript
cue_select 1
cue_select +1
```

Preferred usage:

- use when you need cue-target selection separate from transport movement

Sources:

- `Official`: VDJScript verbs appendix

### `cue_cup`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- If playing, rewind to the last cue and restart on release. If paused, set the current position as cue.

Typical forms:

```vdjscript
cue_cup
```

Sources:

- `Official`: VDJScript verbs appendix

### `cue_button`

Aliases: `cue_3button`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Act like `cue_stop`, `cue_play`, or `cue_cup` depending on the `cueMode` setting

Typical forms:

```vdjscript
cue_button
```

Preferred usage:

- use when you intentionally want the mapping to follow `cueMode`
- for deterministic examples, document `cue_stop`, `cue_play`, or `cue_cup` directly

Sources:

- `Official`: VDJScript verbs appendix

### `goto_first_beat`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Jump to the first beat in the song

Typical forms:

```vdjscript
goto_first_beat
```

Sources:

- `Official`: VDJScript verbs appendix

### `goto_start`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Go to the start of the song

Typical forms:

```vdjscript
goto_start
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Set or remove a loop

Typical forms:

```vdjscript
loop 4
loop 0.5
loop 10ms
loop 200%
loop
```

Preferred usage:

- use when one control should create, resize, or toggle a loop directly
- use `loop_in`, `loop_out`, `loop_length`, and `loop_move` when the UI has separate controls for loop lifecycle and loop size

Sources:

- `Official`: VDJScript verbs appendix

### `loop_in`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- If not in loop, set the beginning of a loop. If already in loop, jump back to the loop start.

Typical forms:

```vdjscript
loop_in
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop_out`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- If not in loop, enter a loop using the last `loop_in` point or stutter point. If already in loop, exit it.

Typical forms:

```vdjscript
loop_out
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop_length`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Change the loop length in milliseconds, beats, or percentage of the current length

Typical forms:

```vdjscript
loop_length 0.5
loop_length 15ms
loop_length +100%
```

Preferred usage:

- use for deterministic loop-size controls and encoders

Sources:

- `Official`: VDJScript verbs appendix

### `loop_move`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Move the loop while keeping its current length

Typical forms:

```vdjscript
loop_move +2
loop_move +10ms
loop_move +50%
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop_double`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Double the current loop length

Typical forms:

```vdjscript
loop_double
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop_half`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Halve the current loop length

Typical forms:

```vdjscript
loop_half
```

Sources:

- `Official`: VDJScript verbs appendix

### `loop_exit`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Remove the loop

Typical forms:

```vdjscript
loop_exit
```

Sources:

- `Official`: VDJScript verbs appendix

### `reloop`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Jump to the stored `loop_in` point

Typical forms:

```vdjscript
reloop
```

Sources:

- `Official`: VDJScript verbs appendix

### `reloop_exit`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- If in loop, remove it. Otherwise, reactivate the last used loop.

Typical forms:

```vdjscript
reloop_exit
```

Notes:

- The official text also notes that it highlights when a loop had been used.

Sources:

- `Official`: VDJScript verbs appendix

### `loop_roll`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Trigger a loop roll of the specified size

Typical forms:

```vdjscript
loop_roll 0.25
loop_roll video
```

Sources:

- `Official`: VDJScript verbs appendix

### `pad_page_select`

Aliases: `pad_page_favorite_select`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Select the pad page assigned to a favorite slot

Typical forms:

```vdjscript
pad_page_select 1
```

Sources:

- `Official`: VDJScript verbs appendix

### `sync`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`

Official summary:

- Smoothly synchronize the song with the other deck

Typical forms:

```vdjscript
sync
```

Preferred usage:

- use when you want standard sync behavior that follows the current sync engine rather than an immediate start-and-play action

Sources:

- `Official`: VDJScript verbs appendix

### `match_bpm`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Set the pitch to match the BPM of the other deck

Typical forms:

```vdjscript
match_bpm
```

Sources:

- `Official`: VDJScript verbs appendix

### `play_sync`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Play the song instantly synchronized with the other deck

Typical forms:

```vdjscript
play_sync
```

Preferred usage:

- use when the action should both sync and start playback immediately
- do not confuse this with the `smart_play` setting/action

Sources:

- `Official`: VDJScript verbs appendix

### `play_sync_onbeat`

Aliases: `sync_nocbg`

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Instantly synchronize using local beat information instead of the global beatgrid

Typical forms:

```vdjscript
play_sync_onbeat
```

Preferred usage:

- call out the alias because older scripts and forum posts often reference `sync_nocbg`

Sources:

- `Official`: VDJScript verbs appendix

### `beatlock`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`, `SkinAction`, `SkinQuery`

Official summary:

- Keep songs synchronized even while moving pitch, scratching, and similar manipulations

Typical forms:

```vdjscript
beatlock
beatlock on
beatlock off
```

Sources:

- `Official`: VDJScript verbs appendix

### `smart_fader`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Synchronize songs while using the crossfader and gradually move tempo toward the target side

Typical forms:

```vdjscript
smart_fader
```

Sources:

- `Official`: VDJScript verbs appendix

### `smart_play`

Aliases: `auto_sync`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- When enabled, songs are automatically synchronized when started

Typical forms:

```vdjscript
smart_play
smart_play on
smart_play off
```

Important note:

- This is a setting-like behavior toggle, not the same thing as the `play_sync` transport action.

Sources:

- `Official`: VDJScript verbs appendix

### `phrase_sync`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `Pad`

Official summary:

- Shift by a number of beats to match the phrase of the other deck

Typical forms:

```vdjscript
phrase_sync
phrase_sync 16
```

Sources:

- `Official`: VDJScript verbs appendix

### `quantize_all`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Set all quantize options

Typical forms:

```vdjscript
quantize_all
```

Sources:

- `Official`: VDJScript verbs appendix

### `is_fluid`

Aliases: `has_variable_bpm`

Kind: `Query`

Typical surfaces: `Map`, `Button`, `SkinQuery`, `Text`

Official summary:

- Return true if the song uses a fluid grid

Typical forms:

```vdjscript
is_fluid
```

Sources:

- `Official`: VDJScript verbs appendix

### `set_fluid`

Aliases: `set_variable_bpm`

Kind: `Action`

Typical surfaces: `Map`, `Button`

Official summary:

- Switch between fluid and rigid grids

Typical forms:

```vdjscript
set_fluid
```

Sources:

- `Official`: VDJScript verbs appendix

### `goto_last_folder`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Go back to the last browsed folder

Typical forms:

```vdjscript
goto_last_folder
```

Preferred usage:

- use when a mapping or skin action needs deterministic browser back-navigation without simulating repeated scrolls

Sources:

- `Official`: VDJScript verbs appendix

### `browser_scroll`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Scroll through the songs or folders

Typical forms:

```vdjscript
browser_scroll +1
browser_scroll -1
browser_scroll 'top'
browser_scroll 'bottom'
```

Preferred usage:

- use for encoders, list navigation buttons, and timed repeat browsing
- pair with `repeat_start` or `repeat_start_instant` when the control should keep scrolling while held

Sources:

- `Official`: VDJScript verbs appendix

### `browser_move`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Move the currently selected song inside a playlist

Typical forms:

```vdjscript
browser_move +1
browser_move 'top'
browser_move 'bottom'
```

Important note:

- Treat this as playlist reordering, not general browser navigation.

Sources:

- `Official`: VDJScript verbs appendix

### `browser_folder`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- If focus is on songs, change focus to folders. If focus is on folders, open or close the subfolders of the selected folder.

Typical forms:

```vdjscript
browser_folder
```

Preferred usage:

- use when you want a single control to hand off focus from song list to folder tree

Sources:

- `Official`: VDJScript verbs appendix

### `browser_enter`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- If focus is on songs, load the selected song. If focus is on folders, change focus to songs.

Typical forms:

```vdjscript
browser_enter
```

Important note:

- This is focus-sensitive. If you need a guaranteed load action regardless of browser focus, prefer `load`.

Sources:

- `Official`: VDJScript verbs appendix
- `Inference`: deterministic API guidance based on the documented focus-dependent behavior

### `browser_open_folder`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Expand the selected folder when closed, or close it when opened

Typical forms:

```vdjscript
browser_open_folder
browser_open_folder on
browser_open_folder off
```

Preferred usage:

- use this when you need explicit folder-tree open or close behavior without also switching song focus

Sources:

- `Official`: VDJScript verbs appendix

### `browser_remove`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Remove the selected song from playlist

Typical forms:

```vdjscript
browser_remove
```

Important note:

- The documented behavior is playlist removal, not deletion from the library or filesystem.

Sources:

- `Official`: VDJScript verbs appendix

### `browser_window`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Change the active browser zone

Typical forms:

```vdjscript
browser_window 'folders'
browser_window 'songs'
browser_window 'sideview'
browser_window 'automix'
browser_window +1
browser_window 'folders,songs'
```

Preferred usage:

- use this to move focus between browser panes
- use `sideview` when the goal is to choose which sideview is shown, not just move focus to the sideview pane

Sources:

- `Official`: VDJScript verbs appendix

### `search`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Put keyboard focus on the search zone, or, if a text parameter is specified, search for this text

Typical forms:

```vdjscript
search
search 'house'
```

Preferred usage:

- use `search 'text'` when you want deterministic scripted search input
- use `edit_search` when you want keyboard focus without replacing the current query

Sources:

- `Official`: VDJScript verbs appendix

### `search_add`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Add the specified text to the search query

Typical forms:

```vdjscript
search_add 'acapella'
```

Preferred usage:

- use when a button should append a token or fragment without discarding the existing search string

Sources:

- `Official`: VDJScript verbs appendix

### `search_delete`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Remove the last character from the search query

Typical forms:

```vdjscript
search_delete
```

Sources:

- `Official`: VDJScript verbs appendix

### `clear_search`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Clear the search string

Typical forms:

```vdjscript
clear_search
```

Sources:

- `Official`: VDJScript verbs appendix

### `edit_search`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Put keyboard focus in the search zone but keep the actual search string

Typical forms:

```vdjscript
edit_search
```

Preferred usage:

- prefer this over plain `search` when the current query must be preserved

Sources:

- `Official`: VDJScript verbs appendix

### `file_count`

Aliases: none

Kind: `Query`

Typical surfaces: `Text`, `SkinQuery`, `Button`

Official summary:

- Get the number of files currently shown in browser

Typical forms:

```vdjscript
`file_count`
`file_count automix`
```

Important note:

- The official verbs page also documents `automix`, `sideview`, `karaoke`, and `sidelist` as valid count targets.

Sources:

- `Official`: VDJScript verbs appendix

### `sideview`

Aliases: none

Kind: `Action`

Typical surfaces: `Map`, `Button`, `SkinAction`

Official summary:

- Show a specific folder in the sideview

Typical forms:

```vdjscript
sideview automix
sideview sampler
sideview +1
sideview -1
```

Important note:

- Use `sideview` to select the sideview content.
- Use `browser_window 'sideview'` when the goal is to move focus to the sideview pane.

Sources:

- `Official`: VDJScript verbs appendix

### `sideview_title`

Aliases: none

Kind: `Query`

Typical surfaces: `Text`, `SkinQuery`

Official summary:

- Show the title of the folder selected in sideview

Typical forms:

```vdjscript
`sideview_title`
```

Preferred usage:

- useful for skin labels and browser helper text that should reflect the active sideview source

Sources:

- `Official`: VDJScript verbs appendix

### `rating`

Aliases: none

Kind: `Dual`

Typical surfaces: `Map`, `Button`, `SkinAction`, `Text`

Official summary:

- Get or set the rating for the current song

Typical forms:

```vdjscript
`rating`
rating 4
```

Preferred usage:

- use the query form for display
- use the action form for explicit rating controls on the current song

Sources:

- `Official`: VDJScript verbs appendix

### `add_list`

Aliases: `add_virtualfolder`

Kind: `Action`

Typical surfaces: `Button`, `SkinAction`

Official summary:

- Create a new list (virtual folder)

Typical forms:

```vdjscript
add_list
```

Sources:

- `Official`: VDJScript verbs appendix

### `info_options`

Aliases: `infos_options`

Kind: `Action`

Typical surfaces: `Button`, `SkinAction`

Official summary:

- Show the context menu about the info panel fields and prelisten behavior

Typical forms:

```vdjscript
info_options
```

Sources:

- `Official`: VDJScript verbs appendix

### `browser_options`

Aliases: none

Kind: `Action`

Typical surfaces: `Button`, `SkinAction`

Official summary:

- Show the context menu about browser filters, root folders, database, and related browser settings

Typical forms:

```vdjscript
browser_options
```

Sources:

- `Official`: VDJScript verbs appendix

### `browser_export`

Aliases: none

Kind: `Action`

Typical surfaces: `Button`, `SkinAction`

Official summary:

- Export the current list of files to a CSV or HTML file

Typical forms:

```vdjscript
browser_export
```

Sources:

- `Official`: VDJScript verbs appendix

## Broad Verb Index

The sections below remain useful as a wide local inventory. They are still being normalized to the same API standard used above, especially around aliases, surface notes, and source-status markers.

## Flow Control

| Verb       | Description                         | Example                         |
| ---------- | ----------------------------------- | ------------------------------- |
| `nothing`  | Do nothing                          | `nothing`                       |
| `up`       | Execute action on key press/release | `up ? action1 : action2`        |
| `down`     | Execute action on key press/release | `down ? action1 : action2`      |
| `isrepeat` | Check if key is auto-repeating      | `isrepeat ? nothing : goto_cue` |

## Parameters & Constants

| Verb                        | Description                    | Example                                                |
| --------------------------- | ------------------------------ | ------------------------------------------------------ |
| `true` / `on` / `yes`       | Returns true                   | `true`                                                 |
| `false` / `no` / `off`      | Returns false                  | `false`                                                |
| `constant` / `get_constant` | Return specified value         | `get constant 75%`                                     |
| `dim`                       | Equivalent to `constant 0.1`   | `dim`                                                  |
| `color_mix`                 | Mix two colors based on action | `color_mix white red \`get_limiter\``                  |
| `color`                     | Return color value             | `color "red"`, `color "#C08040"`, `color 0.8 0.5 0.25` |

## Parameter Comparison & Math

| Verb                             | Description                     | Example                                           |
| -------------------------------- | ------------------------------- | ------------------------------------------------- |
| `param_bigger` / `param_greater` | Check if value is bigger        | `param_bigger 0 ? action1 : action2`              |
| `param_equal`                    | Check if value equals something | `param_equal \`get_browsed_song 'type'\` "audio"` |
| `param_contains`                 | Check if value contains string  | `param_contains`                                  |
| `param_smaller`                  | Check if value is smaller       | `param_smaller 0 ? action1 : action2`             |
| `param_add`                      | Add values                      | `param_add \`get_var a\` \`get_var b\``           |
| `param_multiply`                 | Multiply value                  | `param_multiply 300% & effect slider`             |
| `param_1_x`                      | Invert value (1/x)              | `param_1_x & effect slider`                       |
| `param_pow`                      | Power calculation               | `param_pow 0.5` (square root)                     |
| `param_invert`                   | Invert value (1-x)              | `param_invert & pitch_slider`                     |
| `param_mod`                      | Wrap value                      | `param_mod`                                       |
| `param_pingpong`                 | Linear to forth-and-back scale  | `param_pingpong`                                  |
| `param_cast`                     | Cast to new type                | `param_cast "percentage"`                         |
| `param_delta`                    | Transform absolute to relative  | `param_delta`                                     |
| `param_uppercase`                | Convert to uppercase            | `param_uppercase`                                 |
| `param_lowercase`                | Convert to lowercase            | `param_lowercase`                                 |
| `param_ucfirst`                  | First letter uppercase          | `param_ucfirst`                                   |

### Cast Types

- `integer`, `float`, `percentage`, `ms`, `boolean`, `beats`, `text`
- `int_trunc` - integer part without rounding
- `frac` - decimal part
- `relative`, `absolute` - change parameter type

## Timing & Animation

| Verb                  | Description                              | Example                               |
| --------------------- | ---------------------------------------- | ------------------------------------- |
| `blink`               | Toggle LED on/off                        | `blink 1000ms`, `blink 1bt`           |
| `fadeout`             | Fade out when condition ends             | `fadeout 10000ms 3000ms \`loop\``     |
| `pulse`               | True for duration when action turns true | `is_using 'equalizer' & pulse 2000ms` |
| `param_make_discrete` | Make smooth encoder discrete             | `param_make_discrete 0.1`             |

## Repeat & Delay

| Verb                   | Description                 | Example                                 |
| ---------------------- | --------------------------- | --------------------------------------- |
| `repeat`               | Repeat action while pressed | `repeat 1000ms & browser_scroll +1`     |
| `repeat_start`         | Start repeating action      | `repeat_start 'name' 1000ms 5 & action` |
| `repeat_start_instant` | Start repeating immediately | `repeat_start_instant 'name' 1000ms`    |
| `repeat_stop`          | Stop repeat                 | `repeat_stop 'name'`                    |
| `wait`                 | Wait between actions        | `wait 1bt & pause`                      |
| `holding`              | Execute if held long        | `holding ? automix : mix_now`           |
| `doubleclick`          | Execute if double-clicked   | `doubleclick ? automix : mix_now`       |

## Skin Control

| Verb                        | Description             | Example                                   |
| --------------------------- | ----------------------- | ----------------------------------------- |
| `skin_panel`                | Show/hide panel         | `skin_panel 'my_panel' on`                |
| `skin_panelgroup`           | Change panel in group   | `skin_panelgroup 'groupname' 'panelname'` |
| `skin_panelgroup_available` | Set panel availability  | `skin_panelgroup_available`               |
| `lock_panel`                | Acts on split elements  | `lock_panel`                              |
| `show_splitpanel`           | Show/hide split panel   | `show_splitpanel 'sidelist'`              |
| `rack`                      | Open/close rack unit    | `rack 'rack1' 'unit1'`                    |
| `rack_solo`                 | Open unit full size     | `rack_solo 'rack1' 'unit1'`               |
| `rack_prioritize`           | Prioritize unit         | `rack_prioritize 'rack1' 'unit1'`         |
| `zoom` / `zoom_scratch`     | Zoom horizontal         | `zoom`                                    |
| `zoom_vertical`             | Zoom vertical           | `zoom_vertical`                           |
| `load_skin`                 | Load new skin/variation | `load_skin ':newvariation'`               |

## Custom Buttons & Multi-buttons

| Verb                 | Description          | Example                          |
| -------------------- | -------------------- | -------------------------------- |
| `custom_button`      | Custom button action | `custom_button`                  |
| `custom_button_name` | Get/set button name  | `custom_button_name`             |
| `has_custom_button`  | Check if has action  | `has_custom_button`              |
| `custom_button_edit` | Open editor          | `custom_button_edit`             |
| `multibutton`        | Click multibutton    | `multibutton "my_button"`        |
| `multibutton_select` | Open selection menu  | `multibutton_select "my_button"` |

## System Info

| Verb                   | Description              | Example                             |
| ---------------------- | ------------------------ | ----------------------------------- |
| `get_cpu`              | CPU activity             | `get_cpu`                           |
| `get_clock`            | Current time             | `get_clock`, `get_clock 12` (AM/PM) |
| `get_date`             | Current date             | `get_date "%Y/%m/%d"`               |
| `is_pc` / `is_windows` | Check if PC              | `is_pc`                             |
| `is_mac` / `is_macos`  | Check if Mac             | `is_mac`                            |
| `has_notch`            | Check for display notch  | `has_notch`                         |
| `get_battery`          | Battery level            | `get_battery`                       |
| `is_battery`           | Running on battery       | `is_battery`                        |
| `has_battery`          | Has batteries            | `has_battery`                       |
| `show_keyboard`        | Show onscreen keyboard   | `show_keyboard`                     |
| `system_volume`        | Change system volume     | `system_volume`                     |
| `has_system_volume`    | Can modify system volume | `has_system_volume`                 |

## Variables

| Verb             | Description                   | Example                                     |
| ---------------- | ----------------------------- | ------------------------------------------- |
| `var`            | Conditional based on variable | `var "my_var" ? action1 : action2`          |
| `var_equal`      | Check equality                | `var_equal "my_var" 42 ? action1 : action2` |
| `var_not_equal`  | Check inequality              | `var_not_equal "my_var" 42`                 |
| `var_smaller`    | Check less than               | `var_smaller "my_var" 42`                   |
| `var_greater`    | Check greater than            | `var_greater "my_var" 42`                   |
| `set_var_dialog` | Dialog to set var             | `set_var_dialog 'varname'`                  |
| `set`            | Set variable value            | `set 'varname' 5`                           |
| `toggle`         | Toggle true/false             | `toggle "my_var"`                           |
| `cycle`          | Increment with wrap           | `cycle "my_var" 42`                         |
| `get_var`        | Get variable value            | `get_var "varname"`                         |
| `set_var`        | Set variable value            | `set_var`                                   |
| `var_list`       | Show variables window         | `var_list`                                  |
| `controllervar`  | Controller-unique variable    | `controllervar`                             |

## Window Control

| Verb          | Description                  | Example                 |
| ------------- | ---------------------------- | ----------------------- |
| `close`       | Close application            | `close`                 |
| `minimize`    | Minimize to taskbar          | `minimize`              |
| `maximize`    | Maximize/fullscreen/windowed | `maximize 'fullscreen'` |
| `show_window` | Show/hide window             | `show_window`           |

## Audio Playback

| Verb              | Description                 | Example                             |
| ----------------- | --------------------------- | ----------------------------------- |
| `song_pos`        | Position in song (slider)   | `song_pos`                          |
| `goto`            | Change position             | `goto +10ms`, `goto -4`, `goto 20%` |
| `goto_bar`        | Jump to beat after downbeat | `goto_bar 4`                        |
| `songpos_remain`  | Remaining time              | `songpos_remain 500ms ? blink`      |
| `songpos_warning` | Last 30s warning            | `songpos_warning`                   |
| `seek`            | Move while pressed          | `seek +2`, `seek +420ms`            |
| `reverse`         | Play backward               | `reverse`                           |
| `dump`            | Reverse temporarily         | `dump`, `dump quantized`            |
| `goto_first_beat` | Jump to first beat          | `goto_first_beat`                   |
| `goto_start`      | Go to start                 | `goto_start`                        |

## Deck Management

| Verb                 | Description             | Example                    |
| -------------------- | ----------------------- | -------------------------- |
| `swap_decks`         | Swap deck 1 and 2       | `swap_decks`               |
| `clone_deck`         | Clone deck              | `clone_deck`               |
| `clone_from_deck`    | Clone from other deck   | `clone_from_deck`          |
| `move_deck`          | Move song to other deck | `move_deck`                |
| `stems_split`        | Split stems to decks    | `stems_split vocal target` |
| `stems_split_unlink` | Unlink split stems      | `stems_split_unlink`       |
| `dualdeckmode`       | Toggle dual deck mode   | `dualdeckmode`             |
| `beatjump`           | Jump beats              | `beatjump +1`              |
| `beatjump_select`    | Set jump size           | `beatjump_select 4`        |
| `beatjump_page`      | Change jump offset      | `beatjump_page`            |
| `beatjump_pad`       | Execute jump            | `beatjump_pad`             |

## Play Controls

| Verb             | Description           | Example          |
| ---------------- | --------------------- | ---------------- |
| `play`           | Start deck            | `play`           |
| `play_stutter`   | Start or restart      | `play_stutter`   |
| `play_pause`     | Toggle play/pause     | `play_pause`     |
| `pause_stop`     | Pause or stop         | `pause_stop`     |
| `stop`           | Stop to cue/beginning | `stop`           |
| `pause`          | Pause deck            | `pause`          |
| `play_button`    | Depends on play_mode  | `play_button`    |
| `stop_button`    | Depends on play_mode  | `stop_button`    |
| `emergency_play` | Play something        | `emergency_play` |

## Audio Inputs

| Verb                  | Description               | Example                   |
| --------------------- | ------------------------- | ------------------------- |
| `mic` / `microphone`  | Toggle microphone         | `mic`                     |
| `mic_talkover`        | Lower decks, activate mic | `mic_talkover 20% 1000ms` |
| `mic_eq_low/mid/high` | Mic EQ                    | `mic_eq_low`              |
| `mic_volume`          | Set mic volume            | `mic_volume`              |
| `linein`              | Activate line input       | `deck 1 linein 2 on`      |
| `linein_rec`          | Record line input         | `linein_rec`              |
| `mic_rec`             | Record microphone         | `mic_rec`                 |

## Scratch & Jogwheel

| Verb                          | Description            | Example                  |
| ----------------------------- | ---------------------- | ------------------------ |
| `touchwheel` / `scratchwheel` | Jogwheel with touch    | `touchwheel +1.0`        |
| `touchwheel_touch`            | Touch detection        | `touchwheel_touch`       |
| `jogwheel` / `jog`            | Jogwheel without touch | `jogwheel +1.0`          |
| `motorwheel`                  | Motorized jogwheel     | `motorwheel "move" +1.0` |
| `speedwheel`                  | Position and speed     | `speedwheel +1.0 1.5`    |
| `vinyl_mode`                  | Set vinyl/CD mode      | `vinyl_mode`             |
| `wheel_mode`                  | Change wheel mode      | `wheel_mode +1`          |
| `hold` / `scratch_hold`       | Stop for scratching    | `hold on`                |
| `scratch`                     | Scratch forward/back   | `scratch +120ms`         |
| `nudge`                       | Nudge position         | `nudge +120ms`           |
| `slip_mode`                   | Slip mode              | `slip_mode`              |
| `slip`                        | Global slip mode       | `slip`                   |
| `scratch_dna`                 | Execute DNA scratch    | `scratch_dna`            |
| `scratch_dna_editor`          | Open DNA editor        | `scratch_dna_editor`     |

## Volume & Mixing

| Verb               | Description       | Example                      |
| ------------------ | ----------------- | ---------------------------- |
| `crossfader`       | Move crossfader   | `crossfader 50%`             |
| `auto_crossfade`   | Auto crossfade    | `auto_crossfade 2000ms`      |
| `level` / `volume` | Set deck volume   | `level`                      |
| `mute`             | Mute deck         | `mute`                       |
| `gain`             | Set gain          | `gain`                       |
| `set_gain`         | Set gain to dBA   | `set_gain 0`                 |
| `master_volume`    | Master volume     | `master_volume`              |
| `headphone_volume` | Headphone volume  | `headphone_volume`           |
| `headphone_mix`    | PFL mix           | `headphone_mix`              |
| `crossfader_curve` | Crossfader curve  | `crossfader_curve "scratch"` |
| `get_limiter`      | Check compression | `get_limiter`                |
| `get_level`        | Signal level      | `get_level`                  |
| `get_vu_meter`     | VU meter level    | `get_vu_meter`               |
| `is_audible`       | Deck on-air       | `is_audible`                 |

## Automix

| Verb                   | Description            | Example                |
| ---------------------- | ---------------------- | ---------------------- |
| `automix`              | Start/stop automix     | `automix`              |
| `automix_dualdeckmode` | Use both decks         | `automix_dualdeckmode` |
| `automix_skip`         | Skip current song      | `automix_skip`         |
| `mix_now`              | Crossfade with sync    | `mix_now 4000ms`       |
| `mix_now_nosync`       | Crossfade without sync | `mix_now_nosync`       |
| `mix_selected`         | Mix to selected        | `mix_selected`         |
| `mix_next`             | Mix to next            | `mix_next`             |
| `mix_and_load_next`    | Mix and load next      | `mix_and_load_next`    |
| `playlist_randomize`   | Shuffle playlist       | `playlist_randomize`   |
| `playlist_repeat`      | Repeat playlist        | `playlist_repeat`      |
| `playlist_clear`       | Empty playlist         | `playlist_clear`       |

## Browser

| Verb                   | Description                 | Example                      |
| ---------------------- | --------------------------- | ---------------------------- |
| `browser_scroll`       | Scroll songs/folders        | `browser_scroll +1`          |
| `browser_move`         | Move song in playlist       | `browser_move +1`            |
| `browser_folder`       | Focus folders               | `browser_folder`             |
| `browser_enter`        | Load or focus songs         | `browser_enter`              |
| `browser_open_folder`  | Expand/collapse folder      | `browser_open_folder`        |
| `browser_remove`       | Remove from playlist        | `browser_remove`             |
| `browser_window`       | Change browser zone         | `browser_window 'folders'`   |
| `search`               | Focus search or search text | `search "text"`              |
| `clear_search`         | Clear search                | `clear_search`               |
| `browser_gotofolder`   | Go to folder                | `browser_gotofolder "/path"` |
| `browser_sort`         | Sort browser                | `browser_sort "artist"`      |
| `grid_view`            | Grid view mode              | `grid_view`                  |
| `file_info`            | Open tag editor             | `file_info`                  |
| `browsed_file_color`   | Set file color              | `browsed_file_color "red"`   |
| `browsed_file_analyze` | Reanalyze file              | `browsed_file_analyze`       |

## Loading

| Verb            | Description         | Example               |
| --------------- | ------------------- | --------------------- |
| `load`          | Load song           | `load`, `load "path"` |
| `load_pulse`    | Brief pulse on load | `load_pulse`          |
| `loaded`        | Check if loaded     | `loaded`              |
| `undo_load`     | Reload previous     | `undo_load`           |
| `unload`        | Unload song         | `unload`              |
| `load_next`     | Load next track     | `load_next`           |
| `load_previous` | Load previous track | `load_previous`       |

## Cue Points

| Verb                 | Description              | Example                  |
| -------------------- | ------------------------ | ------------------------ |
| `cue_stop`           | Cue with preview         | `cue_stop`, `cue_stop 1` |
| `cue_play`           | Cue with hold-to-play    | `cue_play 1 1000ms`      |
| `cue`                | Jump to cue              | `cue`, `cue 1`           |
| `hot_cue` / `hotcue` | Set or jump to cue       | `hot_cue 1`              |
| `silent_cue`         | Mute until cue activated | `silent_cue`             |
| `cue_select`         | Select default cue       | `cue_select`             |
| `set_cue`            | Store cue position       | `set_cue 1 500ms`        |
| `goto_cue`           | Jump to cue              | `goto_cue 1`             |
| `delete_cue`         | Delete cue               | `delete_cue 1`           |
| `cue_pos`            | Get cue position         | `cue_pos 1`              |
| `cue_name`           | Get/set cue name         | `cue_name 1`             |
| `has_cue`            | Check if cue exists      | `has_cue 1`              |
| `cue_color`          | Get/set cue color        | `cue_color 1 'yellow'`   |
| `cue_loop`           | Jump and loop            | `cue_loop`               |
| `lock_cues`          | Lock/unlock cues         | `lock_cues`              |

### Cue Point Notes

- `cue_pos <n>` returns the position of cue point `<n>` as a percentage of the track, which makes it especially useful anywhere a skin element expects a progress-style value.
- `cue_pos` also supports alternate outputs such as `msec`, `sec`, `min`, `mseconly`, and `beats`.
- In skin XML, pair `cue_pos` with `has_cue <n>` when you want a marker or fill to appear only after that cue exists.

### Working `cue_pos` Examples

Basic queries:

```text
cue_pos 1
cue_pos 1 beats
cue_pos 1 mseconly
```

Custom progress bar up to Hot Cue 1:

```xml
<group name="cue_1_progress">
  <square color="#11161D">
    <pos x="40" y="200"/>
    <size width="300" height="6"/>
  </square>

  <visual source="cue_pos 1" type="linear" orientation="horizontal" visibility="has_cue 1">
    <pos x="40" y="200"/>
    <size width="300" height="6"/>
    <off shape="square" color="transparent"/>
    <on shape="square" color="`cue_color 1`"/>
  </visual>
</group>
```

This draws a thin bar from the start of the track up to Hot Cue 1. It is a simple way to turn `cue_pos` into a custom progress overlay above or below a `songpos` bar.

Read-only cue marker driven by a slider:

```xml
<slider action="cue_pos 1" orientation="horizontal" visibility="has_cue 1">
  <pos x="40" y="192"/>
  <size width="300" height="18"/>
  <fader>
    <size width="3" height="18"/>
    <off shape="square" color="`cue_color 1`"/>
  </fader>
</slider>
```

Here the slider range becomes a simple placement track, and the `fader` sits at the cue location. This works well as a thin overlay on top of a `songpos` bar when you want a marker instead of a fill.

## Deck Selection

| Verb              | Description                | Example              |
| ----------------- | -------------------------- | -------------------- |
| `select`          | Select working deck        | `select`             |
| `masterdeck`      | Select/unselect master     | `masterdeck`         |
| `masterdeck_auto` | Auto masterdeck            | `masterdeck_auto`    |
| `leftdeck`        | Select left deck           | `leftdeck +1`        |
| `rightdeck`       | Select right deck          | `rightdeck +1`       |
| `invert_deck`     | Swap left/right deck       | `invert_deck`        |
| `leftcross`       | Assign to left crossfader  | `leftcross`          |
| `rightcross`      | Assign to right crossfader | `rightcross`         |
| `pfl`             | Send to headphones         | `pfl`, `pfl 75%`     |
| `get_deck_color`  | Get deck color             | `get_deck_color 50%` |

## Equalizer & Stems

| Verb                   | Description           | Example                           |
| ---------------------- | --------------------- | --------------------------------- |
| `eq_mode`              | Select EQ behavior    | `eq_mode +1`, `eq_mode frequency` |
| `mute_stem`            | Mute stem             | `mute_stem vocal`                 |
| `only_stem`            | Isolate stem          | `only_stem vocal`                 |
| `stem_pad`             | Mute/isolate stem pad | `stem_pad vocal`                  |
| `has_stems`            | Check if has stems    | `has_stems "ready"`               |
| `eq_high`              | High EQ/HiHat/Vocal   | `eq_high`                         |
| `eq_mid`               | Mid EQ/Melody/Vocals  | `eq_mid`                          |
| `eq_low`               | Low EQ/Kick           | `eq_low`                          |
| `stem`                 | Control stem amount   | `stem "vocal" 50%`                |
| `eq_kill_high/mid/low` | Kill EQ band          | `eq_kill_high`                    |
| `filter`               | Apply color FX        | `filter`                          |
| `filter_selectcolorfx` | Select color effect   | `filter_selectcolorfx 'Reverb'`   |

`filter_selectcolorfx`: pops up a gui selector for colorfx effects
`filter_selectcolorfx 'Echo'`: selects the echo effect
`filter_selectcolorfx 'Filter'`: selects the filter effect

### Stem Names

- Individual: `Vocal`, `HiHat`, `Bass`, `Instru`, `Kick`
- Aggregate: `Melody` (Instru+Bass), `Rhythm` (HiHat+Kick), `MeloRhythm`, `Acapella`, `Instrumental`

## Get (Query Actions)

| Verb               | Description           | Example                       |
| ------------------ | --------------------- | ----------------------------- |
| `get_beatpos`      | Beat position         | `get_beatpos`                 |
| `get_bpm`          | Song BPM              | `get_bpm`, `get_bpm absolute` |
| `get_time`         | Elapsed time          | `get_time "remain" "short"`   |
| `get_rotation`     | Disc angle            | `get_rotation`                |
| `get_position`     | Song position         | `get_position`                |
| `get_deck`         | Deck number           | `get_deck`                    |
| `get_artist`       | Artist tag            | `get_artist`                  |
| `get_title`        | Title tag             | `get_title`                   |
| `get_album`        | Album tag             | `get_album`                   |
| `get_genre`        | Genre tag             | `get_genre`                   |
| `get_key`          | Song key              | `get_key "musical"`           |
| `get_browsed_song` | Browsed file property | `get_browsed_song 'title'`    |
| `get_loaded_song`  | Loaded file property  | `get_loaded_song 'album'`     |

## Karaoke

| Verb                    | Description             | Example                             |
| ----------------------- | ----------------------- | ----------------------------------- |
| `karaoke`               | Start/stop karaoke      | `karaoke`                           |
| `karaoke_show`          | Show singer list        | `karaoke_show`                      |
| `get_next_karaoke_song` | Get upcoming track info | `get_next_karaoke_song "singer" +1` |
| `is_karaoke_idle`       | Karaoke idle check      | `is_karaoke_idle`                   |
| `is_karaoke_playing`    | Karaoke playing check   | `is_karaoke_playing`                |

## Key & Pitch

| Verb                   | Description            | Example                     |
| ---------------------- | ---------------------- | --------------------------- |
| `key`                  | Change key (semitones) | `key +1`                    |
| `key_smooth`           | Change key (smooth)    | `key_smooth +0.5`           |
| `key_move`             | Move key by semitones  | `key_move +1`               |
| `set_key`              | Match exact key        | `set_key "A#m"`             |
| `match_key`            | Match compatible key   | `match_key`                 |
| `key_lock` / `keylock` | Lock key               | `key_lock`                  |
| `pitch`                | Set pitch              | `pitch 112%`, `pitch +0.1%` |
| `pitch_zero`           | Reset to 0%            | `pitch_zero`                |
| `pitch_reset`          | Slowly return to 0%    | `pitch_reset 5%`            |
| `pitch_range`          | Set pitch range        | `pitch_range 12%`           |
| `pitch_bend`           | Temporary bend         | `pitch_bend +3%`            |
| `master_tempo`         | Toggle master tempo    | `master_tempo`              |
| `get_pitch`            | Get pitch value        | `get_pitch`                 |

## Loops

| Verb          | Description          | Example                            |
| ------------- | -------------------- | ---------------------------------- |
| `loop`        | Set/remove loop      | `loop 4`, `loop 10ms`, `loop 200%` |
| `loop_in`     | Set loop start       | `loop_in`                          |
| `loop_out`    | Set loop end         | `loop_out`                         |
| `loop_length` | Change loop length   | `loop_length 0.5`                  |
| `loop_move`   | Move loop            | `loop_move +2`                     |
| `loop_double` | Double loop          | `loop_double`                      |
| `loop_half`   | Halve loop           | `loop_half`                        |
| `loop_exit`   | Remove loop          | `loop_exit`                        |
| `reloop`      | Jump to loop start   | `reloop`                           |
| `reloop_exit` | Remove or reactivate | `reloop_exit`                      |
| `loop_save`   | Save loop            | `loop_save 1`, `loop_save "name"`  |
| `loop_load`   | Load saved loop      | `loop_load 1`                      |
| `saved_loop`  | Load or set loop     | `saved_loop 1`                     |
| `loop_roll`   | Loop roll            | `loop_roll 0.25`                   |
| `slicer`      | Slicer effect        | `slicer 1`                         |
| `loop_adjust` | Adjust loop with jog | `loop_adjust 'move'`               |

## Pads

| Verb               | Description             | Example                            |
| ------------------ | ----------------------- | ---------------------------------- |
| `pad`              | Activate pad            | `pad 1`                            |
| `pad_page`         | Activate page           | `pad_page 1`, `pad_page 'hotcues'` |
| `pad_edit`         | Edit page               | `pad_edit`                         |
| `pad_param`        | Change param 1          | `pad_param`                        |
| `pad_color`        | Get pad color           | `pad_color 1`                      |
| `pad_button_color` | Controller button color | `pad_button_color 1`               |
| `padfx`            | Activate named effect   | `padfx "echo" 40% 90%`             |
| `padfx_single`     | Activate single padfx   | `padfx_single "reverb"`            |

## Effects

| Verb                  | Description                         | Example                           |
| --------------------- | ----------------------------------- | --------------------------------- |
| `effect_select`       | Select effect (deactivate previous) | `effect_select 1 "echo"`          |
| `effect_select_multi` | Select effect (keep previous)       | `effect_select_multi 2 "flanger"` |
| `effect_active`       | Activate/deactivate                 | `effect_active 1 on`              |
| `effect_slider`       | Move effect slider                  | `effect_slider 1 2 50%`           |
| `effect_button`       | Press effect button                 | `effect_button 1 2`               |
| `video_fx_select`     | Select video effect                 | `video_fx_select "my_plugin"`     |
| `effect_beats`        | Set beat parameter                  | `effect_beats`                    |
| `get_effect_name`     | Get effect name                     | `get_effect_name`                 |

## POI & BPM

| Verb            | Description      | Example                        |
| --------------- | ---------------- | ------------------------------ |
| `beat_tap`      | Tap to set BPM   | `beat_tap`                     |
| `edit_poi`      | Open POI editor  | `edit_poi`                     |
| `edit_bpm`      | Open BPM editor  | `edit_bpm`                     |
| `set_bpm`       | Set BPM          | `set_bpm 129.3`, `set_bpm 50%` |
| `adjust_cbg`    | Adjust beat grid | `adjust_cbg +2`                |
| `set_firstbeat` | Set first beat   | `set_firstbeat`                |
| `reanalyze`     | Reanalyze file   | `reanalyze multi`              |

## Sampler

| Verb                         | Description                                                      | Example                                                   |
| ---------------------------- | ---------------------------------------------------------------- | --------------------------------------------------------- |
| `sampler_play`               | Play the selected or specified sample slot                       | `sampler_play 4`                                          |
| `sampler_play_stutter`       | Play sample and restart it from the beginning if already playing | `sampler_play_stutter 4`                                  |
| `sampler_play_stop`          | Play sample if stopped, or stop it if already playing            | `sampler_play_stop 4`                                     |
| `sampler_stop`               | Stop one sample or all currently playing samples                 | `sampler_stop 4`, `sampler_stop all`                      |
| `sampler_pad`                | Trigger the currently exposed sampler pad slot; in display/name contexts it can also return the visible pad label | `sampler_pad 1`, `` `sampler_pad 1` `` |
| `sampler_pad_shift`          | Stop a sample if playing, delete it otherwise                    | `sampler_pad_shift 1`                                     |
| `sampler_pad_page`           | Change/query the current 8-pad sampler window                    | `sampler_pad_page +1`, `sampler_pad_page -1`              |
| `sampler_assign`             | Assign a `.vdjsample` file to a slot                             | `sampler_assign 1 "/Samples/horn.vdjsample"`              |
| `sampler_loaded`             | Check whether the visible sampler pad slot currently has a sample loaded | `sampler_loaded 1`, `sampler_loaded 1 "auto"`     |
| `sampler_color`              | Get the color of the visible sampler pad slot                    | `sampler_color 1`                                         |
| `sampler_select`             | Select the default sampler slot for the deck                     | `sampler_select 5`, `sampler_select +1`                   |
| `sampler_position`           | Get the current playback position of the selected sample         | `sampler_position`                                        |
| `sampler_bank`               | Select or cycle sampler banks                                    | `sampler_bank "birthday"`, `sampler_bank +1`              |
| `sampler_mute`               | Mute or unmute a sample                                          | `sampler_mute 4`                                          |
| `sampler_edit`               | Open the Sample Editor for a sample                              | `sampler_edit 4`                                          |
| `sampler_mode`               | Set global or per-sample trigger mode                            | `sampler_mode 1 'stutter'`, `sampler_mode +1`             |
| `sampler_output`             | Route sampler output to master, trigger deck, headphones, etc.   | `sampler_output "headphones"`, `deck master sampler_output` |
| `sampler_options`            | Open or toggle sampler bank options                              | `sampler_options`, `sampler_options "locked"`             |
| `sampler_volume_master`      | Set the sampler master volume                                    | `sampler_volume_master +5%`                               |
| `sampler_pfl`                | Send sampler to headphones or set sampler PFL volume             | `sampler_pfl 75%`                                         |
| `sampler_volume`             | Set sample volume by absolute slot or sample name                | `sampler_volume 9 75%`, `sampler_volume "siren" 75%`      |
| `sampler_pad_volume`         | Set sample volume by visible sampler pad position                | `sampler_pad_volume 1 75%`                                |
| `sampler_volume_nogroup`     | Adjust one sample without also changing other samples in its group | `sampler_volume_nogroup 9 75%`                          |
| `sampler_group_volume`       | Adjust all samples in a sampler group                            | `sampler_group_volume "horns" 75%`                        |
| `sampler_loop`               | Change the loop length of a sample or set it explicitly          | `sampler_loop 1 1`, `sampler_loop +1`                     |
| `sampler_rec`                | Record a sample from the deck, mic, or master                    | `sampler_rec`, `sampler_rec "mic"`, `sampler_rec 1`       |
| `sampler_start_rec`          | Start recording a new sample                                     | `sampler_start_rec "master"`                              |
| `sampler_stop_rec`           | Stop recording and save the sample                               | `sampler_stop_rec`                                        |
| `sampler_abort_rec`          | Cancel recording and delete the unfinished sample                | `sampler_abort_rec`                                       |
| `sampler_rec_delete`         | Delete a sample from the Recordings bank                         | `sampler_rec_delete 3`                                    |
| `sampler_used` / `get_sampler_used` | Check whether any sample, or a specific count of samples, is playing | `sampler_used`, `sampler_used 4`                  |
| `get_sampler_slot`           | Get the sampler slot that currently has focus                    | `get_sampler_slot`                                        |
| `get_sampler_count`          | Get the number of slots in the current sampler bank              | `get_sampler_count`                                       |
| `get_sample_name`            | Get the name of an absolute sample slot                          | `get_sample_name 9`                                       |
| `get_sample_info`            | Read sample metadata such as `fullpath`, `group`, or `length`    | `get_sample_info 9 fullpath`                              |
| `get_sampler_bank`           | Get the name of the active sampler bank                          | `get_sampler_bank`                                        |
| `get_sampler_bank_id`        | Get the numeric id of the active sampler bank                    | `get_sampler_bank_id`                                     |
| `get_sampler_bank_count`     | Get the total number of sampler banks                            | `get_sampler_bank_count`                                  |
| `get_sample_color`           | Get the actual stored color of a sample slot                     | `get_sample_color 9`                                      |

### Sampler Modes

- `on/off` - One press starts the sample, the next press stops it, or it stops when it reaches the end.
- `hold` - The sample plays only while the pad is held.
- `stutter` - Each press restarts the sample from the beginning.
- `unmute` - The sample keeps running, but is only audible while the pad is held.

### Sampler Notes

- `sampler_pad_page` is the pager behind Parameter 2 on the default Sampler pad page and is the main way to reach `9-16`, `17-24`, and later sub-pages in banks with more than 8 samples.
- `sampler_pad`, `sampler_loaded`, `sampler_color`, and `sampler_pad_volume` are the safest page-aware helpers when building sampler pad pages.
- In display contexts such as pad `name=` fields and skin/text `format=` fields, `sampler_pad 1` through `sampler_pad 8` are the safest way to show the current visible sample names on the active sampler page.
- For visibility and empty-slot checks in paged sampler UIs, `sampler_loaded 1` through `sampler_loaded 8` already follow the visible sampler page, so you usually do not need to infer emptiness from a blank `sampler_pad` label.
- `sampler_play`, `sampler_stop`, `sampler_volume`, `get_sample_name`, `get_sample_info`, and `get_sample_color` are best treated as absolute-slot helpers.
- Use `sampler_color` when you want the color of the currently visible sampler pad. Use `get_sample_color` when you want the actual stored color of a specific bank slot.
- Samples triggered from a deck sync to that deck. If you want a pad page to follow a predictable sync source, trigger through an explicit deck:

```text
deck active sampler_pad 1 "auto"
deck master sampler_pad 1 "auto"
```

- `deck master` means the current master deck context, not a separate global sampler namespace.
- In skin XML, raw `deck master sampler_pad <n>` can be less reliable than an explicit deck number in some sampler title/query paths. If a paged sampler title shows the wrong slot, resolve the master deck explicitly:

```text
deck 1 masterdeck ? deck 1 sampler_pad 1 : deck 2 masterdeck ? deck 2 sampler_pad 1 : deck 3 masterdeck ? deck 3 sampler_pad 1 : deck 4 masterdeck ? deck 4 sampler_pad 1 : sampler_pad 1
```

- If you want the traditional left-deck `1-8` and right-deck `9-16` behavior, either page the second deck manually with `sampler_pad_page +1` or enable the `samplerSpanAcrossDecks` option.

### Working Sampler Examples

```text
sampler_pad 1
sampler_loaded 1 "auto" ? sampler_pad 1 "auto" : sampler_rec 1 "auto"
sampler_pad_page +1
sampler_bank +1
sampler_options "locked"
sampler_pad_volume 1 75%
sampler_volume 9 75%
```

### Sampler Source Notes

- Official verbs: [VDJScript verbs](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html)
- Default sampler page behavior: [Pads manual](https://www.virtualdj.com/manuals/virtualdj/interface/decks/decksadvanced/pads.html)
- Trigger modes and loop sync settings: [Sample Editor](https://www.virtualdj.com/manuals/virtualdj/editors/sampleeditor.html)
- Page-aware custom pad page examples: [Custom Sampler Pad Page](https://www.virtualdj.com/forums/253061/General_Discussion/Custom_Sampler_Pad_Page_%28Recording__Looping__Adjust_Beatgrid_and_more%29.html)
- Deck sync guidance: [problem with (pad pages) pads sampler sync](https://virtualdj.com/forums/224203/VirtualDJ_Technical_Support/problem_with_%28pad_pages%29_pads_sampler_sync%21_please_help___is_it_a_bug%3F%3F.html)
- Master-deck sampler quirks in newer builds: [Virtual Dj 2025 Sampler Sync](https://virtualdj.com/forums/265522/VirtualDJ_Technical_Support/Virtual_Dj_2025_Sampler_Sync.html)
- Paging and `9-16` behavior: [No longer possible to access 16 samples from controllers with 8 x 2 pads?](https://virtualdj.com/forums/261416/VirtualDJ_Technical_Support/No_longer_possible_to_access_16_samples_from_controllers_with_8_x_2_pads_.html)
- Explicit matrix/layout argument: [Using Xone K2 to control the sampler](https://www.virtualdj.com/forums/261102/VirtualDJ_Technical_Support/Using_Xone_K2_to_control_the_sampler.html)

## Sync

| Verb           | Description                 | Example          |
| -------------- | --------------------------- | ---------------- |
| `sync`         | Synchronize with other deck | `sync`           |
| `match_bpm`    | Match BPM only              | `match_bpm`      |
| `play_sync`    | Play synchronized           | `play_sync`      |
| `beatlock`     | Keep synchronized           | `beatlock`       |
| `smart_fader`  | Sync while crossfading      | `smart_fader`    |
| `phrase_sync`  | Match phrase                | `phrase_sync 16` |
| `quantize_all` | Set all quantize options    | `quantize_all`   |

## Video

| Verb               | Description             | Example                   |
| ------------------ | ----------------------- | ------------------------- |
| `leftvideo`        | Assign left video       | `leftvideo +1`            |
| `rightvideo`       | Assign right video      | `rightvideo +1`           |
| `video`            | Open/close video window | `video`                   |
| `video_output`     | Select monitor          | `video_output 1`          |
| `video_crossfader` | Video crossfader        | `video_crossfader`        |
| `video_transition` | Launch transition       | `video_transition 1000ms` |
| `is_video`         | Check if has video      | `is_video`                |

## Recording & Broadcasting

| Verb              | Description          | Example             |
| ----------------- | -------------------- | ------------------- |
| `record`          | Start recording      | `record`            |
| `record_cut`      | Cut to new file      | `record_cut`        |
| `broadcast`       | Start/stop broadcast | `broadcast "video"` |
| `get_record_time` | Recording time       | `get_record_time`   |

## Controllers

| Verb                | Description               | Example                                  |
| ------------------- | ------------------------- | ---------------------------------------- |
| `action_deck`       | Check button deck         | `action_deck 1 ? actionA : actionB`      |
| `set_deck`          | Affect which deck         | `set_deck \`get_var varname\` & play`    |
| `device_side`       | Left/right device action  | `device_side 'left' ? action1 : action2` |
| `assign_controller` | Assign controller to deck | `deck 1 assign_controller "CDJ400" 2`    |
| `shift`             | Built-in shift variable   | `shift`                                  |
| `menu_button`       | Changeable button         | `menu_button 1 "hotcue,sampler"`         |

## Configuration

| Verb                       | Description            | Example                               |
| -------------------------- | ---------------------- | ------------------------------------- |
| `settings` / `config`      | Open config window     | `settings`                            |
| `smart_loop`               | Auto-adjust loops      | `smart_loop`                          |
| `smart_play` / `auto_sync` | Auto-sync on play      | `smart_play`                          |
| `smart_cue`                | Auto-sync on cue       | `smart_cue`                           |
| `auto_match_bpm`           | Auto-match BPM on load | `auto_match_bpm`                      |
| `auto_match_key`           | Auto-match key on load | `auto_match_key`                      |
| `setting`                  | Read/write setting     | `setting "jogSensitivityScratch" 80%` |
| `save_config`              | Save config now        | `save_config`                         |

## Timecode

| Verb              | Description             | Example                 |
| ----------------- | ----------------------- | ----------------------- |
| `timecode_active` | Enable timecode control | `timecode_active 1 on`  |
| `timecode_mode`   | Set mode                | `timecode_mode 'smart'` |
| `timecode_bypass` | Use as line input       | `timecode_bypass`       |
| `get_hastimecode` | Check if has timecode   | `get_hastimecode`       |

## Macros

| Verb           | Description  | Example        |
| -------------- | ------------ | -------------- |
| `macro_record` | Record macro | `macro_record` |
| `macro_play`   | Play macro   | `macro_play`   |

## Sandbox

| Verb          | Description          | Example       |
| ------------- | -------------------- | ------------- |
| `sandbox`     | Toggle sandbox mode  | `sandbox`     |
| `can_sandbox` | Check if can sandbox | `can_sandbox` |

## Text Queries

| Verb        | Description             | Example                                                       |
| ----------- | ----------------------- | ------------------------------------------------------------- |
| `get_text`  | Get formatted text      | `get_text 'You are listening to \`get loaded_song "title"\`'` |
| `stopwatch` | Stopwatch               | `stopwatch`                                                   |
| `countdown` | Count down to date/time | `countdown '2025/01/01 00:00'`                                |

## Common Patterns

### Conditional Execution

```
condition ? action_if_true : action_if_false
```

### Backtick Queries

Use backticks to execute queries within actions:

```
set 'varname' `play`
param_equal `get_browsed_song 'type'` "audio"
```

### Time Units

- `ms` - milliseconds
- `bt` - beats
- `%` - percentage

### Deck Specification

Prefix action with deck number:

```
deck 1 play
deck 2 volume 50%
deck master get_level
```
