# VirtualDJ Effects Engines Guide

Comprehensive guide to VirtualDJ's multiple effect systems, how they work, and practical usage patterns.

---

## Overview

VirtualDJ provides multiple independent effect engines that serve different purposes:

| Effect Engine | Purpose                             | Location                  | Number of Slots           |
| ------------- | ----------------------------------- | ------------------------- | ------------------------- |
| **ColorFX**   | Quick filter/effect control         | Filter knob per deck      | 1 per deck (special slot) |
| **Deck FX**   | Standard deck effects               | FX panel per deck         | 3-6 slots per deck        |
| **Master FX** | Global effects on master output     | Master panel              | 3-6 slots                 |
| **Video FX**  | Video effects and transitions       | Video panel               | Multiple slots            |
| **Stems FX**  | Effects applied to individual stems | Via Stems pads or FX menu | Uses deck FX slots        |
| **Pad FX**    | Quick-trigger effects with presets  | Pads pages                | Temporary effect triggers |

---

## ColorFX (Filter Slot)

### What is ColorFX?

ColorFX is VirtualDJ's **special effect slot** integrated with the filter knob on each deck. It provides one-knob control for quick effects that work well with a single parameter.

### Key Characteristics

- **One knob control**: Designed for filter knobs on mixers
- **Pre-fader only**: ColorFX is integrated with the EQ engine and always processes pre-fader
- **Curated effect list**: Only effects designated as "ColorFX-compatible" appear in the selection menu
- **Center position**: Knob at 50% (12 o'clock) = no effect, move left/right to apply

### ColorFX vs Regular Filter

Starting with VirtualDJ 8.4+ builds, the traditional filter and ColorFX system were unified:

- **Filter** is now a ColorFX effect (the default)
- Use `filter_selectcolorfx` to choose different ColorFX
- The knob action `filter` automatically works with whatever ColorFX is selected
- Filter resonance can be adjusted via `filterDefaultResonance` option or `filter_resonance` action

### Available ColorFX Effects

Common ColorFX-compatible effects include:

- Filter (High-pass/Low-pass with resonance)
- Echo
- Flanger
- Phaser
- Noise
- Pan
- Wahwah
- And others marked as ColorFX-compatible

### VDJScript Commands

**Select a ColorFX:**

```
filter_selectcolorfx 'echo'
filter_selectcolorfx 'flanger'
```

**Control the ColorFX knob:**

```
filter                          # Main filter/colorFX control
filter 50%                      # Reset to center (no effect)
filter 75%                      # Set to specific position
filter_resonance 50%            # Adjust filter resonance
```

**Activate/deactivate ColorFX:**

```
effect_active 'colorfx'         # Toggle on/off
effect_active 'colorfx' on      # Turn on
effect_active 'colorfx' off     # Turn off
```

**Show ColorFX GUI:**

```
effect_show_gui 'colorfx'       # Open effect parameters window
```

**Get ColorFX label:**

```
filter_label                    # Returns name of selected ColorFX
```

### Practical Usage

**Basic filter control:**

```xml
<slider action="filter" frommiddle="true">
  <pos x="100" y="100" />
  <size width="60" height="60" />
</slider>
```

**Button to select and activate:**

```xml
<button
  action="effect_active 'colorfx' & filter 50% & effect_show_gui 'colorfx'"
  rightclick="filter_selectcolorfx"
>
  <text action="filter_label" />
</button>
```

### Important Notes

- ColorFX is **always pre-fader** (cannot be changed to post-fader)
- Only one ColorFX can be active per deck at a time
- Some effects work better as ColorFX than others (check by testing parameter response)
- Filter effect has special integration and may behave differently than other ColorFX

---

## Deck FX Slots (FX1, FX2, FX3, etc.)

### What are Deck FX Slots?

Standard effect slots on each deck that provide **full multi-parameter control** with up to 6 parameters and 3 buttons per effect.

### Slot Configuration

VirtualDJ skins can display effects in different layouts:

| Layout                | Slots   | Parameters per Effect          |
| --------------------- | ------- | ------------------------------ |
| **FX x1** (Single FX) | 1 slot  | Up to 6 parameters + 3 buttons |
| **FX x3** (Multi FX)  | 3 slots | Up to 2 parameters per slot    |
| **FX x6** (Advanced)  | 6 slots | 1-2 parameters per slot        |

### VDJScript Commands

**Effect Selection:**

```
effect_select 1 'echo'          # Load Echo into slot 1
effect_select 2 'flanger'       # Load Flanger into slot 2
effect_select 3 'reverb'        # Load Reverb into slot 3
```

**Effect Activation:**

```
effect_active 1                 # Toggle slot 1 on/off
effect_active 2 on              # Turn slot 2 on
effect_active 3 off             # Turn slot 3 off
```

**Effect Parameters (Sliders):**

```
effect_slider 1 1 50%           # Slot 1, parameter 1 = 50%
effect_slider 1 2 1bt           # Slot 1, parameter 2 = 1 beat
effect_slider 2 1               # Control slot 2, param 1 (pass-through from knob)
```

**Effect Buttons:**

```
effect_button 1 1               # Press button 1 on slot 1 effect
effect_button 1 2               # Press button 2 on slot 1 effect
effect_button 1 3               # Press button 3 on slot 1 effect
```

**Effect GUI:**

```
effect_show_gui 1               # Show full GUI for slot 1 effect
```

**Selecting next/previous effect:**

```
effect_select 1 +1              # Next effect in list for slot 1
effect_select 1 -1              # Previous effect in list for slot 1
```

### Checking if Effects are Active

**Query if effect slot is active:**

```
effect_active 1 ? action_if_true : action_if_false
```

**Example button with visual feedback:**

```xml
<button action="effect_active 1">
  <pos x="100" y="100" />
  <size width="80" height="40" />
  <off color="#404040" />
  <on color="#00FF00" />
  <text text="FX1" />
</button>
```

**LED indicator for effect active:**

```xml
<led brightness="`effect_active 1`">
  <pos x="50" y="50" />
  <size width="20" height="20" />
  <off x="0" y="100" />
  <on x="20" y="100" />
</led>
```

### Effect by Name (Bypass Slots)

You can reference effects **by name** instead of slot number:

```
effect_active 'echo'            # Activate Echo (wherever it is)
effect_slider 'echo' 1 75%      # Control Echo param 1 directly
effect_active 'reverb' on       # Turn on Reverb effect
```

This allows **multiple instances** of effects and **effect chaining**.

### Effect Chaining

Effects process in the order they were activated:

```
effect_active 'filter' on       # First in chain
effect_active 'echo' on         # Processes after filter
effect_active 'reverb' on       # Processes last
```

Result: Signal → Filter → Echo → Reverb → Output

### Practical Usage Patterns

**Simple toggle button:**

```
effect_active 1
```

**Hold-to-use effect:**

```
effect_active 1 on while_pressed
```

**Effect with auto-reset:**

```
effect_active 1 on & wait 4bt & effect_active 1 off
```

**Select and activate in one action:**

```
effect_select 1 'echo' & effect_active 1 on
```

**Temporary effect (release to deactivate):**

```
down ? effect_active 1 on : effect_active 1 off
```

---

## Master FX

### What is Master FX?

Effects applied to the **master output** after all decks are mixed together. Affects the entire audio output.

### Targeting Master Deck

Use `deck master` to target master FX slots:

```
deck master effect_select 1 'reverb'
deck master effect_active 1
deck master effect_slider 1 1 50%
```

### Master FX Use Cases

- **Room/venue simulation**: Reverb on master for ambience
- **Overall compression/limiting**: Master processing
- **Broadcast effects**: Effects for live streams
- **Emergency effects**: Quick transitions or drops

### Practical Example

**Master effect that stays active:**

```
# In controller ONINIT or custom button:
deck master effect_select 2 'MyVSTEffect' & 
deck master effect_active 2 on
```

**Temporary master effect:**

```xml
<button action="deck master effect_active 1 on while_pressed">
  <text text="MASTER VERB" />
</button>
```

---

## Video FX

### What are Video FX?

Effects that modify, transform, or overlay video output.

### Video FX Categories

1. **Video Effects** - Applied to video sources (Blur, Colorize, Spectral, etc.)
2. **Video Transitions** - Crossfade effects (Cube, Doors, Fade, etc.)
3. **Video Transforms** - Modifications (Shake, Strobe, Negative, etc.)
4. **Video Overlays** - Text, titles, karaoke, etc.
5. **Video Sources** - Camera, slideshow, shader inputs

### VDJScript Commands

**Select video effect:**

```
deck master video_fx_select 'spectral'
video_fx_select 'blur'          # On current deck
```

**Control video effect parameters:**

```
deck master video_fx_slider 1 50%
deck master video_fx_slider 2 75%
```

**Video transitions:**

```
video_transition 1000ms         # Crossfade with 1 second transition
video_transition 'cube' 2000ms  # Use Cube transition
```

**Check video availability:**

```
is_video ? action_if_video : action_if_audio_only
```

### Practical Usage

**Map video FX to filter knobs:**

```
# Left filter = parameter 1, Right filter = parameter 2
device_side left ? deck master video_fx_slider 1 : deck master video_fx_slider 2
```

---

## Stems FX

### What is Stems FX?

Effects applied to **individual stems** (Vocal, Melody, Bass, Drums, etc.) rather than the entire track.

### How Stems FX Works

1. **Enable Stems FX mode** - Tells VirtualDJ which stem(s) to apply effects to
2. **Activate effects** - Normal deck effects will now only affect selected stems
3. **Result** - Effect processes only the selected stem(s), rest plays clean

### Stem Selection Options

- **Vocal** - Lead vocals only
- **Melody** - Instruments and harmonies (Instru + Bass)
- **Rhythm** - Percussion (HiHat + Kick)
- **MeloRhythm** - Everything except vocals
- **Acapella** - Vocals only (alternative term)
- **Instrumental** - Everything except vocals (alternative term)
- **Individual stems**: Vocal, HiHat, Bass, Instru, Kick

### VDJScript Commands

**Enable Stems FX mode:**

```
effect_stems 'vocal'            # Apply effects to vocals only
effect_stems 'melody'           # Apply effects to melody only
effect_stems 'rhythm'           # Apply effects to drums only
effect_stems 'melorhythm'       # Apply effects to everything except vocals
effect_stems off                # Disable stems FX (back to full track)
```

**Check if Stems FX is active:**

```
effect_stems ? action_if_active : action_if_not_active
```

**Combine with regular effects:**

```
effect_stems 'vocal' & effect_active 1 'echo'
```

### Stems FX from GUI

In the FX dropdown menu, toggle Stems FX mode:

- **Vocal** button - Apply effects to vocals
- **Melody** button - Apply effects to melody
- **Rhythm** button - Apply effects to rhythm

When enabled, a small **"S" icon** appears next to effects indicating Stems FX mode.

### Stems FX Pad Page

The **Stems FX** pad page provides quick access:

- **StemsFX Toggle** - Cycles through Vocal/Melody/Rhythm/Off
- **Standard FX Pads** - Trigger effects with StemsFX mode active

### Practical Usage

**Vocal-only echo out:**

```
effect_stems 'vocal' & 
effect_active 'echo out' & 
wait 4bt & 
mute_stem 'vocal'
```

**Instrument beatgrid effect:**

```
effect_stems 'melorhythm' & 
effect_active 'beatgrid'
```

**Toggle stems FX on/off:**

```xml
<button
  action="toggle '$stemfx' & 
                var '$stemfx' ? effect_stems 'vocal' : effect_stems off"
>
  <text text="STEMS FX" />
</button>
```

### Limitations and Notes

- **Pre-fader recommended**: Stems FX works best with `fxProcessing` set to `pre-fader`
- **Some hardware incompatible**: Controllers with hardware FX sends may not support Stems FX properly
- **ColorFX limitation**: ColorFX does not support Stems FX (applies to full track)
- **Reset options**: Use `resetStemsOnLoad` and `resetFXOnLoad` options to auto-reset

---

## Pad FX

### What is Pad FX?

**Quick-trigger effects** with pre-configured parameters, designed for single-button effect execution.

### Pad FX Characteristics

- **Preset parameters**: Effects are called with specific parameter values
- **Temporary by design**: Intended to trigger and release, not save settings
- **Independent from slots**: Can run alongside regular FX slots
- **Perfect for performance**: Quick creative effects without knob adjustment

### VDJScript Commands

**Basic Pad FX:**

```
padfx 'echo' 50% 1bt            # Echo at 50% strength, 1 beat length
padfx 'beatgrid'                # Trigger beatgrid effect
padfx 'flanger' 75% 2bt         # Flanger at 75%, 2 beat speed
```

**Single Pad FX (one-shot):**

```
padfx_single 'echo out' 80% 1bt
```

**Pad FX with Stems:**

```
padfx 'echo' 50% 1bt 'stemfx:vocal'
padfx 'reverb' 75% 'stemfx:melorhythm'
```

**Full Pad FX syntax:**

```
padfx 'effectname' param1 param2 param3 param4 'stemfx:stemname'
```

### Pad FX Parameter Order

Parameters depend on the effect. Common patterns:

**Echo:**

```
padfx 'echo' strength% length_beats feedback% wetdry%
padfx 'echo' 50% 1bt 65% 75%
```

**Echo Out:**

```
padfx 'echo out' strength% length_beats
padfx 'echo out' 80% 1bt
```

**Beatgrid:**

```
padfx 'beatgrid'                # No parameters needed
```

**Vinyl Brake:**

```
padfx 'vinylbrake' multiplier length_beats echo% repeat%
padfx 'vinylbrake' 1bt 50% 50% 0%
```

### Checking Pad FX Status

Pad FX don't have a built-in "active" query. Track status with variables:

```
# Set variable when activating
set 'padfx_active' 1 & padfx 'echo' 50% 1bt

# Check variable
var 'padfx_active' ? visual_feedback_on : visual_feedback_off
```

### Show Pad FX GUI

```
effect_show_gui 'stemname' 'effectname'
effect_show_gui 'vocal' 'echo'
effect_show_gui 'rhythm' 'beatgrid'
```

### Practical Usage

**Simple pad effect:**

```xml
<button action="padfx 'echo out' 75% 1bt">
  <text text="ECHO OUT" />
</button>
```

**Stems-specific pad effect:**

```xml
<button
  action="effect_stems 'vocal' & 
                padfx_single 'echo out' 80% 1bt & 
                wait 4bt & 
                mute_stem 'vocal' & 
                effect_stems off"
>
  <text text="VOCAL OUT" />
</button>
```

**Advanced stems + effect combo:**

```xml
<!-- Mute melody, echo it out, then mute it -->
<button
  action="var 'stemsnfx' 1 ? 
                  toggle 'stemsnfx' & mute_stem 'melody' off : 
                  toggle 'stemsnfx' & 
                  effect_stems 'melody' & 
                  padfx_single 'echo out' 75% 1bt & 
                  wait 1bt & 
                  mute_stem 'melody' & 
                  effect_stems off"
>
</button>
```

---

## Pre-Fader vs Post-Fader

### What's the Difference?

- **Pre-fader**: Effects process **before** volume faders and crossfader
- **Post-fader**: Effects process **after** volume faders and crossfader

### Signal Flow

**Pre-fader:**

```
Deck → Effects → EQ → Volume Fader → Crossfader → Master
```

**Post-fader:**

```
Deck → EQ → Volume Fader → Crossfader → Effects → Master
```

### When to Use Each

**Use Pre-fader when:**

- Effects should continue playing even when fader is down (echo tails)
- Using effects with stems (Stems FX requires pre-fader)
- Most creative DJ effects work

**Use Post-fader when:**

- Effects should stop immediately when fader is down
- Using effects during transitions
- Hardware requires it (some mixers)

### Setting FX Processing Mode

**Global setting:**

```
setting 'fxProcessing' 'pre-fader'
setting 'fxProcessing' 'post-fader'
```

**Temporary change:**

```
setting 'fxProcessing' 'post-fader' & 
padfx 'echo out' 1bt 50% & 
wait 5000ms & 
setting 'fxProcessing' 'pre-fader'
```

**Option in Settings:**

- Settings → Audio → `fxProcessing` → Pre-fader / Post-fader

### Important Notes

- **ColorFX is always pre-fader** (cannot be changed)
- **Hardware effects**: Some controllers have hardware post-fader sends
- **Stems FX**: Works best in pre-fader mode
- **Default**: Pre-fader is the default and most common setting

---

## Effect Queries and Conditionals

### Checking Effect Status

**Is any effect active on slot:**

```
effect_active 1 ? blink : off
```

**Is specific effect active:**

```
effect_active 'echo' ? action : action
```

**Are stems FX active:**

```
effect_stems ? show_stems_indicator : hide_indicator
```

**Multiple effect check:**

```
effect_active 1 | effect_active 2 | effect_active 3 ? led_on : led_off
```

### Visual Feedback Examples

**Button color based on effect state:**

```xml
<button action="effect_active 1">
  <size width="80" height="40" />
  <up color="#404040" radius="4" />
  <selected color="#00FF00" radius="4" />
  <text text="FX 1" />
</button>
```

**LED indicator:**

```xml
<visual source="`effect_active 1`" type="onoff">
  <pos x="100" y="50" />
  <size width="10" height="10" />
  <off color="black" />
  <on color="red" />
</visual>
```

**Text showing effect name:**

```xml
<textzone>
  <pos x="100" y="100" />
  <size width="200" height="30" />
  <text text="`get_effect_name 1`" />
</textzone>
```

**Blink when effect active:**

```
effect_active 1 & blink 500ms
```

---

## Practical Effect Workflows

### Workflow 1: Echo Out Transition

Gradually remove track with echo tail:

```
effect_stems off & 
effect_select 1 'echo out' & 
effect_slider 1 1 80% & 
effect_slider 1 2 1bt & 
effect_active 1 on & 
wait 4bt & 
volume 0% 2000ms
```

### Workflow 2: Beatgrid Breakdown

Create a breakdown using beatgrid:

```
effect_select 1 'beatgrid' & 
effect_active 1 on & 
wait 16bt & 
effect_active 1 off
```

### Workflow 3: Vocal Echo with Stems

Echo only the vocals:

```
effect_stems 'vocal' & 
padfx 'echo' 50% 1bt & 
wait 8bt & 
effect_stems off
```

### Workflow 4: Build Up with Multiple Effects

Chain effects for a build-up:

```
# Start with light flanger
effect_active 'flanger' on & 
effect_slider 'flanger' 1 25% & 

# Add echo
wait 8bt & 
effect_active 'echo' on & 
effect_slider 'echo' 1 50% & 

# Add reverb
wait 8bt & 
effect_active 'reverb' on & 

# Drop - remove all
wait 8bt & 
effect_active 'flanger' off & 
effect_active 'echo' off & 
effect_active 'reverb' off
```

### Workflow 5: Acapella Out with Reverb

Remove everything but vocals with reverb tail:

```
effect_stems 'vocal' & 
padfx 'reverb' 75% 4bt & 
wait 2bt & 
mute_stem 'vocal' & 
effect_stems off
```

---

## Effect Lists and Organization

### Creating Effect Lists

VirtualDJ allows organizing effects into **custom lists** for quick access.

### Managing Effect Lists

1. Open Effects dropdown
2. Scroll to bottom
3. Click **"More..."** or **"Manage..."**
4. **Audio Effects List Editor** opens

### List Editor Functions

- **Search bar**: Find effects quickly
- **Current list**: Switch between lists
- **Add/Remove**: Build custom effect collections
- **New list**: Create specialized collections
- **Delete list**: Remove unwanted lists

### Practical List Examples

**"Quick FX" list:**

- Filter
- Echo
- Reverb
- Beatgrid
- Brake

**"Build Up FX" list:**

- Flanger
- Phaser
- Echo
- Reverb
- Riser

**"Stems FX" list:**

- Echo Out
- Reverb
- Beatgrid
- Vinyl Brake

---

## Advanced Effect Techniques

### Effect Banks

Use variables to save/recall effect configurations:

```
# Save current effects to bank 1
set 'fx_bank1_slot1' `get_effect_name 1` & 
set 'fx_bank1_slot2' `get_effect_name 2` & 
set 'fx_bank1_slot3' `get_effect_name 3`

# Recall bank 1
effect_select 1 `get_var 'fx_bank1_slot1'` & 
effect_select 2 `get_var 'fx_bank1_slot2'` & 
effect_select 3 `get_var 'fx_bank1_slot3'`
```

### Effect Presets with Parameters

Save effect + parameters:

```
# Preset: Heavy Echo
effect_select 1 'echo' & 
effect_slider 1 1 75% & 
effect_slider 1 2 2bt & 
effect_slider 1 3 80% & 
effect_active 1 on

# Preset: Light Reverb
effect_select 2 'reverb' & 
effect_slider 2 1 30% & 
effect_slider 2 2 50% & 
effect_active 2 on
```

### Macro Effects

Combine multiple actions:

```
# "Drop" macro: Kill all effects, reset filter, cut bass momentarily
effect_active 1 off & 
effect_active 2 off & 
effect_active 3 off & 
filter 50% & 
eq_low 0% & 
wait 1bt & 
eq_low 100%
```

### Effect Automation

Use `repeat_start` for automated effect patterns:

```
# Auto-toggle filter every 4 beats
repeat_start 'auto_filter' 4bt 0 & 
  filter `filter ? 75% : 25%`

# Stop automation
repeat_stop 'auto_filter'
```

---

## Troubleshooting Common Issues

### Effects Not Working

**Check:**

1. Effect is selected in slot: `get_effect_name 1`
2. Effect is activated: `effect_active 1`
3. Parameters are not at 0: `effect_slider 1 1 50%`
4. Pre/post-fader setting matches hardware
5. Audio routing is correct

### Effects Sound Wrong

**Check:**

1. Parameter values are appropriate for the effect
2. Multiple effects aren't conflicting
3. Effect is suitable for the material (some effects work better on certain genres)
4. Stems separation quality (if using Stems FX)

### Stems FX Issues

**Check:**

1. Stems are analyzed/prepared
2. `fxProcessing` is set to `pre-fader`
3. Hardware supports software effects (not hardware sends)
4. `effect_stems` is active: `effect_stems ? on : off`
5. Correct stem name is used (Vocal, Melody, Rhythm, etc.)

### ColorFX Not Responding

**Check:**

1. Effect is ColorFX-compatible (not all effects are)
2. Using `filter` action, not `effect_slider 'colorfx'`
3. Filter knob is mapped correctly: `filter` with `frommiddle="true"`
4. Effect is actually selected: `filter_selectcolorfx 'effectname'`

### Effect Parameters Not Changing

**Check:**

1. Correct slot number: `effect_slider 1 1` vs `effect_slider 2 1`
2. Correct parameter number (1-6)
3. Value format matches parameter type (% for dry/wet, bt for beats, ms for time)
4. Effect GUI isn't overriding controller values

---

## Quick Reference

### Effect Commands Cheat Sheet

```
# SELECTION
effect_select 1 'echo'              # Select effect for slot 1
effect_select 'colorfx' 'filter'    # Select ColorFX
filter_selectcolorfx 'echo'         # Select ColorFX (alternative)

# ACTIVATION
effect_active 1                     # Toggle slot 1
effect_active 1 on                  # Turn on slot 1
effect_active 'echo'                # Toggle Echo by name
effect_active 'colorfx'             # Toggle ColorFX

# PARAMETERS
effect_slider 1 1 50%               # Slot 1, param 1 = 50%
effect_slider 1 2 1bt               # Slot 1, param 2 = 1 beat
effect_slider 'echo' 1 75%          # Echo param 1 = 75%
filter 50%                          # ColorFX/filter = 50%

# BUTTONS
effect_button 1 1                   # Press button 1 on slot 1
effect_button 'echo' 2              # Press button 2 on Echo

# GUI
effect_show_gui 1                   # Show GUI for slot 1
effect_show_gui 'colorfx'           # Show ColorFX GUI
effect_show_gui 'vocal' 'echo'      # Show stems FX GUI

# STEMS FX
effect_stems 'vocal'                # Apply to vocals
effect_stems 'melorhythm'           # Apply to instruments
effect_stems off                    # Disable stems FX

# PAD FX
padfx 'echo' 50% 1bt                # Trigger echo
padfx_single 'echo out' 80% 1bt     # One-shot echo out
padfx 'reverb' 75% 'stemfx:vocal'   # Reverb on vocals

# MASTER FX
deck master effect_select 1 'reverb'
deck master effect_active 1
deck master effect_slider 1 1 50%

# VIDEO FX
deck master video_fx_select 'spectral'
deck master video_fx_slider 1 50%

# QUERIES
effect_active 1                     # Is slot 1 active?
effect_stems                        # Is stems FX active?
get_effect_name 1                   # Get name of effect in slot 1
filter_label                        # Get ColorFX name
```

### Common Effect Values

**Strength/Dry-Wet:**

- 0% = No effect
- 50% = Half mix
- 100% = Full effect

**Beat Values:**

- `0.25bt` = 1/4 beat
- `0.5bt` = 1/2 beat
- `1bt` = 1 beat
- `2bt` = 2 beats
- `4bt` = 1 bar

**Time Values:**

- `100ms` = 0.1 seconds
- `500ms` = 0.5 seconds
- `1000ms` = 1 second

---

## Summary

VirtualDJ's effect system is powerful and flexible:

1. **ColorFX** - Quick one-knob filter/effects
2. **Deck FX Slots** - Full multi-parameter effects (3-6 slots)
3. **Master FX** - Global effects on master output
4. **Video FX** - Visual effects and transitions
5. **Stems FX** - Effects on individual stems
6. **Pad FX** - Quick-trigger preset effects

**Key Principles:**

- Effects can be controlled by **slot number** or **by name**
- **Pre-fader** is default and recommended for most use cases
- **Stems FX** requires pre-fader processing
- **ColorFX** is always pre-fader (integrated with EQ engine)
- Effects can be **chained** for creative combinations
- Use **variables** to track complex effect states

**Best Practices:**

- Organize effects into custom lists
- Learn your most-used effects thoroughly
- Use Stems FX for creative vocal/instrument manipulation
- Combine Pad FX with variables for complex workflows
- Check `resetFXOnLoad` option to avoid effect carryover
- Test effects with different audio material

This guide covers the core effect engines in VirtualDJ. Experiment with combinations to develop your signature sound!
