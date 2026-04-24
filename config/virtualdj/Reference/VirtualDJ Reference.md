# VirtualDJ Reference

Merged reference for this repo's VirtualDJ notes, examples, and preferred implementation patterns.

Last reviewed against live VirtualDJ documentation and forum sources on 2026-04-22.

For verb-by-verb API details, use [VDJScript Verbs](VDJScript%20Verbs.md).

## Scope

This document is the source-backed overview and policy layer above the older split reference pages, which are still being folded into a more reliable local reference set. It focuses on:

- Which methods to prefer
- Why those methods are preferable
- Where the working examples live in this repo
- Which notes come from official docs, official staff posts, or repo inference

Source labels used below:

- `Official`: current VirtualDJ manual or VDJPedia
- `Official forum`: post by VirtualDJ staff, Development Manager, or CTO
- `Inference`: conclusion drawn from official docs plus this repo's build setup

## Quick Decisions

- Main deck filter or ColorFX:
  Prefer `filter_selectcolorfx` to choose the ColorFX and `filter` to drive it.
  Why: the current verbs list describes `filter` as the control that applies the selected ColorFX, with nothing applied at `50%`.
  Source: `Official`

- Extra ColorFX-style controls on custom knobs:
  Prefer `effect_colorfx <1-4>` with `effect_colorslider` or `colorfx_slider`.
  Why: the current verbs list exposes four custom ColorFX slots, and CTO guidance explains they exist for extra dedicated controls rather than replacing the main filter knob.
  Source: `Official`, `Official forum`

- Deterministic deck FX pads and buttons:
  Prefer `effect_select <slot>`, `effect_active <slot>`, and `effect_slider <slot> ...`.
  Why: the official effect verbs are slot-centric, and slot-based mappings avoid ambiguity that comes from global effect-name toggles.
  Source: `Official`

- Page-aware sampler pads:
  Prefer `sampler_pad`, `sampler_color`, and `sampler_pad_page`.
  Why: `sampler_color` explicitly follows the visible sampler page, and the pads manual documents page cycling when a bank has more than eight samples.
  Source: `Official`

- Sampler drag-and-drop assignment inside pads:
  Prefer pad-page `drop="sampler_assign <absolute-slot>"` when you want a dragged file to populate a sampler slot from the Pads area.
  Why: `sampler_assign` is the official assignment verb, and the sampler manual documents dragging files onto unlocked sampler pads. The current skin button docs only document click-oriented handlers, while skin drag targets are handled with `<dropzone>` rather than a generic button drag state.
  Source: `Official`, `Inference`

- Absolute sampler slots:
  Prefer `sampler_play`, `sampler_stop`, `get_sample_name`, and `get_sample_color` when you do not want the action to follow the currently visible sampler page.
  Why: these verbs target fixed slots rather than the visible pad page.
  Source: `Official`

- Dynamic text color in skins:
  Prefer one `<text color="`...`">` over state-specific text color attributes when the color itself is dynamic.
  Why: the color docs distinguish between action-returning color values and literal colors, and Development Manager guidance recommends a single dynamic text color action.
  Source: `Official`, `Official forum`

- Dynamic button borders:
  Do not rely on dynamic `border=` colors.
  Why: CTO guidance says dynamic colors are not supported for button borders.
  Source: `Official forum`

- Time mode toggles:
  Prefer `display_time 'remain,elapsed'` with `get_time` instead of custom variables.
  Why: there is a dedicated verb for the job, and current forum guidance explicitly recommends using it instead of toggling your own skin vars for remain versus elapsed displays.
  Source: `Official`, `Official forum`

- Panel visibility and persistent panel switching:
  Prefer `<panel visibility="...">` for pure query-driven UI and `name=""`, `group=""`, `visible=""`, plus `skin_panelgroup` when you want manual switching that persists across sessions.
  Why: this is exactly how the panel SDK page distinguishes the two patterns.
  Source: `Official`

- Modular skins:
  Prefer build-time includes that flatten to one installed `skin.xml`.
  Why: the official SDK still describes skins as a flat package, while extra XML includes remain a forum wish rather than an official runtime feature. This repo already uses `xmllint --xinclude` to flatten modules before install.
  Source: `Official`, `Inference`

## Real Examples In This Repo

Recommended runnable examples:

- [Reference - Slot FX](../Pads/Reference%20-%20Slot%20FX.xml)
  Canonical slot-based audio FX pads.

- [Reference - ColorFX](../Pads/Reference%20-%20ColorFX.xml)
  Canonical filter and ColorFX selection patterns using the current verbs.

- [Reference - Page Aware Sampler](../Pads/Reference%20-%20Page%20Aware%20Sampler.xml)
  Page-aware sampler labels, colors, and actions.

- [Modular skin skeleton](../Skins/ModularSkeleton/README.md)
  Minimal build-time modular skin scaffold for this repo.

Existing repo examples worth studying:

- [GraveRaver root skin](../Skins/GraveRaver/src/skin.xml)
- [GraveRaver build file](../Skins/GraveRaver/justfile)
- [Current project FX page](../Pads/2.%20EFFECTS.xml)

## Skin SDK

### Root Skin Structure

The official SDK still describes a skin package as a `.zip` containing:

- `image_name.png`
- `skincode_name.xml`
- `preview_image.png` optionally
- optional window image files

That makes two things worth keeping straight:

- VirtualDJ expects a flat installed skin package.
- Modularity is a build concern, not a runtime skin feature.

Minimal root pattern:

```xml
<skin
  name="My Skin"
  version="8"
  width="1600"
  height="900"
  image="skin.png"
  preview="preview.png"
  author="Your Name"
  breakline="900"
  breakline2="900">
  ...
</skin>
```

Source: `Official`

### Containers

The core containers worth using first are:

- `<deck>` for deck-scoped UI
- `<panel>` for grouped visibility and panel persistence
- `<group>` for shared positioning and organization
- `<stack>` when only the last visible items should occupy shared slots
- `<define>` for reusable classes and placeholders

Preferred pattern:

- Nest elements inside containers instead of repeating `deck=""` and `panel=""` everywhere.
- Use `<define>` early for repeated button, text, and frame shapes.

Why:

- The SDK explicitly supports nested containers.
- The common element properties page says nesting is preferred over repeating `panel=""`.

Source: `Official`

### Panels

There are two useful panel modes:

- Query-driven:

```xml
<panel visibility="loop">
  ...
</panel>
```

- Persistent manual switching:

```xml
<panel group="rack" name="fx" visible="yes">
  ...
</panel>
```

Driven with:

```vdjscript
skin_panelgroup 'rack' 'fx'
skin_panelgroup 'rack' +1
skin_panel 'my_panel' on
```

Use query-driven panels when state should follow live deck conditions.
Use named groups when the user is choosing a mode and you want it remembered.

Source: `Official`

### Buttons, State, and Query

The current button SDK page is explicit: `query=""` enables the `<on>` graphics when true.

That means:

- Use `query=""` to drive the button's on-state.
- Do not assume `query=""` selects `<selected>` graphics.
- If the element is informational only, `action="nothing"` is a valid way to make it non-destructive.

Example:

```xml
<button action="nothing" query="play">
  <off color="#1C1F24" border="white" border_size="1" radius="10"/>
  <on color="#1C1F24" border="orange" border_size="1" radius="10"/>
  <text text="PLAYING" color="`masterdeck ? color 'orange' : color 'white'`"/>
</button>
```

Source: `Official`, `Official forum`

### Colors and Visuals

The safest dynamic-color rules are:

- `source=` on `<visual type="color">` expects an action that returns a color.
- `color=` expects a color value, so a script action must be wrapped in backticks.

Examples:

```xml
<visual type="color" source="pad_color 1">
  <pos x="0" y="0"/>
  <size width="24" height="2"/>
</visual>
```

```xml
<text color="`get_key_color`" action="get_key"/>
```

Preferred methods:

- Use `<visual type="color">` for colored underlines, fills, and status bars.
- Use a single dynamic `<text color="`...`">` when only the text color changes.

Avoid:

- Dynamic `border=` colors on button vector states. CTO guidance says this is not supported.

Source: `Official`, `Official forum`

### Positioning

The positioning SDK page allows both nested `<pos>` and inline `x="" y="" width="" height=""` forms.
For larger skins, use `<pos>` and `<size>` consistently because:

- it is easier to scan
- it matches this repo's style
- it makes class placeholders easier to reason about

Source: `Official`

## VDJScript Patterns

### Core Syntax Worth Reaching For

- `action1 & action2`
  Sequential actions

- `condition ? when_true : when_false`
  Branching

- Backticks around action-returning values
  Use when a value consumer needs the result of another action

- `param_*`
  Use for live parameter comparisons and transforms

- `var_*`
  Use when you truly need stored state

Examples:

```vdjscript
param_equal `get_browsed_song 'type'` 'audio' ? load : nothing
```

```vdjscript
down ? filter 75% : filter 50%
```

```vdjscript
repeat_start 'fxpulse' 1bt & effect_active 1
```

Source: `Official`

### Prefer Built-ins Over Skin Vars

If VirtualDJ already has a dedicated action for a behavior, prefer that over inventing a variable.

Good examples:

- `display_time 'remain,elapsed'` instead of a custom elapsed/remain toggle var
- `skin_panelgroup` instead of a custom var that emulates grouped panels
- `setting 'optionName' value` when you are intentionally changing a setting

Reason:

- less state drift
- fewer hidden dependencies
- behavior lines up better with controllers and the default UI

Source: `Official`, `Official forum`

### Write Queries With an Explicit Else

Prefer:

```vdjscript
effect_active 1 ? blink 500ms : off
```

Over:

```vdjscript
effect_active 1 ? blink 500ms
```

Why:

- explicit `off` avoids empty or ambiguous UI states
- it is easier to debug later

Source: `Inference`

## Effects

### Deck FX Slots

The official verbs and the current deck FX UI are slot-based.

Preferred slot workflow:

1. Select the effect into a slot
2. Activate the slot
3. Move the slot's sliders or buttons

Example:

```vdjscript
effect_select 1 'Echo' &
effect_slider 1 1 75% &
effect_slider 1 2 1bt &
effect_active 1 on
```

Why this is the safest reference pattern:

- it mirrors the actual deck FX rack model
- it behaves predictably across skins and controllers
- it avoids name-based ambiguity when several effects are loaded

Use [Reference - Slot FX](../Pads/Reference%20-%20Slot%20FX.xml) for a working repo example.

Source: `Official`

### Filter and ColorFX

Current official behavior:

- `filter` applies the selected ColorFX to the sound
- nothing is applied at `50%`
- more effect is applied the farther the control moves from center
- `filter_selectcolorfx` selects which ColorFX the filter knob controls
- `filter_label` returns the label under the filter knob
- `filter_resonance` changes filter resonance

Preferred method for the main deck filter:

```vdjscript
filter_selectcolorfx 'Echo' &
filter 75%
```

Preferred method for a dedicated select-only button:

```vdjscript
filter_selectcolorfx 'Flanger'
```

Preferred method for an extra custom ColorFX control:

```vdjscript
effect_colorfx 1 'Echo'
effect_colorslider 1
```

Notes:

- `effect_colorslider` is the center-off ColorFX-style slider action.
- `effect_colorfx` exposes up to four extra custom ColorFX slots.
- CTO guidance says that the dedicated `colorfx` slot only exposes approved ColorFX-compatible effects, while extra slots are more flexible.

Use [Reference - ColorFX](../Pads/Reference%20-%20ColorFX.xml) for a working repo example.

Source: `Official`, `Official forum`

### Which ColorFX Method To Use

- If you are emulating the standard deck filter knob:
  use `filter_selectcolorfx` + `filter`

- If you are building extra ColorFX-like controls that should not steal the deck's main filter:
  use `effect_colorfx <1-4>` + `effect_colorslider`

- If you are building a deterministic pad page for normal audio effects:
  use regular slot FX instead of ColorFX

Source: `Official`, `Official forum`

### Stems FX

The official verbs list includes `effect_stems`, `effect_arm_stem`, and `effect_stems_color`.

Use `effect_stems` when you intentionally want FX targeting to follow selected stems.

Caution:

- Older official forum posts from 2021 reported inconsistencies between regular slot FX and special slots such as `colorfx`.
- Treat any ColorFX-plus-stems behavior as build-sensitive and test it on the exact VirtualDJ build you use.

That caution is intentionally dated because the forum guidance is older than the current manual.

Source: `Official`, `Official forum`

### Native Effects

The current native effects appendix is the authoritative list for built-in effects, video effects, and transitions.

High-frequency audio effects to design around first:

- Echo
- Echo Out
- Reverb
- Beat Grid
- Flanger
- Filter
- Noise
- Phaser
- Loop Roll
- VinylBrake
- Stutter Out

For the current full list, use the official appendix instead of hard-coding old plugin menus into your docs.

Source: `Official`

## Sampler and Pads

### Default Page Behavior

The current pads manual says:

- the Sampler page shows the first eight pads of the active bank
- Parameter 2 cycles samples in the bank when there are more than eight
- right-click or shift stops a triggered sample

Source: `Official`

### Page-Aware vs Absolute Sampler Methods

Use page-aware methods when the UI should follow the visible `1-8`, `9-16`, `17-24`, and later pages:

- `sampler_pad`
- `sampler_color`
- `sampler_pad_page`
- `sampler_pad_volume`

Use absolute-slot methods when the UI should always target the same underlying sample slots:

- `sampler_play`
- `sampler_stop`
- `get_sample_name`
- `get_sample_color`
- `sampler_volume`

Practical rule:

- visible pad UI: page-aware
- fixed utility controls: absolute

Use [Reference - Page Aware Sampler](../Pads/Reference%20-%20Page%20Aware%20Sampler.xml) for a working repo example.

Source: `Official`

### `sampler_loaded` and `auto`

The current verbs page documents `sampler_loaded <n>` as a fixed slot query.
In practice, repo examples and modern custom pad pages often use `sampler_loaded <n> 'auto'` beside `sampler_pad <n> 'auto'`.

Treat that pattern as:

- useful and widely used in practice
- worth testing on the current build if you depend on page-awareness for empty-slot detection

This is labeled separately because the current manual is explicit about page-awareness for `sampler_color`, but not equally explicit for every `sampler_* ... 'auto'` helper.

Source: `Official`, `Inference`

### Sampler Options That Matter

Current official options worth knowing:

- `samplerSpanAcrossDecks`
  When set to `yes`, a 16-sample bank makes deck 2 automatically show `9-16`

- `samplerIndependentDeckBanks`
  Each deck and master can have their own sample bank

- `displayTime`
  Selects elapsed, remain, or total display mode

Source: `Official`

### 2025 Sampler Note

A forum thread published on 2025-09-23 reported inconsistent sampler sync behavior on controller pads in VirtualDJ 2025 builds, especially when the triggering deck was not the master deck. The same thread shows:

- a workaround suggested by CTO Adion on 2025-09-26: try `deck master sampler_pad <n>`
- the original poster later reporting on 2025-10-04 that support resolved the issue in an Early Access update

Practical takeaway:

- treat master-deck sampler workarounds as build-specific
- do not document them as timeless behavior

Source: `Official forum`

## Browser Filter Syntax

Useful filter building blocks:

- comparison operators
- logical operators
- date and time filters
- tag filters
- mixing and library filters

Typical patterns:

```text
genre contains house
```

```text
bpm > 120 and bpm < 130
```

```text
year >= 2020
```

```text
type = video
```

The official appendix remains the best exhaustive source here, so keep repo docs focused on patterns you actually use instead of copying the whole appendix into local markdown.

Source: `Official`

## Options Worth Knowing

High-value official options for skin and pad authors:

- `filterDefaultResonance`
  Sets the amount of resonance applied by the filter

- `fxProcessing`
  Chooses whether effects are processed pre-fader or post-fader

- `resetFXOnLoad`
  Stops all effects when a new song loads

- `globalQuantize`
  Sets beat, measure, or quarter quantization

- `smartLoop`
  Auto-adjusts loop points for seamless loops

- `quantizeSetCue`
  Auto-aligns newly set cues according to quantization

Script note:

```vdjscript
setting 'filterDefaultResonance' 75%
setting 'fxProcessing' 'post-fader'
```

Source: `Official`

## Modular Skin Workflow

### What VirtualDJ Officially Describes

The SDK still documents a flat skin package:

- `skin.xml`
- image file
- optional preview file

It does not document runtime support for loading arbitrary extra XML modules from the main skin file.

Source: `Official`

### What This Repo Should Prefer

Use build-time modularity:

- keep source XML split into `defs/` and `panels/`
- compose with XInclude or another XML preprocessor locally
- build one flattened `skin.xml` before install

This is the pattern already used by [GraveRaver](../Skins/GraveRaver/justfile).

Why:

- easier maintenance
- reusable classes and panel slices
- installed output still matches the official flat package model

Source: `Inference`

### Skeleton In This Repo

Use [ModularSkeleton](../Skins/ModularSkeleton/README.md) as the starting point.

It demonstrates:

- build-time XInclude flattening with `xmllint --xinclude`
- shared colors and classes in `src/defs/`
- simple panel slices in `src/panels/`
- a flat built output for installation

## Sources

Official docs:

- [VirtualDJ Skin SDK](https://www.virtualdj.com/wiki/Skin_SDK.html)
- [Skin Button](https://virtualdj.com/wiki/Skin-Button.html)
- [Skin SDK Dropzone](https://www.virtualdj.com/wiki/Skin%20SDK%20Dropzone.html)
- [Skin Panel](https://www.virtualdj.com/wiki/Skin%20SDK%20Panel.html)
- [Skin Default Colors](https://virtualdj.com/wiki/Skin%20Default%20Colors.html)
- [Skin SDK Visual](https://virtualdj.com/wiki/skinsdkvisual.html)
- [List of VDJScript verbs](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html)
- [List of Options](https://www.virtualdj.com/manuals/virtualdj/appendix/optionslist/)
- [List of Native Effects](https://www.virtualdj.com/manuals/virtualdj/appendix/nativeeffects/)
- [Pads manual](https://www.virtualdj.com/manuals/virtualdj/interface/decks/decksadvanced/pads.html)
- [Sampler manual](https://www.virtualdj.com/manuals/virtualdj/interface/browser/sideview/sampler.html)

Official forum guidance cited for method choices:

- [Border Color using placeholder](https://virtualdj.com/forums/242871/VirtualDJ_Skins/Border_Color_using_placeholder.html)
- [effect_colorfx & effect_stems_color ?](https://www.virtualdj.com/forums/241078/VirtualDJ_Technical_Support/effect_colorfx___effect_stems_color__.html)
- [Default filter and color fx filter](https://virtualdj.com/forums/252675/VirtualDJ_Technical_Support/Default_filter_and_color_fx_filter.html)
- [Skin text action; visibility or visual?](https://www.virtualdj.com/forums/267953/VirtualDJ_Skins/Skin_text_action%3B_visibility_or_visual%3F.html)
- [Virtual Dj 2025 Sampler Sync](https://www.virtualdj.com/forums/265522/VirtualDJ_Technical_Support/Virtual_Dj_2025_Sampler_Sync.html)
- [No longer possible to access 16 samples from controllers with 8 x 2 pads?](https://www.virtualdj.com/forums/261416/VirtualDJ_Technical_Support/No_longer_possible_to_access_16_samples_from_controllers_with_8_x_2_pads_.html)
- [Aditional xml for Skins](https://virtualdj.com/forums/248589/Wishes_and_new_features/Aditional_xml_for_Skins.html)

Repo examples:

- [GraveRaver source skin](../Skins/GraveRaver/src/skin.xml)
- [GraveRaver build file](../Skins/GraveRaver/justfile)
- [Reference - Slot FX](../Pads/Reference%20-%20Slot%20FX.xml)
- [Reference - ColorFX](../Pads/Reference%20-%20ColorFX.xml)
- [Reference - Page Aware Sampler](../Pads/Reference%20-%20Page%20Aware%20Sampler.xml)
- [ModularSkeleton](../Skins/ModularSkeleton/README.md)
