# Lexicon cue-point analysis: implementation specification

This describes the cue-section detector recovered from the **locally installed
Lexicon 1.11.14 bundle**, inspected on 2026-09-30. It is sufficient to build a
standalone detector when used with the [exact reference appendix](REFERENCE.md).
The appendix supplies every ordered correction, constant, and genre rule; this
specification supplies the architecture, meaningful names, formulas, and porting
contract. Start a new implementation from the contract below, not from the UI.

**Finding:** this path runs locally in a JavaScript worker. It uses filtered audio,
RMS energy, transient counts, musical timing priors, and sequential corrections.
There is no model loading or remote inference in the traced cue detector. This
does not establish how the constants were chosen or describe every Lexicon version.
The bundled help's machine-learning language is not evidence of a trained model
being used in this path.

## 1. Scope and evidence

Included:

- Start, first drop, first breakdown, second drop, second breakdown, last-beat
  anchors, and the optional emergency loop.
- First-channel preprocessing, genre handling, scoring, ordered refinements,
  defaults, output format, and the boundary between anchors and cue templates.
- An exact, independently evaluable source excerpt and reproducible synthetic
  differential probes against the complete original worker.

Inputs supplied by the caller, rather than algorithms specified here:

- Audio decoding into floating-point PCM and the decoded sample rate.
- A **constant BPM and first-beat timestamp**. Lexicon can calculate these in a
  separate beatgrid routine, but that routine is outside this specification.
- Application storage, UI, export formats, and custom/manual-anchor selection.

The energy rating and musical-key detectors are separate algorithms. Their
coefficients are not used to decide the cue sections described here.

### Provenance

Source archive: `/Applications/Lexicon.app/Contents/Resources/app.asar`.
Version is from the local `Info.plist`; pin the hashes because the version string
alone does not establish identical contents or an unmodified vendor build.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `app.asar` | 29813701 | `a970dc093bf7a8b66ad4f56e7b8ded11e60b344fb9d5ae637166b09e813272d8` |
| `renderer/main_window/182.index.worker.js` | 298854 | `d8bc3b60b1989094d00945f6b03b7a48a63490d390b76f34cb7158345203113e` |
| `renderer/main_window/index.js` | 19177397 | `0ca24a0875d25117d6f9a909272158e66eb8b0e97fb2afa5a3bc78c8a1554101` |

The renderer constructs `new Worker(r(87074))`; module `87074` resolves to
`182.index.worker.js`. `AudioAnalyzer` sends settings and sample buffers to that
worker. The worker's `self.onmessage` calls `le(...)` when cue analysis is enabled
and a truthy tempo is available, returning `sectionPoints`. The renderer converts
these anchors into saved cues using the template.

**Verification level:** source inspection plus 49 successful synthetic differential
comparisons of the excerpt against the original worker. No real-track accuracy
claim, live Lexicon execution, or independent cross-language implementation has
been verified. See [probe details and reproduction](PROBES.md).

## 2. Recommended implementation interface

```text
analyze_sections(
    raw_channel_0: float32[],
    sample_rate_hz: float64,
    bpm: float64,
    first_beat_seconds: float64,
    minimum_breakdown_beats: integer = 64,
    include_emergency_loop: boolean = false,
    prefiltered_channel_0: optional float32[],
    drop_at_start: "never" | "highEnergyOnly" = "never",
    genre: optional string
) -> SectionPoint[]
```

Suggested validation for a new implementation: finite PCM, positive finite BPM,
positive sample rate, and a finite nonnegative first beat. The recovered function
does not comprehensively validate these: add guards explicitly rather than
claiming its behavior on invalid values is well-defined. The transient window
requires `floor(sample_rate * 0.005) >= 1` to avoid a zero-step loop.

Use one tempo throughout. The app selects the first existing tempo marker when
available; this is not a variable-tempo beat-grid algorithm. `first_beat_seconds`
is an alignment origin, not the detected drop and not necessarily the first sound.

```text
SectionPoint {
    markerType: "start" | "drop" | "breakdown" | "second_drop"
                | "second_breakdown" | "lastbeat" | "emergency_loop",
    startTimeSeconds: number,
    duration: number,          // seconds; see below
    setByUser: false,
    name: string,
    color: string,
    cueType: "memory_cue" | "loop_active",
    loopDuration?: number,    // BEATS, not seconds
    enabled?: boolean
}
```

Ordinary section markers have a duration of the first four-beat section (normally
`240 / bpm`). Last-beat duration is `60 / bpm`. Emergency-loop duration is zero,
with `loopDuration = 16` and `enabled = true`. These `duration` values are marker
metadata, **not the length of the musical section**.

Worker defaults for presentation: Start/Drop/Second Drop are red; Breakdown/Second
Breakdown blue; Last beat orange; S.O.S. magenta. All are memory cues except S.O.S.,
which is an active loop. Templates can overwrite these properties. The default
renderer template uses hot cues, enables the six non-loop anchors, and has no
emergency-loop row. It sets breakdown minimum to 64 beats, drop-at-start to
`never`, start behavior to `atFirstBeat`, and custom anchors off.

Missing detections are omitted, not represented as timestamp zero. Start is
always constructed. Coincident markers are retained. Final anchor sorting is
stable by timestamp; do not deduplicate equal times.

## 3. Units and numeric conventions

Throughout the specification:

- `x`: raw channel-0 PCM; `fs`: sample rate; `D = len(x)/fs`: duration in seconds.
- `b = 60/bpm`: seconds per beat; `q = 4*b`: seconds per section.
- A **section is four beats**, even if the music uses another time signature.
- `E[i]`: RMS energy of section `i`; section indices are not beat indices.
- `mean(E, a, z)` averages the half-open interval `[a,z)` after the explicitly
  stated bounds clipping. Do not silently replace a fixed divisor with the
  number of available samples where the reference uses a fixed divisor.
- `B` or `baseline`: mean energy of the eight sections beginning at a drop;
  return `1e-9` if empty or nonpositive.
- `nearest_section(time)`: earliest section with minimal absolute time distance.
- Maximum-score searches retain the first candidate on ties (`>` updates).
  The emergency-loop search runs backward, so ties favor its later candidate.

JavaScript scalar arithmetic is double precision; PCM buffers are Float32Array.
Preserve float32 writes between filter stages and double-precision running state.
`Math.round` rounds half toward positive infinity, unlike ties-to-even rounding
in some languages. Negative remainders use JavaScript semantics; positive phase
modulo is written explicitly as `((value % modulus) + modulus) % modulus`.
Percentiles below are indexed order statistics, not interpolated quantiles.

## 4. Audio preprocessing: two distinct bass representations

### 4.1 The normal renderer-supplied representation

For cue analysis the renderer uses the complete decoded AudioBuffer. It builds an
OfflineAudioContext at that buffer's sample rate, length, and channel count and
cascades seven Web Audio low-pass BiquadFilterNodes, in this order:

```text
800 Hz -> 400 Hz -> 400 Hz -> 200 Hz -> 200 Hz -> 200 Hz -> 200 Hz
```

It supplies only `frequency` and `type`; Q and the other parameters are left at
the Web Audio implementation defaults. It renders the complete track from time
zero and takes output channel 0. Raw audio passed alongside it is also channel 0.
There is **no stereo averaging in this cue path**. Do not substitute a mono mix
if exact behavior is the goal.

For parity, reproduce this Web Audio preprocessing or capture its output as a
fixture. The decoding sample rate is passed through; this stage does not request
one fixed universal sample rate.

### 4.2 The worker's direct-call fallback

When `K` receives `null` or `undefined` for the prefiltered array it constructs the
same frequency sequence using the local `F` biquad implementation with
`Q = 1/sqrt(2)`. At each cutoff `fc`:

```text
w = 2*pi*fc/fs
alpha = sin(w)/(2*Q)
a0 = 1 + alpha
b0 = (1-cos(w))/(2*a0)
b1 = (1-cos(w))/a0
b2 = b0
a1 = -2*cos(w)/a0
a2 = (1-alpha)/a0
out[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]
         - a1*out_state[n-1] - a2*out_state[n-2]
```

Each stage starts with zero state, stores each output to float32, but keeps the
unrounded double result in its recurrence state. Do not assume this is identical
to the renderer's Web Audio filters: the caller leaves Q implicit and the fallback
sets it explicitly. This documentation does not claim measured filter equivalence.

A subtle wrapper issue: `self.onmessage` constructs a Float32Array even when the
incoming prefilter is missing. That gives an **empty non-null array**, which does
not trigger `K`'s null fallback. Direct `le(..., null, ...)` and sending a null
prefilter through the original worker wrapper are different cases.

### 4.3 The worker's six-pass one-pole representation

Most structural decisions use `v(x, fs, 300, 6)`:

```text
alpha = 1 / (1 + fs/(2*pi*cutoff))
y = float32 copy of x
repeat 6 times:
    state = 0
    for n = 0 .. len(y)-1:
        state = alpha*y[n] + (1-alpha)*state
        y[n] = float32(state)
```

The first-drop genre correction also uses this function at **80 Hz**, six passes.
There is no zero-phase filtering, delay compensation, or amplitude normalization.

## 5. Feature extraction

`_(pcm, fs, bpm, firstBeat, 4, 0.02)` produces four-beat sections:

1. Set `section_samples = floor(q*fs)`.
2. First section start is `firstBeat - floor(firstBeat/q)*q`, which places it in
   `[0,q)` for finite inputs. Subsequent starts advance by `q` seconds.
3. For each start `s`, use samples `[floor(s*fs), min(floor(s*fs)+section_samples,N))`.
   Energy is `sqrt(sum(sample^2)/sample_count)`. Store actual duration
   `sample_count/fs`, including a truncated final section.
4. Beat centers begin at `firstBeat + ceil((s-firstBeat)/b - 1e-9)*b` and advance
   by `b`. Include a center while `floor(1e6*center) <= floor(1e6*(s+q))`.
5. At each center, measure RMS in the clipped half-open window
   `[floor((center-0.05)*fs), floor((center+0.05)*fs))`.
6. Store the individual `beatEnergies` and their mean `beatStrength`.

The inclusive endpoint in step 4 can include **five beat windows** in a four-beat
section. Keep this behavior. The emergency-loop score reads only indices 0..3.

For consecutive groups of four sections, regress `beatStrength` against local
index 0..count-1. Let slope be ordinary least squares and threshold be
`mean(beatStrength)*0.02`, or `1e-6` if the mean is nonpositive. Label every section
in the group `up`, `down`, or `flat` according to slope > threshold, < -threshold,
or neither. Also store adjacent `energyDelta`. These fields exist in the feature
records but do not drive the recovered cue choices; preserve them only if needed
for reference-level data parity.

### Full-band transient count

First-drop candidate B also uses raw-audio attack counts:

- Divide raw PCM into windows of `floor(fs*0.005)` samples; keep max absolute
  amplitude in each window.
- First attack value is zero. Later values are `max(0, peak[i]-peak[i-1])`.
- Threshold = `1.5 * mean(strictly positive attack values)`, or zero if none.
- For each section map its start and end to indices with `floor(seconds/0.005)`.
  Count interior attack values strictly greater than both neighbors and the
  threshold. This mapping uses 0.005 seconds even when `fs*0.005` was rounded.

## 6. First drop and start anchor

The source entry is `K`. Use these names when porting:

- `A`: sections from the supplied seven-biquad audio; `iA = j(A)`.
- `E`: sections from the six-pass 300 Hz audio; `iB`: candidate from energy plus
  transient counts.
- `tA`, `tB`: candidate times; `strengthA = A[iA].energy/max(A.energy)` and
  `strengthB = E[iB]/max(E)`, with `1e-9` guards.
- `sustainB = mean(E[iB+1 .. iB+8])/E[iB]` over available sections.
- `jumpB = E[iB]/max(1e-9, mean(previous up to 8 sections))`.
- `crossA = E[iA]/max(E)`; source defaults this to 1 if `iA` is unavailable.

### 6.1 Initial candidates

For each candidate section, calculate the forward mean over up to four sections.
Candidate A is the earliest index satisfying both:

```text
forward_mean >= 0.85 * maximum_forward_mean
section_energy >= 0.65 * maximum_section_energy
```

Candidate B uses the same tests on `E`, plus:

```text
transient_count >= 0.20 * maximum_section_transient_count
```

Both can move to an earlier stable plateau using `z`:

- Scan from index zero to strictly before `candidate-16`.
- Require energy at least the median (`sorted[floor(N/2)]`).
- Require the following seven sections to stay within 0.75..1.25 of that energy.
- Require the previous up-to-eight-section mean below 0.6 of that energy.
  Skip a candidate with fewer than four predecessor sections if its index > 4.
- Take the first match.

Apply this search to A only if `iA/N >= 0.4`; to B only if `iB/N >= 0.3`.
Replace the original candidate only if the minimum energy between `earlier+8`
and `candidate` is below half the earlier energy. For B, also require that the
original candidate energy is **less than 1.2 times** the earlier energy.

### 6.2 Resolve A versus B

If only one candidate exists, take it. If both exist:

| Condition | Decision |
| --- | --- |
| `abs(tA-tB) <= 16` **seconds** | Begin with B, then apply the two close-candidate corrections in `K`. |
| B later, `iA/N < .15`, `.15 <= iB/N <= .4`, `strengthB >= .7`, `sustainB >= .8`, `jumpB >= 1.3` | B |
| B later, other cases | Compute `preferB` and `protectA` below; B iff `preferB && !protectA`. |
| B earlier | B iff the earlier-B tests below succeed; otherwise A. |

For B later:

```text
preferB = sustainB >= .85 && jumpB >= 1.4 &&
          (strengthA >= .7
             ? strengthB-strengthA >= .01 || (strengthB >= .7 && crossA < .6)
             : strengthB >= .7)
protectA = strengthA >= .7 && .15 <= iA/N <= .7 && .55 <= crossA < .75
```

Cancel `protectA` if a run of at least four sections below `.5*E[iA]` exists
between `iA+4` and `iB`.

For B earlier, choose B if either:

```text
strengthA >= .7 && strengthB >= .7 && sustainB >= .85 && jumpB >= 1.4 && crossA < .6
```

or:

```text
strengthB >= .7 && sustainB >= .85 && jumpB >= 1.4 &&
iA/N >= .55 && .08 <= iB/N <= .45
```

The two close-candidate corrections, in order:

1. If `iB/N >= .25`, scan section indices 4 through `<iB-16`. Seek energy
   `>=.7*max(E)`, at least four of the next eight sections `>=.7` of candidate
   energy, a preceding four-section minimum `<.3` of candidate energy, and a
   later minimum from candidate+9 to `<iB` `<=.2` of candidate energy. Take the
   first match.
2. If A precedes B by 2..12 sections and `iA>=2`, prefer A when `E[iA]>=.5*max(E)`
   and either of the preceding two sections is `<.15*E[iA]`.

### 6.3 Ordered corrections to the selected drop

**Order is observable behavior.** Each stage sees the modified result of earlier
stages; do not run them independently and combine votes. Exact inequalities and
loop endpoints are retained in `K`, `V`, and `w` in the appendix.

1. If the candidate is in section 0..4, detect a subsequent long quiet passage
   below `.3*max(E)`, then a return to `.7*max(E)`. The minimum quiet length is
   7 or 12 sections depending on candidate position and depth. Move to the
   return, then apply `V`.
2. Otherwise apply `V`: detect an early sustained burst followed by a quiet gap
   and a substantial return. If it does not match, a sustained high-energy
   opening can instead move the candidate back to section zero.
3. Move one section earlier if the predecessor is `>=.7` of current energy and
   the section before that is `<.6` of it.
4. Apply `w`: skip a weak opening toward a sustained stronger arrival, using
   previous/following eight-section means and up to 16 sections of lookahead.
5. Seek an earlier 16-beat boundary, up to three sections earlier, only if the
   intervening energy stays high and the earlier boundary has a clear attack.
6. Detect a loud four-section decay followed by a stable lower plateau; move
   forward four sections when its exact conditions hold.
7. Apply phase corrections relative to the first-beat origin: phase 1 moves -1
   section; phase 3 moves +1; phase 2 conditionally moves +2. Phase 0 can move
   +4 when the next phrase is stronger. These are not unconditional rounding.
8. Apply the genre-specific 80 Hz correction below.

For an exact port, translate the corresponding blocks, including `V` and `w`,
from the reference. These descriptive names explain purpose; they do not replace
the preserved predicates. The appendix is part of this specification.

### 6.4 Genre-specific correction

The classifier lowercases, NFD-normalizes, removes combining marks, replaces
hyphens/slashes/ampersands/plus with spaces, removes other non-ASCII-alphanumeric
characters, collapses whitespace, and trims. It applies non-music exclusions and
an **ordered genre regex table**, then returns `house`, `techno`, `hardcore`, or
null. Other genres return null. Preserve rule order for subgenres and mixed labels;
full mappings are in `g`, `y`, `k`, `M` in the appendix.

For those three classes, analyze 80 Hz six-pass sections:

- Find the first **completed** quiet run: techno uses energy `<.35*max`, length
  at least 3 sections; house/hardcore use `<.30*max`, length at least 4.
- A trailing run that never returns above the threshold is not a match (`J`).
- House/techno apply only if the current drop index is <=4.
- Hardcore applies only if current drop index <=40% of section count and the
  quiet run begins after that index.
- Starting just after the run, use the first energy `>=.7*max` as the new drop.

Start time is `min(firstBeat, dropTime)` when a drop exists, otherwise firstBeat.
It is not a silence-onset detector.

## 7. Breakdown detector

`vt(raw, fs, bpm, firstBeat, dropTime, D, minimumBeats, genre, relaxed=false)`.
Use 300 Hz six-pass sections. Let `d = nearest_section(dropTime)`,
`B = mean(E[d:d+8])`, floored to `1e-9`, and `dropFraction = time[d]/D`.

### 7.1 Candidate locations and score

Scan `k = 1..30`, candidate `i = d + 16*k` sections, i.e. every **64 beats**.
Stop if `i+16 > section_count`. For each:

```text
after  = mean(E[i:i+16])
before = mean(E[max(d,i-16):i])
loss   = max(0, 1-after/B)
edge   = max(0, (min(before,B)-after)/B)
position_prior = exp(-0.5*((time[i]/D - .48)/.18)^2)
spacing_prior  = exp(-0.5*((k*64 - 192)/96)^2)
score = loss * edge * min(1,before/B) *
        (hardcore ? spacing_prior : max(position_prior,spacing_prior))
```

Zero the score when less than 30 seconds remain. Minimum candidate track fraction:

- `.10` if relaxed.
- `.15` if the original drop is before 10%, `loss>.75`, and no strong middle
  breakdown candidate was found in the prepass described next.
- `.25` otherwise.

The prepass runs only for an early drop (<10%). It scans the same candidate grid
in track fraction `.35..57` and sets a flag if `loss*edge > .3`.

For candidates after 55% of the track, halve score if an earlier run of at least
8 sections below `.3*B` exists between the drop and the candidate. Pick the first
maximum score; if it is zero, no breakdown.

### 7.2 Refine the boundary

`Ft` searches near the selected candidate on a grid relative to the drop:

- Grid step 4 sections (16 beats), initially phase offset 0.
- Search from snapped candidate-16 to snapped candidate+16 sections, forward.
- Remain after the drop and at least 30 seconds before track end.
- Require current energy `<.4*mean(previous up to 4 sections)` and that preceding
  mean `>=.5*B`.
- Require current or next energy `<.3*B`, and the next six-section mean `<.3*B`.

Take the first match. If none, keep the original candidate if its next-six mean
is `<.5*B`; otherwise try the same grid at offset 2 sections and use it if found.

Then validate the refined candidate `i`:

- Mean of up to 8 sections starting at `i+1` must be `<.85*B`.
- Mean of up to the first 4 of those must be `<.8*B`.
- Mean of up to 16 sections before `i`, bounded by the drop, minus the first mean
  above must be `>=.2*B`.
- Some section from `i+16` onward must have energy `>=.4*B`.

### 7.3 Final adjustments and minimum duration

The remaining `vt` blocks, in order, locate the onset of a gradual energy descent,
possibly move backward to an earlier abrupt edge, and align relative to the drop.
Preserve them from the appendix, especially:

1. If the predecessor isn't already `>=.7*B`, search up to 12 sections backward
   for a `>=.7*B` to `<=.6*previous` edge, with fewer than three later rebound
   sections `>=.85*B` before the original refined candidate.
2. If that fails, search up to 24 backward while energies stay below `.5*B`,
   looking for a predecessor `>=.5*B` and a ratio `>=1.5`.
3. A preceding run of at least eight sections in `[.3*B,.7*B)` may extend the
   boundary backward to a previous section with energy `>=.7*B`.
4. Snap `round((candidate-d)/4)*4 + d`, with a minimum `d+4`.
5. If neither a strong predecessor nor the abrupt-edge searches determined the
   boundary, use the later sustained return and the preceding active region to
   adjust the duration to a multiple of 16 **sections**, not 16 beats.

Find the first run of eight sections `>=.85*B` after the original refined
candidate, starting the search at its next section. If the run starts at `r`,
compute `durationBeats = 4*(r-finalBoundary)`. Reject only if
`16*ceil(durationBeats/16) < minimumBeats`. If no such run is found, this duration
check does not reject the candidate. Return the boundary's timestamp.

## 8. Second-drop detector

`Ht(raw, fs, bpm, firstBeat, dropTime, breakdownTimeOrNull, D)`.
Use 300 Hz six-pass sections, drop index `d`, baseline `B`.

```text
breakIndex = breakdown exists after drop ? nearest_section(breakdown) : d+48
breakTime  = breakdown exists after drop ? breakdown : dropTime+192*b
startIndex = max(d+24, breakIndex+16)
startIndex = d + ceil((startIndex-d)/4)*4
```

Thus scanning starts at least **96 beats after the drop** and **64 beats after
the breakdown reference**, on a 16-beat grid relative to the drop.

For candidate `i`, require `i+16 < N`; stop if `i-breakIndex > 80` sections or
`time[i]/D > .82`. Scan in increments of 4 sections:

```text
after  = mean(E[i:i+16])
before = mean(E[max(0,i-16):i])
energetic = after >= .6*B
passes = energetic &&
         (before <= .7*B || (E[i] >= .85*B && E[i-1] < .4*B))
position_prior = exp(-0.5*((time[i]/D-.65)/.10)^2)
spacing_prior  = exp(-0.5*(((time[i]-breakTime)/b-128)/64)^2)
```

Set the position prior to zero if `(dropFraction<.15 && breakTime/D<.35)` or
`(breakTime+128*b)/D < .55`. Score:

```text
min(1,after/B) * max(0,(after-before)/B) * max(position_prior,spacing_prior)
```

Selection and fallback order:

1. Among passing candidates choose first maximum score, refine with `zt`, label
   internal tier `principled`.
2. If none pass and breakdown is **null**, return no result. This null check is
   distinct from whether breakdown is after the drop.
3. If candidates exist, choose the first whose forward energy is >=95% of the
   maximum candidate forward energy; refine with `zt`; tier `energy-fallback`.
4. Otherwise call `Ot`; tier `heuristic-fallback`.

`Ot`: try breakdown+128 beats if breakdown>drop and that time is strictly before
track end; otherwise try drop+384 beats if before end; otherwise use `.65*D`.
`qt` snaps to the nearest section on the 4-section grid relative to drop (earliest
wins ties). The empty-section early path calls `Ot` directly.

### Ordered second-drop refinement (`zt`)

There are **14 sequential transformations**, all preserved in the appendix:

1. Walk backward up to 16 sections through an energy plateau, allowing certain
   one-section dips, to locate its leading edge.
2. Apply common weak-arrival correction `w`.
3. Skip a brief weak section toward a sustained stronger four-section region.
4. Move through a gradual rise to near the strongest subsequent four-section mean.
5. Skip an energetic but short false arrival followed by a low gap and long return.
6. Check for a stronger return 3..5 sections later after an intervening dip.
7. Handle four- and eight-section lead-ins, punctuated starts, and decaying ramps.
8. Resolve a two-section pickup after a moderate-energy predecessor.
9. Skip a later quiet gap to a stronger grid-aligned four-section region.
10. If energy is below `.3*B`, permit a one- or two-section move to a >=3x arrival.
11. Correct half-phrase offset (phase 2 relative to the first drop) after a dip.
12. Skip a six-section plateau plus dip to a stronger arrival eight sections later.
13. Skip a rising/falling flourish to a stable plateau four sections later.
14. Skip a loud four-section decay to a stable lower plateau.

These transformations are independent ordered code blocks; several can apply in
one call. Their guards and thresholds are necessary for parity. Use the complete
`zt` definition rather than implementing only the descriptions above. Diagnostic
tier strings are not exposed on the final marker, but matter to orchestration.

## 9. Orchestration and drop-at-start policy

The main function `le` performs:

```text
(start, drop1) = detect_first_drop()
if drop1 exists:
    breakdown1 = detect_breakdown(drop1)
    (drop2, tier) = detect_second_drop(drop1, breakdown1)
    optionally revise drop1 according to the policy below
append start and existing drop1/breakdown1/drop2
if drop2 exists:
    breakdown2 = detect_breakdown(drop2)
    if absent: try terminal-breakdown fallback
    append if found, relabeled second_breakdown
lastBeat = detect_last_beat()
append if found
if emergency loop requested and drop1 exists:
    find loop between drop2-or-drop1 and breakdown2-or-lastBeat-or-duration
append if found
stable sort by timestamp
```

### `dropAtStart = "never"` is a conditional policy

It runs only if a second drop exists and duration>0. Propose promoting drop2 to
drop1 when either:

- `drop1/D < .10` and `drop1 < 12` seconds; or
- Both drop1 and drop2 are before 50%, and treating drop2 as the first drop
  yields another second drop with tier `principled`.

Before promoting, classify the overall 300 Hz section-energy pattern:

1. Reference = `sortedEnergy[floor(.95*(N-1))]`, or `1e-9` if zero.
2. Trim leading/trailing sections below `.05*reference`.
3. Require at least eight retained sections.
4. Check fraction `>=.5*reference` is at least `.55`, longest run
   `<.2*reference` is at most 4 sections, and total fraction below `.2*reference`
   is at least `.075`.

If all checks pass, cancel promotion and recompute breakdown1 with relaxed=true,
then drop2 from that new breakdown. Otherwise promote drop2, preserve Start unless
it is later than the promoted drop, and recompute both subsequent anchors.

`highEnergyOnly` takes the ordinary path without this promotion block. It does
not invoke a separate energy-rating model. Even `never` can retain a drop at zero
when no suitable second drop exists.

### Terminal second-breakdown fallback

Only after normal breakdown detection on drop2 fails:

- Let `d` be drop2's nearest section and `B` its eight-section baseline.
- Scan `i=d+16,d+32,...` while `i+8 < N`.
- Require next-eight mean `<=.7*B`, previous-eight mean minus next-eight mean
  `>=.25*B`, and **no** section from `i` onward with energy `>.95*B`.
- Walk backward while the previous section remains `<=.7*B`, bounded by
  `max(d+1,i-16)`. Use that time.

This fallback does not apply the normal minimum-breakdown-length parameter and
allows an outro without a later energetic return.

## 10. Last-beat / fade-out anchor

This works at beat resolution, not four-beat-section resolution:

1. `Nbeats = max(0,floor((D-firstBeat)/b))`; if <4, return none.
2. At beat centers `firstBeat+i*b`, calculate RMS in +/-0.05-second windows for
   raw and six-pass 300 Hz PCM. Clip sample indices to available audio.
3. `rawRef` and `bassRef` are separate 75th percentiles, indexed
   `sorted[floor(.75*Nbeats)]`. If rawRef<=0, return none.
4. Make trailing four-beat maxima for each series, including the current beat.
5. Active beat = raw trailing max `>=.55*rawRef` AND bass trailing max
   `>=.30*bassRef`.
6. Find the end of the **last active run of at least 16 beats**. If none, take
   the last beat whose raw trailing maximum passes `.55*rawRef`. If none, fail.
7. Search backward from `min(Nbeats-1, activeEnd+64)` to strictly after activeEnd
   for a possible later ending beat. Require raw RMS `>=.35*rawRef`, some beat
   in the next up-to-six beats `<.15*rawRef`, no later beat after that six-beat
   window `>=.35*rawRef`, and mean raw RMS from activeEnd+1 through candidate
   `>=.45*rawRef`. Use the first passing candidate in this backward search.
8. Round the selected beat index to the nearest multiple of 16. If that index
   is outside the beat array, round downward instead; clamp negative to zero.
9. Return `firstBeat + roundedIndex*b`.

This can deliberately land before the last audible sound, or coincide with a
breakdown. The UI's “End / Fade-out” label does not mean precise end-of-file or
silence detection.

## 11. Emergency loop

Use 300 Hz four-beat sections. Region start = drop2 if available, otherwise drop1.
Region end = breakdown2 if available, otherwise lastBeat if available, otherwise D.

- `startIndex`: first section whose time >= region start.
- `endIndex`: first section whose time >= region end, or N if absent.
- Earliest candidate = startIndex+4 sections.
- Latest candidate = endIndex-8 sections. Thus leave four sections before the
  four-section loop and four after it.
- Reference maximum = max energy from earliest candidate through `<endIndex`.
- Scan **backward** from latest candidate to earliest, step four sections.

For each four-section / 16-beat candidate:

```text
meanE = mean(its 4 section energies)
relativeLevel = meanE / referenceMaximum
sectionVariation = (maxE-minE)/meanE
```

Reject if meanE<=0, relativeLevel<.45, or sectionVariation>.7. For each beat slot
0..3, compute `(max-min)/mean` across that slot in the four sections, including only
positive-mean slots in the final average; call this `beatVariation`.

```text
recency = (candidate-earliest)/max(1,latest-earliest)
score = recency + .5*relativeLevel - .8*sectionVariation - .6*beatVariation
```

Choose the strictly highest score. **If all candidates fail, use the first
available candidate visited (the latest one)**. Thus the fallback can violate the
energy/stability tests. Return 16 beats, not four beats; omit if no usable region
or nonpositive reference maximum.

## 12. Turning anchors into user cues

Keep this layer separate from structural detection. This is an integration map,
not a full specification of Lexicon's UI/storage behavior.

1. Match template rows to anchors by marker type. Checked rows determine which
   cues are emitted; copy their names, colors, cue types, and loop lengths.
2. Second-drop selection requires a first drop and a different timestamp.
   Second-breakdown selection requires a first breakdown and a different timestamp.
   The raw detector itself does not enforce these UI selection conditions.
3. For a checked normal row relative to an existing anchor:
   `cueTime = anchorTime +/- beatsOffset*(60/track.bpm)`. Add it only if the
   application range predicate accepts its time within track duration. Endpoint
   inclusivity of that helper has not been independently specified here.
4. Normal-row offsets are computed **before** the special Start anchor is changed
   to an existing cue or to zero. Do not silently reverse this order in a parity port.
5. `atFirstBeat` leaves detected Start unchanged; `atExistingCue` uses the first
   existing cue when one is present; `atZero` changes Start to zero.
6. Loop end = cue time + loopDuration*(60/track.bpm). The renderer uses track BPM
   for templates, which may be a separately rounded stored BPM; the worker can
   operate on an unrounded tempo. Keep these concepts distinct.
7. Cue positions can follow working-list order or template row positions. Memory
   cues get a Rekordbox metadata marker; active loops get an active-loop flag.
8. Custom-anchor mode can bypass the audio detector entirely.

## 13. Porting map and exactness checklist

| Source symbol | Suggested name |
| --- | --- |
| `g`, `y`, `k`, `M` | normalize_genre, ordered_genre_rules, non_music_rules, cue_genre_class |
| `_` | extract_sections |
| `v` | lowpass_one_pole_cascade |
| `F` | lowpass_biquad_fallback |
| `x` | nearest_section_index |
| `B` | drop_baseline |
| `w` | advance_to_sustained_arrival |
| `z` | find_earlier_stable_plateau |
| `j` | primary_bass_candidate |
| `U` | normalized_candidate_strength |
| `V` | skip_early_burst |
| `J` | first_completed_quiet_run |
| `K` | detect_start_and_first_drop |
| `vt` | detect_breakdown |
| `Ft` | scan_abrupt_breakdown_edge |
| `qt` | snap_to_drop_relative_phrase_grid |
| `Ot` | second_drop_time_fallback |
| `Ht` | detect_second_drop |
| `zt` | refine_second_drop |
| `jt` | make_second_drop_marker |
| `ae` | rms_around_time |
| `oe` | indexed_percentile |
| `le` | analyze_sections |

`le` contains inline functions for the drop-at-start classifier, terminal
breakdown fallback, last beat, and emergency loop. Lift these into named helpers
without changing their evaluation order. Symbols are scoped: short names reused
as local variables do not refer to the outer helpers/constants.

Build in this order:

1. Define the input/output types, numeric behavior, and a supplied-BPM interface.
2. Implement filtering and section features; compare arrays to the appendix.
3. Port `K` and its helpers, then `vt/Ft`, then `Ht/zt/qt/Ot/jt`.
4. Port orchestration, last beat, and loop selection.
5. Use the reference as an oracle on identical PCM and prefiltered PCM. Compare
   intermediate features, candidate indices, diagnostic tiers, and final anchors.
6. Add the cue-template layer only after anchor parity is established.
7. Validate on labeled real tracks in multiple genres and sample rates, including
   silence, partial sections, nonzero first beat, immediate drops, false starts,
   long breakdowns, and fading endings. Keep accuracy separate from source parity.

For a simpler independent implementation, start with bass-energy changes on a
beat grid and the broad priors, leaving out most corrections. That is a reasonable
product choice, but label it **inspired by this detector**, not behaviorally
compatible with it.

### Known quirks and deliberate limits

- Silence can produce Start and Drop at zero: zero-valued energy thresholds pass
  with `>=`, despite last-beat detection returning none. Confirmed by a synthetic
  worker probe; do not silently claim silence is rejected by the original.
- `never` is conditional and does not universally forbid a zero-time drop.
- Some refinement phases use 16 seconds, others 16 beats, others 16 sections.
  Porting all of them as one “phrase” constant changes the algorithm.
- The section endpoint can count a boundary beat twice across adjacent sections.
- Fallbacks can emit a second drop based on timing priors or a loop that failed
  its stability criteria. There is no calibrated confidence output.
- The app's Web Audio preprocessing, decoder, actual playback context sample
  rate, UI range helper, beatgrid estimator, and template storage require their
  own parity work. The executable appendix specifies the supplied-input detector.
- No real-track host test was performed. Synthetic parity proves the excerpt
  reproduces the worker on those inputs; it does not establish musical accuracy.

## 14. Handoff prompt

> Implement the cue-section detector described in this directory in the target
> project's language. Read README.md, then translate the complete detector from
> REFERENCE.md using the symbol map; treat the appendix as normative for exact
> branch ordering and boundary conditions. Take decoded channel-0 float32 PCM,
> sample rate, constant BPM, first-beat time, and optional prefiltered PCM as inputs.
> Keep Web Audio-equivalent preprocessing distinct from the local fallback. Do not
> add an ML dependency. First achieve parity with the provided reference using
> synthetic fixtures and identical inputs; report any intentional deviations.
> Then evaluate real-track accuracy separately. Do not claim full Lexicon parity
> without validating decoding, beatgrid, preprocessing, and template conversion.
