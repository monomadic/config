# VirtualDJ Native Effects Reference

Comprehensive list of all native audio and video effects, video transitions, and visualizations included with VirtualDJ.

## Audio Effects

### Time & Rhythm Effects

| Effect             | Description                                            |
| ------------------ | ------------------------------------------------------ |
| **BackSpin**       | Simulates a vinyl record spinning backward             |
| **Beat Brake**     | Applies a brake/slowdown effect synchronized to beats  |
| **BeatGrid**       | Beat-synchronized slicer effect using the beatgrid     |
| **Brake**          | Simulates slowing down/stopping like a turntable brake |
| **Break Start**    | Jump start effect for track beginnings                 |
| **Flippin Double** | Doubles every beat for rapid-fire transitions          |
| **Loop Out**       | Creates a loop that fades out                          |
| **Loop Roll**      | Temporary loop that releases back to original position |
| **Mobius**         | Continuous reverse/forward time manipulation           |
| **Mobius Saw**     | Sawtooth wave time manipulation effect                 |
| **Mobius Tri**     | Triangle wave time manipulation effect                 |
| **Recycler**       | Rhythmic stuttering and repeat effect                  |
| **Riser**          | Pitch rise effect typically used for buildups          |
| **Scale Down**     | Pitch-shift down effect                                |
| **Scratch DNA**    | Automated scratch patterns                             |
| **Slicer**         | Chops audio into rhythmic slices                       |
| **Slip Roll**      | Loop roll effect in slip mode                          |
| **Spiral**         | Spiraling time manipulation effect                     |
| **Stutter Out**    | Stuttering effect that fades out                       |
| **VinylBrake**     | Vinyl-style brake effect with customizable parameters  |

### Delay & Echo Effects

| Effect           | Description                                          |
| ---------------- | ---------------------------------------------------- |
| **Down Echo**    | Echo with pitch-shifting downward                    |
| **Ducking Echo** | Echo that ducks the dry signal                       |
| **Echo**         | Standard delay/echo effect with beat synchronization |
| **Hold Echo**    | Echo that holds and repeats a section                |
| **Low Cut Echo** | Echo with high-pass filtering                        |
| **MT Delay**     | Multi-tap delay effect                               |
| **Pitch Echo**   | Echo with pitch shifting                             |
| **Rev Delay**    | Reverse delay effect                                 |
| **Up Echo**      | Echo with pitch-shifting upward                      |

### Modulation Effects

| Effect         | Description                                |
| -------------- | ------------------------------------------ |
| **Cyclone**    | Rotating modulation effect                 |
| **Flanger**    | Classic flanging effect with feedback      |
| **Helix**      | Spiral modulation effect                   |
| **LFO Filter** | Low-frequency oscillator controlled filter |
| **Matrix**     | Complex modulation matrix                  |
| **Pan**        | Auto-panning effect                        |
| **Phaser**     | Classic phaser effect with multiple stages |
| **Ping Pong**  | Ping-pong delay/echo effect                |

### Filter Effects

| Effect     | Description                                   |
| ---------- | --------------------------------------------- |
| **Filter** | Resonant high-pass/low-pass filter (Color FX) |
| **Pumper** | Rhythmic volume pumping/ducking effect        |
| **Sweep**  | Filter sweep effect                           |
| **Wahwah** | Auto-wah effect                               |

### Pitch & Tone Effects

| Effect         | Description                             |
| -------------- | --------------------------------------- |
| **Distortion** | Adds distortion/overdrive to the signal |
| **Noise**      | Noise generator effect                  |
| **Pitch**      | Pitch shifting effect                   |
| **Pitch Down** | Dedicated pitch-down effect             |

### Special Effects

| Effect      | Description                           |
| ----------- | ------------------------------------- |
| **Cut**     | Beat-synchronized gate/cutting effect |
| **MergeFX** | Merges/combines effects together      |
| **Mute**    | Mute/unmute effect                    |
| **Reverb**  | Reverb/room ambience effect           |
| **Rider**   | Side-chain style ducking effect       |
| **Stems**   | Stems-based effect processing         |
| **Stretch** | Time-stretching effect                |
| **Vocals**  | Vocal isolation/processing effect     |

## Video Effects

### Transforms

| Effect              | Description                          |
| ------------------- | ------------------------------------ |
| **Blur**            | Blurs the video output               |
| **Blur Black Bars** | Blur with black bars overlay         |
| **Boom**            | Zoom/boom effect                     |
| **Boom Auto**       | Automatic zoom synchronized to beats |
| **Colorize**        | Changes video colors/hue             |
| **Negative**        | Inverts video colors                 |
| **Shake**           | Camera shake effect                  |
| **Spectral**        | Spectral visualization effect        |
| **Strobe**          | Strobe light effect                  |

### Overlays

| Effect          | Description             |
| --------------- | ----------------------- |
| **Karaoke**     | Karaoke display overlay |
| **Lyrics**      | Lyrics text overlay     |
| **Screen Grab** | Screen capture overlay  |
| **Text**        | Custom text overlay     |
| **Title**       | Title card overlay      |

### Sources

| Effect        | Description                  |
| ------------- | ---------------------------- |
| **Camera**    | Webcam/camera input source   |
| **Cover**     | Album cover art display      |
| **Lottery**   | Random visual lottery/picker |
| **Shader**    | Custom shader effects        |
| **Slideshow** | Image slideshow display      |

## Video Transitions

| Transition         | Description                     |
| ------------------ | ------------------------------- |
| **Additive**       | Additive blending transition    |
| **Blinds**         | Venetian blinds wipe            |
| **Cloud Dissolve** | Cloud-based dissolve            |
| **Color Swap**     | Color channel swap transition   |
| **Cube**           | 3D cube rotation transition     |
| **Doors**          | Door opening/closing transition |
| **Drain**          | Drain effect transition         |
| **Drain Light**    | Lighter drain effect            |
| **Droplets**       | Water droplet transition        |
| **Extreme Cut**    | Hard cut with effects           |
| **Fade**           | Standard crossfade              |
| **Fixed Grid**     | Grid-based transition           |

## Effect Parameters

Most effects support multiple parameters that can be adjusted:

### Common Parameters

1. **Strength/Dry-Wet** - Controls effect intensity (usually 0-100%)
2. **Length/Speed** - Controls timing, usually in beats (e.g., 1bt, 2bt, 4bt)
3. **Feedback** - Controls effect repetition/resonance
4. **Additional Controls** - Effect-specific parameters (filters, stages, modes, etc.)

### Parameter Examples

**Echo:**

- Parameter 1: Strength (%)
- Parameter 2: Length (beats)
- Parameter 3: Feedback (%)

**Flanger:**

- Parameter 1: Strength (%)
- Parameter 2: Speed (beats)
- Parameter 3: Feedback (%)
- Parameter 4: LFO Amplitude (%)

**Cut:**

- Parameter 1: Strength (%)
- Parameter 2: Length (beats)
- Parameter 3: Duty cycle (%)
- Parameter 4: Swing (%)
- Toggles: Low Cut, High Cut, Mute Beats, Video

**BeatGrid:**

- Creates rhythmic slices based on the track's beatgrid
- Visual interface for triggering individual beats

## Effect Usage

### VDJScript Control

Effects can be controlled via VDJScript:

```
effect_active 'Echo' on
effect_slider 'Echo' 1 50%
effect_slider 'Echo' 2 1bt
```

### Beat Synchronization

Many effects support beat synchronization:

- `1bt` = 1 beat
- `2bt` = 2 beats
- `4bt` = 4 beats
- Negative values possible for some effects (e.g., `-1bt`)

### Effect Slots

VirtualDJ supports multiple effect slots:

- FX1, FX2, FX3 (deck effects)
- Master FX (master output effects)
- Each slot can have its own effect with independent parameters

## Pre-Fader vs Post-Fader

Effects can be processed:

- **Pre-Fader**: Before volume fader and crossfader (default)
- **Post-Fader**: After volume fader and crossfader (requires supported hardware)

Set in Options: `fxProcessing` - `pre-fader` or `post-fader`

## Stems Effects

Effects can be applied to individual stems:

- Vocal
- HiHat
- Bass
- Instru
- Kick

Activated via Stems FX Pad on the Stems Pad Page.

## Effect Lists

VirtualDJ allows organizing effects into custom lists for easier access:

- Create multiple lists for different genres or use cases
- Manage lists via Audio Effects List Editor
- Access by scrolling to bottom of Effects List and clicking "More..." or "Manage..."

## Notes

- All native effects are included with VirtualDJ (no additional purchase required)
- Additional effects can be downloaded from VirtualDJ's plugin directory
- Effect availability and parameters may vary by VirtualDJ version
- Some effects have graphical user interfaces (GUIs) for advanced parameter control
- Effects can be combined and chained for complex sound design

---

## Additional Resources

- **VDJScript Verbs**: See VDJScript reference for programmatic effect control
- **Pads Editor**: Customize pad pages to trigger effects
- **Effect GUIs**: Click effect name or gear icon to open detailed controls
- **VirtualDJ Forums**: Community-created effects and tips
- **Plugin Directory**: https://virtualdj.com/plugins/
