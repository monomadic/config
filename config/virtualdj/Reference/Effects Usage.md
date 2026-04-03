# VirtualDJ FX Engines

This doc is **not just a list of effects** — it’s a mental model for **VirtualDJ’s different FX engines** and how to drive them from **skins** and **pad pages** without fighting the scripting model.

---

## 0) The 3 FX “engines” you’ll interact with in skins

VirtualDJ has multiple effect systems that look similar in the UI, but are controlled differently in VDJScript:

1. **Deck FX Slots (Audio FX rack)**  
   Think “FX1/FX2/FX3 on a deck” (and sometimes Master FX).  
   - Select an effect into a slot
   - Turn slot on/off
   - Move parameter sliders

2. **ColorFX (aka the Filter engine)**  
   Think “the filter knob, but instead of just HP/LP it can be Echo/Noise/etc.”  
   - Select which ColorFX preset is attached
   - Drive the **amount** (`filter`), which effectively turns it on/off

3. **Video FX / Video Transitions / Visualizations**  
   Similar concepts but different targets (video decks, master video output, transitions between sources).

Most “why doesn’t this work?” moments happen when you use **deck FX slot verbs** on **ColorFX**, or vice versa.

---

## 1) Canonical control patterns (cheat sheet)

### A) Deck FX Slots (Audio FX rack)

You control **a slot**, not “an effect name globally”.

**Typical pattern**
- choose effect in slot
- activate slot
- set sliders

Example (slot-based):
```vdjscript
effect_select 1 'Echo'
effect_active 1 on
effect_slider 1 1 50%
effect_slider 1 2 1bt
```

Toggle slot 1

```vdjscript
effect_active 1
```

Query if slot 1 is active

```vdjscript
effect_active 1 ? ...
```

Slot numbers and exact variants can differ across contexts (deck vs master), but the concept is always “slot owns activation + params”.

⸻

### B) ColorFX (Filter engine)

ColorFX is controlled by filter_select/colorfx + filter amount.

Select which ColorFX

```vdjscript
filter_select 'Echo'
```

Turn on (set amount) / off (0%)

```vdjscript
filter 50%     // “on” at a sane default
filter 0%      // off
```

Toggle (with a default-on value)

```vdjscript
param_equal filter 0% ? filter 50% : filter 0%
```

Select + toggle

```vdjscript
filter_select 'Echo' & param_equal filter 0% ? filter 50% : filter 0%
```

Query: is ColorFX effectively on?

param_greater filter 0% ? ...

Don’t use effect_active with a ColorFX name. Treat ColorFX as “selected preset + amount”.

⸻

C) “Filter” vs “ColorFX” naming confusion
	•	filter is the amount control for the ColorFX engine.
	•	filter_select (alias colorfx) chooses which ColorFX preset the filter amount is driving.

So “turning ColorFX on” usually means “set filter to non-zero”, not “activate an effect by name”.

⸻

2) How this maps into skins + pad pages

A) XML: where logic belongs
	•	Action body: what happens when you press the pad/button
	•	query=: what the UI uses to decide selected/down/blink/visibility
	•	color= / name text: can be dynamic depending on control type, but keep it simple first

Common gotchas:
	•	Use param_equal, not random compare verbs.
	•	Don’t quote backticked expressions ('…' turns them into literal strings).
	•	Always include the else side of ternaries in query (? ... : off) to avoid empty/undefined UI state.

⸻

B) Canonical pad patterns

1) “Select Echo ColorFX and toggle it”

```
<pad1
  name="Echo"
  query="param_greater filter 0% ? blink 500ms : off"
>
  filter_select 'Echo' & param_equal filter 0% ? filter 50% : filter 0%
</pad1>

2) “Momentary ColorFX (hold = on, release = off)”
Use the down ? ... : ... pattern:

<pad1
  name="Echo (hold)"
  query="param_greater filter 0% ? on : off"
>
  down ? filter_select 'Echo' & filter 50% : filter 0%
</pad1>

3) “Deck FX slot: Echo toggle on slot 1”

<pad1
  name="Echo FX1"
  query="effect_active 1 ? blink 500ms : off"
>
  effect_select 1 'Echo' & effect_active 1
</pad1>
```

⸻

3) Audio Effects (native) — organized by how you usually use them

The lists below are still useful, but the control approach depends on whether you’re using FX slots or ColorFX.

Time & Rhythm (mostly FX slot-friendly)

Effect	Typical Use
BackSpin / VinylBrake / Brake	Turntable-style stops/backs
Slicer / BeatGrid / Recycler	Beat-chopping rhythmic FX
Loop Roll / Slip Roll	Temporary looping w/ release
Stutter Out	Exit/transition stutter
Riser / Scale Down	Build-ups / drops

Delay & Echo (FX slots + sometimes ColorFX variants)

Effect	Typical Use
Echo / MT Delay / Ping Pong	Transition tails, rhythmic repeats
Hold Echo	Freeze-style echo
Ducking Echo	Cleaner echo in busy mixes
Pitch/Up/Down Echo	Hype FX / ear-candy

Modulation

Effect	Typical Use
Flanger / Phaser	Sweeps, movement
Pan	Space / stereo motion
LFO Filter	Automated filtering

Filter-family

Effect	Typical Use
Filter (ColorFX)	HP/LP or ColorFX engine (depending on selection)
Wahwah / Sweep	Movement filter FX
Pumper / Rider	Sidechain-ish groove

Special / Utility

Effect	Typical Use
Reverb	Space, tail
Mute / Cut	Gates / kills
Stems / Vocals	Stem-scoped processing when mapped that way


⸻

4) Video Effects + Transitions (how to think about them)

Video has the same conceptual split as audio:
	•	“effects” applied to a source (a video deck, a camera input, etc.)
	•	“transitions” applied between sources

Video Effects

Transforms (post-processing)

Effect	Typical Use
Blur / Negative / Colorize	Looks + masking
Shake / Strobe	Energy / accent
Boom / Boom Auto	Beat zooms

Overlays (rendered on top)

Effect	Typical Use
Lyrics / Karaoke / Text / Title	UI overlays
Screen Grab	live capture overlay

Sources (generate video)

Effect	Typical Use
Camera	live input
Slideshow / Cover	media-driven visuals
Shader	custom pipeline

Video Transitions

Transition	Typical Use
Fade / Additive	clean blend
Cube / Doors / Blinds	obvious visual transitions
Dissolves / Droplets	texture transitions


⸻

5) Effect parameters: how to approach them in skins

A) Think in “presets” for pad pages

Pads want repeatable results. Instead of exposing 4 sliders, hardcode a “good preset”:

Example: “Echo 1bt, medium feedback”

effect_select 1 'Echo'
& effect_active 1 on
& effect_slider 1 1 50%
& effect_slider 1 2 1bt
& effect_slider 1 3 35%

B) Make the UI reflect state reliably
	•	FX slot: query effect_active <slot>
	•	ColorFX: query param_greater filter 0%
	•	If you care about “which one is selected”, query label/name (when available) but avoid brittle string compares unless you’ve verified the exact returned label.

C) Avoid “half-on” UX

If you toggle ColorFX by amount, decide what “on” means:
	•	filter 50% is a common neutral default
	•	you can store/restore a previous amount using vars, but keep it deterministic for pads

⸻

6) Pre-fader vs post-fader (why skins should care)

This affects how tails behave when you cut volume/crossfader.
	•	Pre-fader: FX continues even if you cut audio? depends on routing; often tails are less “natural”
	•	Post-fader: more “DJ mixer-like” tails when cutting volume

If you’re designing pads for transitions (Echo Out, Reverb Out), you want to know what the user’s fxProcessing option implies.

⸻

7) Stems FX (mental model)

Stems adds a targeting layer (vocal/bass/etc.) to whatever control scheme you’re using.
In pad design:
	•	Decide if the pad affects whole mix or a stem
	•	Make the UI show “has stems / stems ready” so pads don’t feel broken

⸻

8) Practical design rules for skins & pad pages
	1.	Pick your engine first
“Is this a ColorFX moment or an FX-slot moment?”
	2.	Pads should be deterministic
Avoid relying on whatever effect happens to already be loaded unless that’s intentional.
	3.	Always write query like it’s a state machine
... ? blink 500ms : off (don’t leave it empty)
	4.	Keep selection separate from activation
	•	FX slots: select effect vs activate slot
	•	ColorFX: select preset vs set amount
	5.	Don’t mix verbs between engines
If it feels like “it should work”, it’s probably the wrong engine.

⸻

Additional Resources
	•	VirtualDJ Skin SDK: https://www.virtualdj.com/wiki/Skin%20SDK
	•	VDJScript verbs reference: https://virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html
	•	Script params/variables/math: https://virtualdj.com/forums/251658/General_Discussion/Script_Param_Variable_Maths.html
	•	Plugins directory: https://virtualdj.com/plugins/

If you want, paste one of your real pad pages (sampler / FX / ColorFX page), and I’ll rewrite it into a consistent “engine-correct” style with reliable `query` states and minimal brittleness.
