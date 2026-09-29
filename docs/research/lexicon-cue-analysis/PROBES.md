# Verification and reproduction

These probes compare the documented source excerpt against the complete original
worker using identical synthetic PCM and prefiltered PCM. They **do not** compare
an independently implemented detector, invoke the Lexicon UI, reproduce its Web
Audio preprocessing, or measure accuracy on music.

## Recorded result

On 2026-09-30, all **49 comparisons passed**:

- 4 signal shapes: silence, one-second audio, steady pulsed audio, and a synthetic
  arrangement with loud and quiet sections.
- 3 genre inputs: null, House, Hardcore.
- 2 drop-at-start policies: never and highEnergyOnly.
- 2 first-beat offsets: zero and 0.125 seconds.
- 48 combinations from that matrix, plus 1 absent-prefilter wrapper comparison.

Each comparison checks the entire returned marker objects, including types,
colors, names, durations, loop metadata, ordering, and timestamps. Both runs use
2 kHz PCM, 120 BPM, a 64-beat minimum breakdown, and (for the matrix) loop detection.
The 2 kHz sample rate is a cheap synthetic test setting, not a claim about the
app's normal decoder rate. The signals contain deterministic 60/180/620 Hz tones
and a beat-synchronized decaying envelope. Preprocessing in this test explicitly
uses the appendix's local biquad helper and passes that same result to both paths.
It does not verify renderer Web Audio equivalence.

Examples with no genre and firstBeat=0, identical for both policies:

| Signal | Detected anchors in seconds |
| --- | --- |
| Silence, 240 seconds | Start 0; Drop 0 |
| Short, 1 second | Start 0; Drop 0 |
| Steady, 240 seconds | Start 0; Drop 0; S.O.S. 216; Last beat 232 |
| Structured, 240 seconds | Start 0; Drop 16; Breakdown 80; Second drop 112; S.O.S. 160; Second breakdown 176; Last beat 176 |

These examples illustrate behavior, not musical ground truth. The structured
signal has amplitude boundaries at 16, 80, 112, 176, and 208 seconds. The fact that
Last beat is 176 rather than 240 is part of the observed output.

## Run the exact probe

Requires Node.js and Python 3 already available locally; no npm dependencies.
The harness reads the installed ASAR archive and verifies the worker SHA-256
before evaluating it. It does not launch Lexicon or access its library.
The appendix block is executable reference evidence, not a standalone app.

From the repository root:

```sh
python3 - <<'PYPROBE'
from pathlib import Path
text = Path('docs/research/lexicon-cue-analysis/PROBES.md').read_text()
code = text.split('```javascript\n', 1)[1].split('\n```', 1)[0]
Path('/tmp/lexicon-cue-probe.mjs').write_text(code)
PYPROBE
node /tmp/lexicon-cue-probe.mjs "$PWD/docs/research/lexicon-cue-analysis"
```

An optional second argument to the script is an alternate `app.asar` path.
A hash mismatch is a provenance failure, not permission to update the hash blindly.
The expected output begins with `"passed": 49`, followed by the example markers.

```javascript
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import path from 'node:path';
const docDir = process.argv[2];
if (!docDir) throw new Error('Pass the documentation directory as the first argument');
const md=fs.readFileSync(path.join(docDir,'REFERENCE.md'),'utf8');
const code=md.match(/```javascript\n([\s\S]*?)\n```/)[1];
const ref=vm.createContext({});
vm.runInContext(code+'\nglobalThis.api={le,F,v,sections:_,genre:M};',ref);
let posted;
const original=vm.createContext({console,self:{postMessage(value){posted=value;}}});
const asar = fs.readFileSync(process.argv[3] ?? '/Applications/Lexicon.app/Contents/Resources/app.asar');
const headerSize = asar.readUInt32LE(4);
const jsonSize = asar.readUInt32LE(12);
const header = JSON.parse(asar.subarray(16,16+jsonSize).toString('utf8'));
let entry = header;
for (const name of 'renderer/main_window/182.index.worker.js'.split('/')) entry=entry.files[name];
assert.ok(!entry.unpacked);
const offset=8+headerSize+Number(entry.offset);
const worker=asar.subarray(offset,offset+entry.size);
assert.equal(createHash('sha256').update(worker).digest('hex'),
  'd8bc3b60b1989094d00945f6b03b7a48a63490d390b76f34cb7158345203113e',
  'Installed worker changed: stop and review it before updating the expected hash');
vm.runInContext(worker.toString('utf8'),original);
const sr=2000,bpm=120;
function synth(kind,offset=0){
 const duration=kind==='short'?1:240;
 const pcm=new Float32Array(duration*sr);
 for(let i=0;i<pcm.length;i++){
  const t=i/sr,phase=((t-offset)%0.5+0.5)%0.5;
  let amp=kind==='silence'?0:kind==='steady'?0.7:t<16?0.1:t<80?0.8:t<112?0.06:t<176?0.9:t<208?0.08:0.35;
  if(kind==='short')amp=0.8;
  const pulse=Math.exp(-phase*22);
  pcm[i]=amp*(0.7*pulse*Math.sin(2*Math.PI*60*t)+0.2*Math.sin(2*Math.PI*180*t)+0.1*Math.sin(2*Math.PI*620*t));
 }
 return pcm;
}
const results=[];
for(const kind of ['silence','short','steady','sections'])for(const genre of [null,'House','Hardcore'])for(const dropAtStart of ['never','highEnergyOnly'])for(const firstBeat of [0,0.125]){
 const raw=synth(kind,firstBeat);
 // Explicitly synthetic alternate preprocessing, not Web Audio app equivalence.
 let filtered=raw;
 for(const cutoff of [800,400,400,200,200,200,200])filtered=ref.api.F(filtered,sr,cutoff);
 const settings={tempo:bpm,firstBeat,analyzeCuepoints:true,analyzeBeatgrid:false,analyzeEnergy:false,analyzeKey:false,breakdownMinBeats:64,addEmergencyLoopSectionPoint:true,dropAtStart,genre};
 posted=undefined;
 original.self.onmessage({data:[settings,sr,{send:filtered.buffer},{send:raw.buffer},null,null]});
 const actual=ref.api.le(raw,sr,bpm,firstBeat,64,true,filtered,{dropAtStart,genre});
 const expected=JSON.parse(JSON.stringify(posted.sectionPoints));
 assert.deepEqual(JSON.parse(JSON.stringify(actual)),expected);
 results.push({kind,genre,dropAtStart,firstBeat,points:expected.map(p=>[p.markerType,p.startTimeSeconds])});
}
// The absent-prefilter wrapper behavior is not the same as direct le(...,null).
const raw=synth('sections');
const settings={tempo:bpm,firstBeat:0,analyzeCuepoints:true,breakdownMinBeats:64,addEmergencyLoopSectionPoint:false,dropAtStart:'never',genre:null};
original.self.onmessage({data:[settings,sr,null,{send:raw.buffer},null,null]});
const emptyRef=ref.api.le(raw,sr,bpm,0,64,false,new Float32Array(),{dropAtStart:'never',genre:null});
assert.deepEqual(JSON.parse(JSON.stringify(emptyRef)),JSON.parse(JSON.stringify(posted.sectionPoints)));
console.log(JSON.stringify({passed:results.length+1,examples:results.filter(x=>x.genre===null&&x.firstBeat===0)},null,2));
```

## Follow-up validation for an independent port

The matrix is a smoke/differential check for extraction integrity. It does not
exercise every correction predicate. A port should additionally compare each
helper against this reference on generated section-energy patterns, especially:

- Threshold equality and just-above/just-below cases; array-end conditions;
  earliest-candidate ties and backward-loop ties.
- Different sample rates (including 44.1 kHz with the 5 ms rounding discrepancy),
  partial sections, large first-beat offsets, and beat/phrase phase changes.
- Every second-drop refinement stage, including cases where several apply.
- House/techno/hardcore genre corrections, ordered classifier precedence, and
  excluded non-music labels.
- Direct-call null prefilter versus worker-wrapper empty prefilter.
- No detected breakdown, no passing second-drop candidates, timing fallback,
  drop-at-start promotion/cancellation, and terminal-breakdown fallback.
- Emergency-loop best score versus the all-candidates-rejected fallback.

For end-to-end app parity, obtain identical decoded PCM, the renderer's actual
Web Audio prefilter output, BPM, and first-beat time. Comparing independently
decoded audio with a different resampler does not isolate detector differences.

For musical accuracy, use a separately labeled real-track corpus and report
anchor error in beats/time. The original worker's output is a compatibility
oracle, not ground truth for the best place to mix.
