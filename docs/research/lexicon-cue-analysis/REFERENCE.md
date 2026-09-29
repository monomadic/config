# Lexicon cue analysis: exact reference appendix

Read [the specification](README.md) first. This appendix preserves the complete
cue-section detector and its genre classifier from the locally installed bundle.
It is evidence for a port, not a new implementation or an upstream source release.

The JavaScript block is the contiguous formatted worker excerpt from `function g`
(genre normalization) through `function le` (section orchestration), inclusive.
Only the four enum bindings at the beginning were added to make it independently
executable. No detector expressions, rule ordering, genre patterns, or thresholds
were rewritten. The preserved excerpt (excluding adapter bindings), after formatting and common
indent removal and with no final newline, has SHA-256
`c5ac9181c5a6b6c44c6abe62aeb97f476e5cf02860fb32346f7808ab8fba29b8`.

The original minified identifiers are intentional: the specification
maps them to meaningful names. The first matching genre rule wins; the long table
therefore cannot be replaced by a simple substring search for exact parity.

This block has no package dependencies. Evaluating it defines `le`, the entry point:

`le(rawChannel0, sampleRate, bpm, firstBeatSeconds, breakdownMinBeats, emergencyLoop, prefilteredChannel0OrNull, { dropAtStart, genre })`

The null fallback and the normal app preprocessing differ; see the specification.
Keep `analyzeBeatgrid` outside this interface. The separate energy and key classifiers,
Webpack runtime, renderer, and unrelated library modules are deliberately excluded.

```javascript
// Adapter bindings: values copied from the worker enums; not detector logic.
const r = { Start: "start", Normal: "normal", Drop: "drop", SecondDrop: "second_drop", Breakdown: "breakdown", SecondBreakdown: "second_breakdown", Lastbeat: "lastbeat", EmergencyLoop: "emergency_loop" };
const a = { MemoryCue: "memory_cue", ActiveLoop: "loop_active" };
const m = { Red: "red", Blue: "blue", Orange: "orange", Magenta: "magenta" };
const s = { Never: "never", HighEnergyOnly: "highEnergyOnly" };
function g(t) {
  return t
    ? t
        .toLowerCase()
        .normalize("NFD")
        .replace(/[\u0300-\u036f]/g, "")
        .replace(/[-\u2010-\u2015\u2212\u2013\u2014/&+]/g, " ")
        .replace(/[^a-z0-9\s]/g, "")
        .replace(/\s+/g, " ")
        .trim()
    : "";
}
const y = [
    {
      pattern: /\bfrench house\b/,
      canonicalName: "French House",
      mainGenre: "House",
    },
    {
      pattern: /\bdeep house\b/,
      canonicalName: "Deep House",
      mainGenre: "House",
    },
    {
      pattern: /\btech house\b/,
      canonicalName: "Tech House",
      mainGenre: "House",
    },
    {
      pattern: /\bprogressive house\b/,
      canonicalName: "Progressive House",
      mainGenre: "House",
    },
    {
      pattern: /\bbass house\b/,
      canonicalName: "Bass House",
      mainGenre: "House",
    },
    {
      pattern: /\bfunky house\b/,
      canonicalName: "Funky House",
      mainGenre: "House",
    },
    {
      pattern: /\bjackin house\b/,
      canonicalName: "Jackin House",
      mainGenre: "House",
    },
    {
      pattern: /\borganic house\b/,
      canonicalName: "Organic House",
      mainGenre: "House",
    },
    {
      pattern: /\bmelodic house\b/,
      canonicalName: "Melodic House",
      mainGenre: "House",
    },
    {
      pattern: /\btribal house\b/,
      canonicalName: "Tribal House",
      mainGenre: "House",
    },
    {
      pattern: /\btropical house\b/,
      canonicalName: "Tropical House",
      mainGenre: "House",
    },
    {
      pattern: /\blatin house\b/,
      canonicalName: "Latin House",
      mainGenre: "House",
    },
    {
      pattern: /\bchicago house\b/,
      canonicalName: "Chicago House",
      mainGenre: "House",
    },
    {
      pattern: /\bacid house\b/,
      canonicalName: "Acid House",
      mainGenre: "House",
    },
    {
      pattern: /\bslap house\b/,
      canonicalName: "Slap House",
      mainGenre: "House",
    },
    {
      pattern: /\bfuture house\b/,
      canonicalName: "Future House",
      mainGenre: "House",
    },
    {
      pattern: /\bhard house\b/,
      canonicalName: "Hard House",
      mainGenre: "House",
    },
    {
      pattern: /\bstutter house\b/,
      canonicalName: "Stutter House",
      mainGenre: "House",
    },
    {
      pattern: /\blo ?fi house\b/,
      canonicalName: "Lo-Fi House",
      mainGenre: "House",
    },
    {
      pattern: /\belectro house\b/,
      canonicalName: "Electro House",
      mainGenre: "House",
    },
    {
      pattern: /\bjazz house\b/,
      canonicalName: "Jazz House",
      mainGenre: "House",
    },
    {
      pattern: /\bhip house\b/,
      canonicalName: "Hip House",
      mainGenre: "House",
    },
    {
      pattern: /\bgarage house\b/,
      canonicalName: "Garage House",
      mainGenre: "House",
    },
    {
      pattern: /\bghetto house\b/,
      canonicalName: "Ghetto House",
      mainGenre: "House",
    },
    {
      pattern: /\beuro house\b/,
      canonicalName: "Euro House",
      mainGenre: "House",
    },
    {
      pattern: /\bambient house\b/,
      canonicalName: "Ambient House",
      mainGenre: "House",
    },
    {
      pattern: /\bwitch house\b/,
      canonicalName: "Witch House",
      mainGenre: "Electronic",
    },
    {
      pattern: /\brally house\b/,
      canonicalName: "Rally House",
      mainGenre: "House",
    },
    {
      pattern: /\bafro\s*house\b/,
      canonicalName: "Afro House",
      mainGenre: "House",
    },
    {
      pattern: /\bmelbourne bounce\b/,
      canonicalName: "Melbourne Bounce",
      mainGenre: "House",
    },
    {
      pattern: /\bbig room\b/,
      canonicalName: "Big Room",
      mainGenre: "House",
    },
    {
      pattern: /\bmainstage\b/,
      canonicalName: "Mainstage",
      mainGenre: "House",
    },
    {
      pattern: /\bhouse\b/,
      canonicalName: "House",
      mainGenre: "House",
    },
    {
      pattern: /\bhard techno\b/,
      canonicalName: "Hard Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bmelodic techno\b/,
      canonicalName: "Melodic Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bminimal techno\b/,
      canonicalName: "Minimal Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bdub techno\b/,
      canonicalName: "Dub Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bacid techno\b/,
      canonicalName: "Acid Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bdeep tech\b/,
      canonicalName: "Deep Tech",
      mainGenre: "House",
    },
    {
      pattern: /\bhypertechno\b/,
      canonicalName: "Hypertechno",
      mainGenre: "Techno",
    },
    {
      pattern: /\btechno.*peak\b|peak.*techno\b/,
      canonicalName: "Peak Time Techno",
      mainGenre: "Techno",
    },
    {
      pattern:
        /\btechno.*raw\b|raw.*techno|techno.*hypnotic|hypnotic.*techno\b/,
      canonicalName: "Hypnotic Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bschranz\b/,
      canonicalName: "Schranz",
      mainGenre: "Techno",
    },
    {
      pattern: /\btekno\b/,
      canonicalName: "Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\btechno\b/,
      canonicalName: "Techno",
      mainGenre: "Techno",
    },
    {
      pattern: /\bpsy ?trance\b/,
      canonicalName: "Psytrance",
      mainGenre: "Trance",
    },
    {
      pattern: /\bprogressive trance\b/,
      canonicalName: "Progressive Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\bgoa trance\b/,
      canonicalName: "Goa Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\bhard trance\b/,
      canonicalName: "Hard Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\btech trance\b/,
      canonicalName: "Tech Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\btrance.*main\s*floor\b/,
      canonicalName: "Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\btrance.*hypnotic|hypnotic.*trance\b/,
      canonicalName: "Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\btrance\b/,
      canonicalName: "Trance",
      mainGenre: "Trance",
    },
    {
      pattern: /\bliquid funk\b/,
      canonicalName: "Liquid Funk",
      mainGenre: "Drum & Bass",
    },
    {
      pattern: /\bdrum\s*(?:n|and)?\s*bass\b|\bdrumstep\b|\bdnb\b/,
      canonicalName: "Drum & Bass",
      mainGenre: "Drum & Bass",
    },
    {
      pattern: /\bdeep dubstep\b/,
      canonicalName: "Deep Dubstep",
      mainGenre: "Dubstep",
    },
    {
      pattern: /\b140\b/,
      canonicalName: "Deep Dubstep",
      mainGenre: "Dubstep",
    },
    {
      pattern: /\bdeathstep\b/,
      canonicalName: "Deathstep",
      mainGenre: "Dubstep",
    },
    {
      pattern: /\briddi?m\b/,
      canonicalName: "Riddim",
      mainGenre: "Dubstep",
    },
    {
      pattern: /\bdubstep\b/,
      canonicalName: "Dubstep",
      mainGenre: "Dubstep",
    },
    {
      pattern: /\bbreakcore\b/,
      canonicalName: "Breakcore",
      mainGenre: "Breakbeat",
    },
    {
      pattern: /\bbreakbeat\b|\bbreaks\b/,
      canonicalName: "Breakbeat",
      mainGenre: "Breakbeat",
    },
    {
      pattern: /\buk bass\b|\bbassline\b/,
      canonicalName: "Bassline",
      mainGenre: "Garage",
    },
    {
      pattern: /\buk garage\b/,
      canonicalName: "UK Garage",
      mainGenre: "Garage",
    },
    {
      pattern: /\buk funky\b/,
      canonicalName: "UK Funky",
      mainGenre: "Garage",
    },
    {
      pattern: /\bspeed garage\b/,
      canonicalName: "Speed Garage",
      mainGenre: "Garage",
    },
    {
      pattern: /\bgarage rock\b|\bgarage punk\b/,
      canonicalName: "Garage Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bgarage\b/,
      canonicalName: "Garage",
      mainGenre: "Garage",
    },
    {
      pattern: /\bjersey club\b/,
      canonicalName: "Jersey Club",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bbaltimore club\b/,
      canonicalName: "Baltimore Club",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bfootwork\b/,
      canonicalName: "Footwork",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bjungle\b/,
      canonicalName: "Jungle",
      mainGenre: "Drum & Bass",
    },
    {
      pattern: /\bhardstyle\b/,
      canonicalName: "Hardstyle",
      mainGenre: "Hardstyle",
    },
    {
      pattern: /\bhappy hardcore\b/,
      canonicalName: "Happy Hardcore",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bfrenchcore\b/,
      canonicalName: "Frenchcore",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bgabber\b/,
      canonicalName: "Gabber",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bhardcore techno\b/,
      canonicalName: "Hardcore Techno",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bhardcore punk\b/,
      canonicalName: "Hardcore Punk",
      mainGenre: "Rock",
    },
    {
      pattern: /\bmelodic\s*hardcore\b/,
      canonicalName: "Melodic Hardcore",
      mainGenre: "Metal",
    },
    {
      pattern: /\bpost\s*hardcore\b/,
      canonicalName: "Post-Hardcore",
      mainGenre: "Rock",
    },
    {
      pattern: /\bhardcore hip\b/,
      canonicalName: "Hardcore Hip Hop",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bhardcore\b/,
      canonicalName: "Hardcore",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bnoise\b/,
      canonicalName: "Noise",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bchiptune\b/,
      canonicalName: "Chiptune",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bspeedcore\b/,
      canonicalName: "Speedcore",
      mainGenre: "Hardcore",
    },
    {
      pattern: /\bmusique concr[eè]te\b/,
      canonicalName: "Musique Concrete",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bneo rave\b/,
      canonicalName: "Neo Rave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdark ambient\b/,
      canonicalName: "Dark Ambient",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bambient\b/,
      canonicalName: "Ambient",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdowntempo\b|\bchill ?out\b/,
      canonicalName: "Downtempo",
      mainGenre: "Electronic",
    },
    {
      pattern: /\btrip hop\b/,
      canonicalName: "Trip Hop",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bidm\b/,
      canonicalName: "IDM",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bglitch\b/,
      canonicalName: "Glitch",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdrone\b/,
      canonicalName: "Drone",
      mainGenre: "Electronic",
    },
    {
      pattern: /\belectroclash\b/,
      canonicalName: "Electroclash",
      mainGenre: "Electronic",
    },
    {
      pattern: /\belectro swing\b/,
      canonicalName: "Electro Swing",
      mainGenre: "Electronic",
    },
    {
      pattern: /\belectro\b/,
      canonicalName: "Electro",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bsynthwave\b/,
      canonicalName: "Synthwave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bvaporwave\b/,
      canonicalName: "Vaporwave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bchillwave\b/,
      canonicalName: "Chillwave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bedm trap\b/,
      canonicalName: "EDM Trap",
      mainGenre: "Electronic",
    },
    {
      pattern: /\btrap\s*(?:latino|funk|soul)\b/,
      canonicalName: "Trap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\btrap\b/,
      canonicalName: "Trap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bfuture bass\b/,
      canonicalName: "Future Bass",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bmelodic bass\b/,
      canonicalName: "Melodic Bass",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bbass music\b/,
      canonicalName: "Bass Music",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bmoombahton\b/,
      canonicalName: "Moombahton",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bphonk\b/,
      canonicalName: "Phonk",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bhyperpop\b/,
      canonicalName: "Hyperpop",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bnightcore\b/,
      canonicalName: "Nightcore",
      mainGenre: "Electronic",
    },
    {
      pattern: /\blo ?fi\b.*\bbeat/,
      canonicalName: "Lo-Fi Beats",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bminimal\b/,
      canonicalName: "Minimal",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bindie dance\b/,
      canonicalName: "Indie Dance",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bnew beat\b/,
      canonicalName: "New Beat",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bbalearic\b/,
      canonicalName: "Balearic",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bleftfield\b/,
      canonicalName: "Leftfield",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdeconstructed club\b/,
      canonicalName: "Deconstructed Club",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bmicrohouse\b/,
      canonicalName: "Microhouse",
      mainGenre: "House",
    },
    {
      pattern: /\bbroken beat\b/,
      canonicalName: "Broken Beat",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdonk\b/,
      canonicalName: "Donk",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bhands up\b/,
      canonicalName: "Hands Up",
      mainGenre: "Electronic",
    },
    {
      pattern: /\blento violento\b/,
      canonicalName: "Lento Violento",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bebm\b/,
      canonicalName: "EBM",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bcedm\b|\bedm\b/,
      canonicalName: "EDM",
      mainGenre: "Dance",
    },
    {
      pattern: /\belectronic\s*rock\b/,
      canonicalName: "Electronic Rock",
      mainGenre: "Electronic",
    },
    {
      pattern: /\belectronica\b/,
      canonicalName: "Electronica",
      mainGenre: "Electronic",
    },
    {
      pattern: /\b3\s*step\b/,
      canonicalName: "3 Step",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdarkwave\b|\bcoldwave\b/,
      canonicalName: "Darkwave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\blo\s*fi\b/,
      canonicalName: "Lo-Fi",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bnew\s*age\b/,
      canonicalName: "New Age",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bbig\s*beat\b/,
      canonicalName: "Big Beat",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bfreestyle\b/,
      canonicalName: "Freestyle",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bmiami bass\b/,
      canonicalName: "Miami Bass",
      mainGenre: "Electronic",
    },
    {
      pattern: /\blo ?fi\b/,
      canonicalName: "Lo-Fi",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bexperimental\b/,
      canonicalName: "Experimental",
      mainGenre: "Electronic",
    },
    {
      pattern: /\belectronic\b/,
      canonicalName: "Electronic",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdeath metal\b/,
      canonicalName: "Death Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bblack metal\b/,
      canonicalName: "Black Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bthrash\s*metal\b|\bthrash\b/,
      canonicalName: "Thrash Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bdoom metal\b/,
      canonicalName: "Doom Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bpower metal\b/,
      canonicalName: "Power Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bprogressive metal\b/,
      canonicalName: "Progressive Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bsymphonic metal\b/,
      canonicalName: "Symphonic Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bgothic metal\b/,
      canonicalName: "Gothic Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bgroove metal\b/,
      canonicalName: "Groove Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bsludge metal\b/,
      canonicalName: "Sludge Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bspeed metal\b/,
      canonicalName: "Speed Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bfolk metal\b/,
      canonicalName: "Folk Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bpirate metal\b/,
      canonicalName: "Pirate Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bindustrial metal\b/,
      canonicalName: "Industrial Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\balternative metal\b/,
      canonicalName: "Alternative Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bglam metal\b/,
      canonicalName: "Glam Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bnu metal\b/,
      canonicalName: "Nu Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bmetalcore\b/,
      canonicalName: "Metalcore",
      mainGenre: "Metal",
    },
    {
      pattern: /\bdeathcore\b/,
      canonicalName: "Deathcore",
      mainGenre: "Metal",
    },
    {
      pattern: /\bmelodic\s*(death\s*)?metal\b/,
      canonicalName: "Melodic Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bdjent\b/,
      canonicalName: "Djent",
      mainGenre: "Metal",
    },
    {
      pattern: /\bneo\s*classical\s*metal\b/,
      canonicalName: "Neoclassical Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bgrindcore\b|\bgoregrind\b/,
      canonicalName: "Grindcore",
      mainGenre: "Metal",
    },
    {
      pattern: /\bheavy metal\b/,
      canonicalName: "Heavy Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bmetal\b/,
      canonicalName: "Metal",
      mainGenre: "Metal",
    },
    {
      pattern: /\bpsychobilly\b/,
      canonicalName: "Psychobilly",
      mainGenre: "Rock",
    },
    {
      pattern: /\bdeathrock\b/,
      canonicalName: "Deathrock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bhorror\s*(?:core|rock|punk)\b/,
      canonicalName: "Horrorcore",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpsychedelic rock\b|\bpsychedelia\b/,
      canonicalName: "Psychedelic Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\balternative rock\b|\balt rock\b/,
      canonicalName: "Alternative Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bindie rock\b/,
      canonicalName: "Indie Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpunk rock\b|\bpop punk\b/,
      canonicalName: "Pop Punk",
      mainGenre: "Rock",
    },
    {
      pattern: /\bskate punk\b/,
      canonicalName: "Skate Punk",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpost\s*punk\b/,
      canonicalName: "Post-Punk",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpost\s*rock\b/,
      canonicalName: "Post-Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpost\s*grunge\b/,
      canonicalName: "Post-Grunge",
      mainGenre: "Rock",
    },
    {
      pattern: /\bprogressive rock\b|\bprog rock\b/,
      canonicalName: "Progressive Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bclassic rock\b|\balbum rock\b/,
      canonicalName: "Classic Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bhard rock\b/,
      canonicalName: "Hard Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bsoft rock\b/,
      canonicalName: "Soft Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bblues rock\b/,
      canonicalName: "Blues Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bcountry rock\b/,
      canonicalName: "Country Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bsouthern rock\b/,
      canonicalName: "Southern Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bfolk rock\b/,
      canonicalName: "Folk Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bsymphonic rock\b/,
      canonicalName: "Symphonic Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bsurf rock\b|\bsurf\b/,
      canonicalName: "Surf Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bstoner rock\b/,
      canonicalName: "Stoner Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bspace rock\b/,
      canonicalName: "Space Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bmath rock\b/,
      canonicalName: "Math Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bnoise rock\b/,
      canonicalName: "Noise Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bshoegaze\b/,
      canonicalName: "Shoegaze",
      mainGenre: "Rock",
    },
    {
      pattern: /\bgrunge\b/,
      canonicalName: "Grunge",
      mainGenre: "Rock",
    },
    {
      pattern: /\bemo\b(?!\s*rap)/,
      canonicalName: "Emo",
      mainGenre: "Rock",
    },
    {
      pattern: /\bscreamo\b/,
      canonicalName: "Screamo",
      mainGenre: "Rock",
    },
    {
      pattern: /\bbritpop\b/,
      canonicalName: "Britpop",
      mainGenre: "Rock",
    },
    {
      pattern: /\bmadchester\b/,
      canonicalName: "Madchester",
      mainGenre: "Rock",
    },
    {
      pattern: /\bnew wave\b|\bneue deutsche welle\b/,
      canonicalName: "New Wave",
      mainGenre: "Rock",
    },
    {
      pattern: /\bglam rock\b|\bglam\b/,
      canonicalName: "Glam Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bkrautrock\b/,
      canonicalName: "Krautrock",
      mainGenre: "Rock",
    },
    {
      pattern: /\brockabilly\b/,
      canonicalName: "Rockabilly",
      mainGenre: "Rock",
    },
    {
      pattern:
        /\brock\s*and\s*roll\b|\brock\s*n\s*roll\b|\brock roll\b/,
      canonicalName: "Rock & Roll",
      mainGenre: "Rock",
    },
    { pattern: /\bpunk\b/, canonicalName: "Punk", mainGenre: "Rock" },
    {
      pattern: /\bindustrial rock\b/,
      canonicalName: "Industrial Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\bindustrial\b/,
      canonicalName: "Industrial",
      mainGenre: "Rock",
    },
    {
      pattern: /\bpop rock\b/,
      canonicalName: "Pop Rock",
      mainGenre: "Rock",
    },
    {
      pattern: /\brap rock\b/,
      canonicalName: "Rap Rock",
      mainGenre: "Rock",
    },
    { pattern: /\baor\b/, canonicalName: "AOR", mainGenre: "Rock" },
    { pattern: /\brock\b/, canonicalName: "Rock", mainGenre: "Rock" },
    {
      pattern: /\buk drill\b/,
      canonicalName: "UK Drill",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bsexy drill\b/,
      canonicalName: "Drill",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bdrill\b/,
      canonicalName: "Drill",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bgrime\b/,
      canonicalName: "Grime",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bboom bap\b/,
      canonicalName: "Boom Bap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bcloud rap\b/,
      canonicalName: "Cloud Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bemo rap\b/,
      canonicalName: "Emo Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bjazz rap\b|\bjazzy hip\b/,
      canonicalName: "Jazz Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bmelodic rap\b/,
      canonicalName: "Melodic Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\brage rap\b/,
      canonicalName: "Rage Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bmeme rap\b/,
      canonicalName: "Meme Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bgangsta\b|\bgangster rap\b/,
      canonicalName: "Gangsta Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bg\s*funk\b/,
      canonicalName: "G-Funk",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bcrunk\b/,
      canonicalName: "Crunk",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bhyphy\b/,
      canonicalName: "Hyphy",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bsouthern hip\b|\bdirty south\b/,
      canonicalName: "Southern Hip Hop",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\beast coast hip\b/,
      canonicalName: "East Coast Hip Hop",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bwest coast hip\b/,
      canonicalName: "West Coast Hip Hop",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bunderground hip\b/,
      canonicalName: "Underground Hip Hop",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bnerdcore\b/,
      canonicalName: "Nerdcore",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bbounce\b/,
      canonicalName: "Bounce",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bhip\s*hop\b|\brap\b/,
      canonicalName: "Hip-Hop/Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    {
      pattern: /\bneo\s*soul\b/,
      canonicalName: "Neo Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bsmooth\s*soul\b/,
      canonicalName: "Smooth Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bnorthern soul\b/,
      canonicalName: "Northern Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bphilly soul\b/,
      canonicalName: "Philly Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bretro soul\b/,
      canonicalName: "Retro Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bblue\s*eyed\s*soul\b/,
      canonicalName: "Blue-Eyed Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bpop\s*soul\b/,
      canonicalName: "Pop Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bsoul\s*blues\b/,
      canonicalName: "Soul Blues",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bsoul\s*jazz\b/,
      canonicalName: "Soul Jazz",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bsoul\b/,
      canonicalName: "Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bnew jack swing\b|\bswingbeat\b/,
      canonicalName: "New Jack Swing",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bquiet storm\b/,
      canonicalName: "Quiet Storm",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bmotown\b/,
      canonicalName: "Motown",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bfunk\b/,
      canonicalName: "Funk",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bgospel\b/,
      canonicalName: "Gospel",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bnu disco\b/,
      canonicalName: "Nu Disco",
      mainGenre: "Dance",
    },
    {
      pattern: /\bitalo\s*disco\b/,
      canonicalName: "Italo Disco",
      mainGenre: "Dance",
    },
    {
      pattern: /\beuro\s*disco\b/,
      canonicalName: "Euro Disco",
      mainGenre: "Dance",
    },
    {
      pattern: /\bdisco house\b/,
      canonicalName: "Disco House",
      mainGenre: "House",
    },
    {
      pattern: /\bpost\s*disco\b/,
      canonicalName: "Post-Disco",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bdisco\s*polo\b/,
      canonicalName: "Disco Polo",
      mainGenre: "Dance",
    },
    {
      pattern: /\bdisco\b/,
      canonicalName: "Disco",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bgo\s*go\b/,
      canonicalName: "Go-Go",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bsmooth r\s*n?\s*b\b/,
      canonicalName: "Smooth R&B",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\balternative r\s*n?\s*b\b|\bindie r\s*n?\s*b\b/,
      canonicalName: "Alternative R&B",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bcontemporary r\s*n?\s*b\b/,
      canonicalName: "Contemporary R&B",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\br\s*n?\s*b\b|\brhythm\s+blues\b/,
      canonicalName: "R&B/Soul/Funk",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bacid jazz\b/,
      canonicalName: "Acid Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bcool jazz\b/,
      canonicalName: "Cool Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bfree jazz\b/,
      canonicalName: "Free Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bsmooth jazz\b/,
      canonicalName: "Smooth Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bjazz fusion\b|\bfusion\b/,
      canonicalName: "Jazz Fusion",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bjazz funk\b/,
      canonicalName: "Jazz Funk",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bjazz rock\b/,
      canonicalName: "Jazz Rock",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bvocal jazz\b/,
      canonicalName: "Vocal Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bbebop\b|\bbop\b/,
      canonicalName: "Bebop",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bhard bop\b/,
      canonicalName: "Hard Bop",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bmodal jazz\b/,
      canonicalName: "Modal Jazz",
      mainGenre: "Jazz",
    },
    { pattern: /\bswing\b/, canonicalName: "Swing", mainGenre: "Jazz" },
    {
      pattern: /\bbig band\b/,
      canonicalName: "Big Band",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bgypsy jazz\b/,
      canonicalName: "Gypsy Jazz",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bnu jazz\b|\bfuture jazz\b/,
      canonicalName: "Nu Jazz",
      mainGenre: "Jazz",
    },
    { pattern: /\bjazz\b/, canonicalName: "Jazz", mainGenre: "Jazz" },
    {
      pattern: /\bdelta blues\b/,
      canonicalName: "Delta Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bchicago blues\b/,
      canonicalName: "Chicago Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\belectric blues\b/,
      canonicalName: "Electric Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bcountry blues\b/,
      canonicalName: "Country Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bclassic blues\b/,
      canonicalName: "Classic Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bpiano blues\b/,
      canonicalName: "Piano Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bblues\b/,
      canonicalName: "Blues",
      mainGenre: "Blues",
    },
    {
      pattern: /\bbaroque\b/,
      canonicalName: "Baroque",
      mainGenre: "Classical",
    },
    {
      pattern: /\bopera\b/,
      canonicalName: "Opera",
      mainGenre: "Classical",
    },
    {
      pattern: /\bchamber\s*(?:music|pop)\b/,
      canonicalName: "Chamber Music",
      mainGenre: "Classical",
    },
    {
      pattern: /\borchestra\b/,
      canonicalName: "Orchestral",
      mainGenre: "Classical",
    },
    {
      pattern: /\bchoral\b|\bgregorian\b/,
      canonicalName: "Choral",
      mainGenre: "Classical",
    },
    {
      pattern: /\bneoclassical\b|\bneo\s*classical\b/,
      canonicalName: "Neoclassical",
      mainGenre: "Classical",
    },
    {
      pattern: /\bminimalis[mt]\b/,
      canonicalName: "Minimalism",
      mainGenre: "Classical",
    },
    {
      pattern: /\bclassical\s*piano\b/,
      canonicalName: "Classical Piano",
      mainGenre: "Classical",
    },
    {
      pattern: /\bconcerto\b/,
      canonicalName: "Concerto",
      mainGenre: "Classical",
    },
    {
      pattern: /\bsymphon\b/,
      canonicalName: "Orchestral",
      mainGenre: "Classical",
    },
    {
      pattern: /\bclassical\b/,
      canonicalName: "Classical",
      mainGenre: "Classical",
    },
    {
      pattern: /\bbluegrass\b|\bnewgrass\b/,
      canonicalName: "Bluegrass",
      mainGenre: "Country",
    },
    {
      pattern: /\bhonky tonk\b/,
      canonicalName: "Honky Tonk",
      mainGenre: "Country",
    },
    {
      pattern: /\bamericana\b/,
      canonicalName: "Americana",
      mainGenre: "Country",
    },
    {
      pattern: /\btexas country\b/,
      canonicalName: "Texas Country",
      mainGenre: "Country",
    },
    {
      pattern: /\btraditional country\b/,
      canonicalName: "Traditional Country",
      mainGenre: "Country",
    },
    {
      pattern: /\bcountry\b/,
      canonicalName: "Country",
      mainGenre: "Country",
    },
    {
      pattern: /\bindie folk\b/,
      canonicalName: "Indie Folk",
      mainGenre: "Folk",
    },
    {
      pattern: /\bceltic\b/,
      canonicalName: "Celtic",
      mainGenre: "Folk",
    },
    {
      pattern: /\bneofolk\b/,
      canonicalName: "Neofolk",
      mainGenre: "Folk",
    },
    {
      pattern: /\bsinger\s*songwriter\b/,
      canonicalName: "Singer-Songwriter",
      mainGenre: "Pop",
    },
    {
      pattern: /\bchanson\b/,
      canonicalName: "Chanson",
      mainGenre: "Folk",
    },
    { pattern: /\bfolk\b/, canonicalName: "Folk", mainGenre: "Folk" },
    {
      pattern: /\breggaeton\b/,
      canonicalName: "Reggaeton",
      mainGenre: "Latin",
    },
    {
      pattern: /\bdembow\b/,
      canonicalName: "Dembow",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcumbia\b/,
      canonicalName: "Cumbia",
      mainGenre: "Latin",
    },
    {
      pattern: /\bsalsa\b/,
      canonicalName: "Salsa",
      mainGenre: "Latin",
    },
    {
      pattern: /\bbachata\b/,
      canonicalName: "Bachata",
      mainGenre: "Latin",
    },
    {
      pattern: /\bbossa nova\b/,
      canonicalName: "Bossa Nova",
      mainGenre: "Latin",
    },
    {
      pattern: /\bsamba\b/,
      canonicalName: "Samba",
      mainGenre: "Latin",
    },
    {
      pattern: /\btango\b/,
      canonicalName: "Tango",
      mainGenre: "Latin",
    },
    {
      pattern: /\bmerengue\b/,
      canonicalName: "Merengue",
      mainGenre: "Latin",
    },
    {
      pattern: /\bmambo\b/,
      canonicalName: "Mambo",
      mainGenre: "Latin",
    },
    {
      pattern: /\bbolero\b/,
      canonicalName: "Bolero",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcorridos?\b|\bsierren?o\b/,
      canonicalName: "Corrido",
      mainGenre: "Latin",
    },
    {
      pattern: /\branchera\b/,
      canonicalName: "Ranchera",
      mainGenre: "Latin",
    },
    {
      pattern: /\bmariachi\b/,
      canonicalName: "Mariachi",
      mainGenre: "Latin",
    },
    {
      pattern: /\bnorten?o\b/,
      canonicalName: "Norteno",
      mainGenre: "Latin",
    },
    {
      pattern: /\btejano\b/,
      canonicalName: "Tejano",
      mainGenre: "Latin",
    },
    {
      pattern: /\bbanda\b/,
      canonicalName: "Banda",
      mainGenre: "Latin",
    },
    {
      pattern: /\bgrupera\b/,
      canonicalName: "Grupera",
      mainGenre: "Latin",
    },
    {
      pattern: /\bguaracha\b/,
      canonicalName: "Guaracha",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcha\s*cha\b/,
      canonicalName: "Cha-Cha",
      mainGenre: "Latin",
    },
    {
      pattern: /\brumba\b/,
      canonicalName: "Rumba",
      mainGenre: "Latin",
    },
    {
      pattern: /\bvallenato\b/,
      canonicalName: "Vallenato",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcuarteto\b/,
      canonicalName: "Cuarteto",
      mainGenre: "Latin",
    },
    {
      pattern: /\btimba\b/,
      canonicalName: "Timba",
      mainGenre: "Latin",
    },
    { pattern: /\bmpb\b/, canonicalName: "MPB", mainGenre: "Latin" },
    {
      pattern: /\bpagode\b/,
      canonicalName: "Pagode",
      mainGenre: "Latin",
    },
    {
      pattern: /\bsertanejo\b/,
      canonicalName: "Sertanejo",
      mainGenre: "Latin",
    },
    {
      pattern: /\bforro\b/,
      canonicalName: "Forro",
      mainGenre: "Latin",
    },
    { pattern: /\baxe\b/, canonicalName: "Axe", mainGenre: "Latin" },
    {
      pattern: /\bbrega\b/,
      canonicalName: "Brega",
      mainGenre: "Latin",
    },
    {
      pattern: /\bpiseiro\b/,
      canonicalName: "Piseiro",
      mainGenre: "Latin",
    },
    {
      pattern: /\bbrazilian\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\blatin\s*pop\b/,
      canonicalName: "Latin Pop",
      mainGenre: "Latin",
    },
    {
      pattern: /\blatin\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\bmusica mexicana\b/,
      canonicalName: "Musica Mexicana",
      mainGenre: "Latin",
    },
    {
      pattern: /\bmexican\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcolombian\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\bargentine\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\bchilean\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\burbano\b/,
      canonicalName: "Latin",
      mainGenre: "Latin",
    },
    {
      pattern: /\bneoperreo\b/,
      canonicalName: "Neoperreo",
      mainGenre: "Latin",
    },
    { pattern: /\brkt\b/, canonicalName: "RKT", mainGenre: "Latin" },
    {
      pattern: /\bturreo\b/,
      canonicalName: "Turreo",
      mainGenre: "Latin",
    },
    {
      pattern: /\bdancehall\b/,
      canonicalName: "Dancehall",
      mainGenre: "Reggae",
    },
    {
      pattern: /\bdub\b(?!\s*(?:techno|step))/,
      canonicalName: "Dub",
      mainGenre: "Reggae",
    },
    { pattern: /\bska\b/, canonicalName: "Ska", mainGenre: "Reggae" },
    {
      pattern: /\broots reggae\b/,
      canonicalName: "Roots Reggae",
      mainGenre: "Reggae",
    },
    {
      pattern: /\brocksteady\b/,
      canonicalName: "Rocksteady",
      mainGenre: "Reggae",
    },
    {
      pattern: /\blovers rock\b/,
      canonicalName: "Lovers Rock",
      mainGenre: "Reggae",
    },
    {
      pattern: /\bragga\b/,
      canonicalName: "Ragga",
      mainGenre: "Reggae",
    },
    {
      pattern: /\breggae\b/,
      canonicalName: "Reggae",
      mainGenre: "Reggae",
    },
    { pattern: /\bzouk\b/, canonicalName: "Zouk", mainGenre: "World" },
    {
      pattern: /\beurodance\b/,
      canonicalName: "Eurodance",
      mainGenre: "Dance",
    },
    {
      pattern: /\bitalo\s*dance\b/,
      canonicalName: "Italodance",
      mainGenre: "Dance",
    },
    {
      pattern: /\bhi\s*nrg\b/,
      canonicalName: "Hi-NRG",
      mainGenre: "Dance",
    },
    {
      pattern: /\beuro\s*(?:beat|pop)\b/,
      canonicalName: "Europop",
      mainGenre: "Dance",
    },
    {
      pattern: /\bdance\s*pop\b/,
      canonicalName: "Dance Pop",
      mainGenre: "Dance",
    },
    {
      pattern: /\balternative dance\b/,
      canonicalName: "Alternative Dance",
      mainGenre: "Dance",
    },
    {
      pattern: /\bdance\b/,
      canonicalName: "Dance",
      mainGenre: "Dance",
    },
    {
      pattern: /\bindie\s*pop\b|\bindie\b/,
      canonicalName: "Indie Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bsynth\s*pop\b/,
      canonicalName: "Synth-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bart pop\b/,
      canonicalName: "Art Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bdream pop\b/,
      canonicalName: "Dream Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bk\s*pop\b/,
      canonicalName: "K-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bj\s*pop\b/,
      canonicalName: "J-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bt\s*pop\b/,
      canonicalName: "T-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bc\s*pop\b|\bmandopop\b/,
      canonicalName: "C-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bbedroom pop\b/,
      canonicalName: "Bedroom Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bacoustic pop\b/,
      canonicalName: "Acoustic Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bbaroque pop\b/,
      canonicalName: "Baroque Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bjangle pop\b/,
      canonicalName: "Jangle Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bpower pop\b/,
      canonicalName: "Power Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bsoft pop\b/,
      canonicalName: "Soft Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bcity pop\b/,
      canonicalName: "City Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bballad\b/,
      canonicalName: "Ballad",
      mainGenre: "Pop",
    },
    {
      pattern: /\beasy listening\b|\blounge\b|\bexotica\b/,
      canonicalName: "Easy Listening",
      mainGenre: "Pop",
    },
    {
      pattern: /\badult\s*(?:standards|contemporary)\b/,
      canonicalName: "Adult Contemporary",
      mainGenre: "Pop",
    },
    {
      pattern: /\bnederpop\b/,
      canonicalName: "Nederpop",
      mainGenre: "Pop",
    },
    { pattern: /\bpop\b/, canonicalName: "Pop", mainGenre: "Pop" },
    {
      pattern: /\bafrobeat\b/,
      canonicalName: "Afrobeat",
      mainGenre: "World",
    },
    {
      pattern: /\bafrobeats\b|\bafropop\b|\bafroswing\b|\bafropiano\b/,
      canonicalName: "Afrobeats",
      mainGenre: "World",
    },
    {
      pattern: /\bamapiano\b/,
      canonicalName: "Amapiano",
      mainGenre: "House",
    },
    { pattern: /\bgqom\b/, canonicalName: "Gqom", mainGenre: "World" },
    {
      pattern: /\bhighlife\b|\bhiplife\b/,
      canonicalName: "Highlife",
      mainGenre: "World",
    },
    {
      pattern: /\bflamenco\b/,
      canonicalName: "Flamenco",
      mainGenre: "World",
    },
    {
      pattern: /\bkizomba\b/,
      canonicalName: "Kizomba",
      mainGenre: "World",
    },
    {
      pattern: /\bcalypso\b/,
      canonicalName: "Calypso",
      mainGenre: "World",
    },
    { pattern: /\bsoca\b/, canonicalName: "Soca", mainGenre: "World" },
    {
      pattern: /\bkuduro\b/,
      canonicalName: "Kuduro",
      mainGenre: "World",
    },
    {
      pattern: /\bbhangra\b/,
      canonicalName: "Bhangra",
      mainGenre: "World",
    },
    {
      pattern:
        /\bbollywood\b|\bkollywood\b|\btollywood\b|\bmollywood\b/,
      canonicalName: "Bollywood",
      mainGenre: "World",
    },
    {
      pattern: /\bqawwali\b/,
      canonicalName: "Qawwali",
      mainGenre: "World",
    },
    {
      pattern: /\bpolka\b/,
      canonicalName: "Polka",
      mainGenre: "World",
    },
    {
      pattern: /\bschlager\b/,
      canonicalName: "Schlager",
      mainGenre: "World",
    },
    { pattern: /\brai\b/, canonicalName: "Rai", mainGenre: "World" },
    {
      pattern: /\bgnawa\b/,
      canonicalName: "Gnawa",
      mainGenre: "World",
    },
    { pattern: /\bfado\b/, canonicalName: "Fado", mainGenre: "World" },
    {
      pattern: /\bchampeta\b/,
      canonicalName: "Champeta",
      mainGenre: "World",
    },
    {
      pattern: /\bshatta\b/,
      canonicalName: "Shatta",
      mainGenre: "World",
    },
    {
      pattern: /\barabesk\b/,
      canonicalName: "Arabesk",
      mainGenre: "World",
    },
    {
      pattern: /\bkhaleeji\b/,
      canonicalName: "Khaleeji",
      mainGenre: "World",
    },
    {
      pattern: /\bcaribbean\b/,
      canonicalName: "Caribbean",
      mainGenre: "World",
    },
    {
      pattern: /\banime\b|\bvocaloid\b/,
      canonicalName: "Anime",
      mainGenre: "World",
    },
    {
      pattern: /\bk\s*(?:ballad|rock|rap)\b/,
      canonicalName: "K-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\bj\s*(?:dance|rock|rap)\b/,
      canonicalName: "J-Pop",
      mainGenre: "Pop",
    },
    {
      pattern: /\balt[eé]\b/,
      canonicalName: "Alte",
      mainGenre: "World",
    },
    {
      pattern: /\bgengetone\b/,
      canonicalName: "Gengetone",
      mainGenre: "World",
    },
    {
      pattern: /\bazonto\b/,
      canonicalName: "Azonto",
      mainGenre: "World",
    },
    { pattern: /\bdesi\b/, canonicalName: "Desi", mainGenre: "World" },
    {
      pattern: /\bprivate school piano\b/,
      canonicalName: "Private School Piano",
      mainGenre: "World",
    },
    {
      pattern:
        /\bharyanvi\b|\bhindi\b|\btamil\b|\btelugu\b|\bmalayalam\b|\bkannada\b|\bgujarati\b|\bmarathi\b|\bpunjabi\b|\bindian\b/,
      canonicalName: "Indian",
      mainGenre: "World",
    },
    {
      pattern: /\bturkish\b/,
      canonicalName: "Turkish",
      mainGenre: "World",
    },
    {
      pattern: /\bafrican\b/,
      canonicalName: "African",
      mainGenre: "World",
    },
    {
      pattern: /\bmoroccan\b|\begyptian\b|\barabic\b|\bmahraganat\b/,
      canonicalName: "Arabic",
      mainGenre: "World",
    },
    {
      pattern: /\bchinese\b|\bpinoy\b|\bkorean\b|\bjapanese\b/,
      canonicalName: "Asian",
      mainGenre: "World",
    },
    {
      pattern: /\bworld\b/,
      canonicalName: "World",
      mainGenre: "World",
    },
    {
      pattern: /\bfilm\s*score\b/,
      canonicalName: "Film Score",
      mainGenre: "Soundtrack",
    },
    {
      pattern: /\bsoundtrack\b|\bscore\b/,
      canonicalName: "Soundtrack",
      mainGenre: "Soundtrack",
    },
    {
      pattern: /\bmusicals?\b/,
      canonicalName: "Musical",
      mainGenre: "Soundtrack",
    },
    {
      pattern: /\bvideo game\b/,
      canonicalName: "Video Game Music",
      mainGenre: "Soundtrack",
    },
    {
      pattern: /\bstage\s*(?:and)?\s*screen\b/,
      canonicalName: "Soundtrack",
      mainGenre: "Soundtrack",
    },
    {
      pattern:
        /\bchristian\b|\bworship\b|\bccm\b|\bpentecostal\b|\bdevotional\b|\bsholawat\b|\bbhajan\b|\bsufi\b|\breligious\b/,
      canonicalName: "Christian/Gospel",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bchristmas\b|\bholiday\b/,
      canonicalName: "Christmas",
      mainGenre: "Pop",
    },
    {
      pattern: /\bchildrens?\b|\blullaby\b/,
      canonicalName: "Children's",
      mainGenre: "Pop",
    },
    {
      pattern: /\bcomedy\b|\bspoken\s*word\b|\bparody\b/,
      canonicalName: "Comedy",
      mainGenre: "Pop",
    },
    {
      pattern: /\bdoo\s*wop\b/,
      canonicalName: "Doo-Wop",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bboogie\b/,
      canonicalName: "Boogie",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /\bragtime\b/,
      canonicalName: "Ragtime",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bavant\s*garde\b/,
      canonicalName: "Avant-Garde",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bfree\s*improvisation\b/,
      canonicalName: "Free Improvisation",
      mainGenre: "Jazz",
    },
    {
      pattern: /\bfield recording\b/,
      canonicalName: "Field Recording",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bspace\s*music\b/,
      canonicalName: "Space Music",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bred\s*dirt\b/,
      canonicalName: "Red Dirt",
      mainGenre: "Country",
    },
    {
      pattern: /\bhillbilly\b/,
      canonicalName: "Hillbilly",
      mainGenre: "Country",
    },
    {
      pattern: /\bwestern\s*swing\b/,
      canonicalName: "Western Swing",
      mainGenre: "Country",
    },
    {
      pattern: /\btrov[ae]\b/,
      canonicalName: "Trova",
      mainGenre: "Latin",
    },
    {
      pattern: /\bson\s*(?:cubano|montuno)?\b/,
      canonicalName: "Son Cubano",
      mainGenre: "Latin",
    },
    {
      pattern: /\bchoro\b/,
      canonicalName: "Choro",
      mainGenre: "Latin",
    },
    {
      pattern: /\bguaguanco\b/,
      canonicalName: "Guaguanco",
      mainGenre: "Latin",
    },
    {
      pattern: /\bfolklore\b/,
      canonicalName: "Folklore",
      mainGenre: "Latin",
    },
    {
      pattern: /\bhuayno\b/,
      canonicalName: "Huayno",
      mainGenre: "Latin",
    },
    {
      pattern: /\bchicha\b/,
      canonicalName: "Chicha",
      mainGenre: "Latin",
    },
    {
      pattern: /\bsea\s*shanties?\b/,
      canonicalName: "Sea Shanties",
      mainGenre: "Folk",
    },
    {
      pattern: /\btraditional\b/,
      canonicalName: "Traditional",
      mainGenre: "Folk",
    },
    {
      pattern: /\bacoustic\b/,
      canonicalName: "Acoustic",
      mainGenre: "Folk",
    },
    {
      pattern: /\bappalachian\b/,
      canonicalName: "Appalachian",
      mainGenre: "Folk",
    },
    {
      pattern: /\bantifolk\b/,
      canonicalName: "Antifolk",
      mainGenre: "Folk",
    },
    { pattern: /\bvocal\b/, canonicalName: "Vocal", mainGenre: "Pop" },
    {
      pattern:
        /\bswedish ballads?\b|\biskelmä?\b|\bschlagerparty\b|\bdansband\b|\bdansktop\b|\brusselater\b/,
      canonicalName: "Schlager",
      mainGenre: "Pop",
    },
    {
      pattern: /\bvariet[eé]\b/,
      canonicalName: "Chanson",
      mainGenre: "Folk",
    },
    {
      pattern: /\bmediev[ae]l\b|\brenaissance\b/,
      canonicalName: "Medieval",
      mainGenre: "Classical",
    },
    {
      pattern: /\brequiem\b/,
      canonicalName: "Requiem",
      mainGenre: "Classical",
    },
    {
      pattern: /\binstrumental\b/,
      canonicalName: "Instrumental",
      mainGenre: "Classical",
    },
    {
      pattern: /\babstract\b/,
      canonicalName: "Abstract",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bacid\b/,
      canonicalName: "Acid",
      mainGenre: "Electronic",
    },
    {
      pattern: /\btribal\b/,
      canonicalName: "Tribal",
      mainGenre: "World",
    },
    {
      pattern: /\bdangdut\b|\bfunkot\b|\bbudots\b/,
      canonicalName: "Dangdut",
      mainGenre: "World",
    },
    {
      pattern: /\bentehno\b|\blaiko\b/,
      canonicalName: "Greek",
      mainGenre: "World",
    },
    {
      pattern: /\bseresta\b/,
      canonicalName: "Seresta",
      mainGenre: "Latin",
    },
    {
      pattern: /\bcold\s*wave\b/,
      canonicalName: "Coldwave",
      mainGenre: "Electronic",
    },
    {
      pattern: /\btropical\b/,
      canonicalName: "Tropical",
      mainGenre: "World",
    },
    { pattern: /\boi\b/, canonicalName: "Oi!", mainGenre: "Rock" },
    {
      pattern: /\bslowcore\b/,
      canonicalName: "Slowcore",
      mainGenre: "Rock",
    },
    {
      pattern: /\bchill\s*step\b/,
      canonicalName: "Chillstep",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bdungeon\s*synth\b/,
      canonicalName: "Dungeon Synth",
      mainGenre: "Electronic",
    },
    {
      pattern: /\bpower\s*electronics\b/,
      canonicalName: "Power Electronics",
      mainGenre: "Electronic",
    },
    {
      pattern: /core\b/,
      canonicalName: "Hardcore",
      mainGenre: "Hardcore",
    },
    { pattern: /rock/, canonicalName: "Rock", mainGenre: "Rock" },
    { pattern: /metal/, canonicalName: "Metal", mainGenre: "Metal" },
    { pattern: /jazz/, canonicalName: "Jazz", mainGenre: "Jazz" },
    { pattern: /blues/, canonicalName: "Blues", mainGenre: "Blues" },
    { pattern: /folk/, canonicalName: "Folk", mainGenre: "Folk" },
    { pattern: /punk/, canonicalName: "Punk", mainGenre: "Rock" },
    {
      pattern: /soul/,
      canonicalName: "Soul",
      mainGenre: "R&B/Soul/Funk",
    },
    {
      pattern: /funk/,
      canonicalName: "Funk",
      mainGenre: "R&B/Soul/Funk",
    },
    { pattern: /reggae/, canonicalName: "Reggae", mainGenre: "Reggae" },
    {
      pattern: /country/,
      canonicalName: "Country",
      mainGenre: "Country",
    },
    {
      pattern: /classical/,
      canonicalName: "Classical",
      mainGenre: "Classical",
    },
    {
      pattern: /disco/,
      canonicalName: "Disco",
      mainGenre: "R&B/Soul/Funk",
    },
    { pattern: /house/, canonicalName: "House", mainGenre: "House" },
    { pattern: /techno/, canonicalName: "Techno", mainGenre: "Techno" },
    { pattern: /trance/, canonicalName: "Trance", mainGenre: "Trance" },
    {
      pattern: /hip\s*hop|rap\b/,
      canonicalName: "Hip-Hop/Rap",
      mainGenre: "Hip-Hop/Rap",
    },
    { pattern: /pop/, canonicalName: "Pop", mainGenre: "Pop" },
    { pattern: /dance/, canonicalName: "Dance", mainGenre: "Dance" },
  ],
  k = [
    /\bpainting\b/,
    /\bnovel\b/,
    /\bfiction\b/,
    /\bfilm\s*noir\b/,
    /\bportrait\b/,
    /\bsculpture\b/,
    /\barchitect/,
    /\bpoetry\b|\blyric\s*poetry\b/,
    /\bessay\b/,
    /\bmemoir\b/,
    /\bautobiography\b/,
    /\bdiary\b/,
    /\bdrama(?:turgy)?\b/,
    /\bcomedy\s+(?:drama|film)\b/,
    /\bcrime\b/,
    /\bthriller\b/,
    /\bhorror\s*film\b/,
    /\bmystery\s*film\b/,
    /\baction\s*film\b/,
    /\badventure\s*(?:film|fiction)\b/,
    /\bbiograph/,
    /\bhistor(?:y|ical)\s*(?:painting|film|drama)\b/,
    /\bdocumentary\b/,
    /\bepic\s*film\b/,
    /\bfantasy\s*film\b/,
    /\bheist\s*film\b/,
    /\bfigure\s*(?:painting)?\b/,
    /\bgenre\s*(?:art|painting)\b/,
    /\blandscape\s*painting\b/,
    /\bmythological\b/,
    /\bnaturalis[mt]\b/,
    /\bneo\s*noir\b/,
    /\bnude\b/,
    /\bstill\s*life\b/,
    /\btragedy\b/,
    /\ballegory\b/,
    /\bdebate\b/,
    /\bserial\b/,
    /\bhaikai\b/,
    /\bitalian\s*neorealism\b/,
    /\bepistolary\b/,
    /\bdark\s*romanticism\b/,
    /\bdecadent\s*movement\b/,
    /\bgothic\s*literature\b/,
    /\bequestrian\b/,
    /\bfantastique\b/,
    /\bgwerz\b/,
    /\banimal\s*(?:art|painting)\b/,
    /\bpublic\s*broadcast\b/,
    /\binterview\b/,
    /\bdialogue\b/,
    /\bpromotional\b/,
    /\bparody\b/,
    /\bkaraoke\b/,
    /\baudiobook\b/,
    /\bspeech\b/,
    /\bchildren\s*television\b/,
    /\bnon\s*music\b/,
  ];
function M(t) {
  if (!t) return null;
  const { mainGenre: e, isOther: n } = (function (t) {
    const e = g(t);
    if (!e)
      return { canonicalName: t.trim(), mainGenre: null, isOther: !0 };
    if ("folk world country" === e)
      return { canonicalName: t.trim(), mainGenre: null, isOther: !0 };
    for (const n of k)
      if (n.test(e))
        return {
          canonicalName: t.trim(),
          mainGenre: null,
          isOther: !0,
        };
    for (const t of y)
      if (t.pattern.test(e))
        return {
          canonicalName: t.canonicalName,
          mainGenre: t.mainGenre,
          isOther: !1,
        };
    return { canonicalName: t.trim(), mainGenre: null, isOther: !0 };
  })(t);
  return n || !e
    ? null
    : "House" === e
      ? "house"
      : "Techno" === e
        ? "techno"
        : "Hardcore" === e
          ? "hardcore"
          : null;
}
function _(t, e, n, r, a, o = 0.05) {
  const i = (60 * a) / n,
    s = Math.floor(i * e),
    c = [],
    l = 60 / n;
  let u = r - (i > 0 ? Math.floor(r / i) : 0) * i;
  for (;;) {
    const n = Math.floor(u * e);
    if (n >= t.length) break;
    const a = n + s,
      o = Math.min(a, t.length),
      f = o - n;
    let h = 0;
    for (let e = n; e < o; e++) {
      const n = t[e];
      h += n * n;
    }
    const p = f > 0 ? Math.sqrt(h / f) : 0,
      d = f / e,
      m = [];
    let b = 0,
      g = 0;
    const y = u + i;
    let k = r + Math.ceil((u - r) / l - 1e-9) * l;
    for (; Math.floor(1e6 * k) <= Math.floor(1e6 * y); ) {
      const n = k - 0.05,
        r = k + 0.05,
        a = Math.max(0, Math.floor(n * e)),
        o = Math.min(t.length, Math.floor(r * e)),
        i = o - a;
      if (i > 0) {
        let e = 0;
        for (let n = a; n < o; n++) {
          const r = t[n];
          e += r * r;
        }
        const n = Math.sqrt(e / i);
        ((b += n), g++, m.push(n));
      }
      k += l;
    }
    const M = g > 0 ? b / g : 0;
    (c.push({
      energy: p,
      beatStrength: M,
      rampType: "flat",
      beatEnergies: m,
      startTime: u,
      duration: d,
      energyDelta: 0,
    }),
      (u += i));
  }
  for (let t = 0; t < c.length; t += 4) {
    const e = c.slice(t, t + 4),
      n = e.length;
    if (n < 2) {
      e.forEach((t) => (t.rampType = "flat"));
      continue;
    }
    let r = 0,
      a = 0,
      i = 0,
      s = 0;
    for (let t = 0; t < n; t++) {
      const n = t,
        o = e[t].beatStrength;
      ((r += n), (a += o), (i += n * o), (s += n * n));
    }
    const l = n * s - r * r;
    let u = 0;
    0 !== l && (u = (n * i - r * a) / l);
    const f = a / n,
      h = f > 0 ? f * o : 1e-6;
    let p = "flat";
    ((p = u > h ? "up" : u < -h ? "down" : "flat"),
      e.forEach((t) => (t.rampType = p)));
  }
  for (let t = 0; t < c.length; t++)
    c[t].energyDelta = 0 === t ? 0 : c[t].energy - c[t - 1].energy;
  return c;
}
function v(t, e, n, r) {
  const a = 1 / (1 + e / (2 * Math.PI * n)),
    o = new Float32Array(t.length);
  o.set(t);
  for (let t = 0; t < r; t++) {
    let t = 0;
    for (let e = 0; e < o.length; e++)
      ((t = a * o[e] + (1 - a) * t), (o[e] = t));
  }
  return o;
}
function F(t, e, n) {
  const r = 1 / Math.SQRT2,
    a = (2 * Math.PI * n) / e,
    o = Math.sin(a),
    i = Math.cos(a),
    s = o / (2 * r),
    c = 1 + s,
    l = (1 - i) / 2 / c,
    u = (1 - i) / c,
    f = (1 - i) / 2 / c,
    h = (-2 * i) / c,
    p = (1 - s) / c,
    d = new Float32Array(t.length);
  let m = 0,
    b = 0,
    g = 0,
    y = 0;
  for (let e = 0; e < t.length; e++) {
    const n = t[e],
      r = l * n + u * m + f * b - h * g - p * y;
    ((b = m), (m = n), (y = g), (g = r), (d[e] = r));
  }
  return d;
}
function x(t, e) {
  let n = 0,
    r = 1 / 0;
  for (let a = 0; a < t.length; a++) {
    const o = Math.abs(t[a].startTime - e);
    o < r && ((r = o), (n = a));
  }
  return n;
}
function B(t, e, n = 8) {
  if (0 === t.length) return 1e-9;
  let r = 0,
    a = 0;
  const o = Math.min(t.length, e + n);
  for (let n = e; n < o; n++) ((r += t[n].energy), a++);
  const i = a > 0 ? r / a : 1e-9;
  return i > 0 ? i : 1e-9;
}
function w(t, e, n = 16) {
  if (e < 1 || e >= t.length) return e;
  const r = t[e].energy;
  if (r <= 0) return e;
  if (t[e - 1].energy > 1.3 * r) return e;
  const a = e + 8,
    o = Math.min(t.length, e + 24);
  if (o - a < 8) return e;
  let i = 0;
  for (let e = a; e < o; e++) i += t[e].energy;
  const s = i / (o - a);
  if (s <= 0 || r >= 0.75 * s) return e;
  const c = Math.min(t.length - 8, e + 1 + n),
    l = Math.max(1.3 * r, 0.9 * s);
  for (let n = e + 1; n < c; n++) {
    if (t[n].energy < l) continue;
    let e = 0;
    const r = Math.max(0, n - 8);
    for (let a = r; a < n; a++) e += t[a].energy;
    const a = e / Math.max(1, n - r);
    if (a <= 0) continue;
    let o = 0;
    for (let e = n; e < n + 8; e++) o += t[e].energy;
    if (o / 8 >= 1.5 * a) return n;
  }
  return e;
}
const G = 4,
  N = 0.85,
  S = 0.65,
  A = 0.2,
  R = 0.005,
  T = 1.5,
  $ = 1 / 0,
  P = 8,
  D = 0.25,
  E = 0.6,
  C = 16,
  L = 16,
  I = 0.7,
  W = 0.01,
  q = 0.85,
  O = 1.4,
  H = 0.6;
function z(t, e) {
  const n = t
      .map((t) => t.energy)
      .slice()
      .sort((t, e) => t - e),
    r = n[Math.floor(n.length / 2)],
    a = Math.max(0, e - $),
    o = e - C;
  for (let e = a; e < o; e++) {
    if (t[e].energy < r) continue;
    let n = !0;
    for (let r = e + 1; r < e + P; r++) {
      if (r >= t.length) {
        n = !1;
        break;
      }
      const a = t[r].energy / t[e].energy;
      if (a < 1 - D || a > 1 + D) {
        n = !1;
        break;
      }
    }
    if (!n) continue;
    const a = Math.max(0, e - 8);
    if (e - a < 4 && e > 4) continue;
    let o = 0,
      i = 0;
    for (let n = a; n < e; n++) ((o += t[n].energy), i++);
    if (!((0 === i ? 0 : o / i) >= t[e].energy * E)) return e;
  }
  return null;
}
function j(t) {
  if (0 === t.length) return null;
  let e = (function (t) {
    const e = [];
    for (let n = 0; n < t.length; n++) {
      const r = Math.min(t.length, n + G);
      let a = 0,
        o = 0;
      for (let e = n; e < r; e++) ((a += t[e].energy), o++);
      e.push(0 === o ? 0 : a / o);
    }
    const n = Math.max(...e),
      r = Math.max(...t.map((t) => t.energy)),
      a = n * N,
      o = r * S;
    for (let n = 0; n < t.length; n++)
      if (e[n] >= a && t[n].energy >= o) return n;
    return null;
  })(t);
  if (null === e) return null;
  if (e / t.length >= 0.4) {
    const n = z(t, e);
    if (null !== n) {
      let r = 1 / 0;
      for (let a = n + 8; a < e; a++)
        t[a].energy < r && (r = t[a].energy);
      r < 0.5 * t[n].energy && (e = n);
    }
  }
  return e;
}
function U(t, e) {
  if (null === e || 0 === t.length) return 0;
  const n = Math.max(...t.map((t) => t.energy));
  return t[e].energy / Math.max(1e-9, n);
}
function V(t, e) {
  if (e < 1 || e > 25) return null;
  const n = t[e].energy;
  if (n <= 0) return null;
  const r = Math.min(t.length, e + 8);
  let a = 0;
  for (let o = e; o < r; o++) t[o].energy >= 0.6 * n && a++;
  if (a < 6) return null;
  let o = 0;
  const i = Math.min(t.length, e + 16);
  for (let r = e + 8; r < i; r++) t[r].energy >= 0.8 * n && o++;
  if (o >= 7) {
    let r = e + 16;
    for (
      let a = e + 16;
      a < Math.min(t.length, e + 40) && t[a].energy >= 0.6 * n;
      a++
    )
      r = a;
    let a = -1,
      o = 0;
    for (let e = r + 1; e < Math.min(t.length, r + 20); e++)
      if (t[e].energy < 0.4 * n) o++;
      else if (o >= 4 && t[e].energy >= 0.7 * n) {
        a = e;
        break;
      }
    if (-1 === a) return null;
    if (t[a].energy >= 0.85 * n) return null;
  }
  let s = -1,
    c = -1;
  for (let r = e + a; r < Math.min(t.length, e + 24); r++)
    if (t[r].energy < 0.4 * n) (-1 === s && (s = r), (c = r));
    else {
      if (-1 !== s && c - s + 1 >= 4) break;
      ((s = -1), (c = -1));
    }
  if (-1 === s || c - s + 1 < 4) return null;
  let l = 0;
  for (let n = e; n < e + a; n++) t[n].energy > l && (l = t[n].energy);
  for (let e = c + 1; e < Math.min(t.length - 4, c + 1 + 24); e++) {
    if (t[e].energy < 0.8 * l) continue;
    let n = 0;
    for (let r = e; r < e + 4; r++) n += t[r].energy;
    if (!(n / 4 < 0.75 * l)) return e;
  }
  return null;
}
function J(t, e, n) {
  const r = Math.max(...t.map((t) => t.energy));
  let a = -1;
  for (let o = 0; o < t.length; o++)
    if (t[o].energy < r * e) -1 === a && (a = o);
    else {
      if (-1 !== a && o - a >= n) return [a, o - 1];
      a = -1;
    }
  return null;
}
function K(t, e, n, o, i = null, s = null) {
  const c =
      i ??
      (function (t, e) {
        let n = t;
        for (const t of [800, 400, 400, 200, 200, 200, 200])
          n = F(n, e, t);
        return n;
      })(t, e),
    l = _(c, e, n, o, 4, 0.02),
    u = j(l),
    f = null !== u && l[u] ? l[u].startTime : null,
    h = U(l, u),
    p = _(v(t, e, 300, 6), e, n, o, 4, 0.02);
  let d = null;
  if (p.length > 0) {
    const n = (function (t, e, n) {
      const r = Math.floor(n * R),
        a = [];
      for (let t = 0; t < e.length; t += r) {
        let n = 0;
        const o = Math.min(t + r, e.length);
        for (let r = t; r < o; r++) {
          const t = Math.abs(e[r]);
          t > n && (n = t);
        }
        a.push(n);
      }
      const o = [0];
      let i = 0,
        s = 0;
      for (let t = 1; t < a.length; t++) {
        const e = a[t] - a[t - 1];
        (o.push(e > 0 ? e : 0), e > 0 && ((i += e), s++));
      }
      const c = (0 === s ? 0 : i / s) * T,
        l = [];
      for (const e of t) {
        const t = Math.floor(e.startTime / R),
          n = Math.min(
            o.length,
            Math.floor((e.startTime + e.duration) / R),
          );
        let r = 0;
        for (let e = t + 1; e < n - 1; e++)
          o[e] > o[e - 1] && o[e] > o[e + 1] && o[e] > c && r++;
        l.push(r);
      }
      return l;
    })(p, t, e);
    if (
      ((d = (function (t, e) {
        const n = [];
        for (let e = 0; e < t.length; e++) {
          const r = Math.min(t.length, e + G);
          let a = 0,
            o = 0;
          for (let n = e; n < r; n++) ((a += t[n].energy), o++);
          n.push(0 === o ? 0 : a / o);
        }
        const r = Math.max(...n),
          a = Math.max(...t.map((t) => t.energy)),
          o = Math.max(...e),
          i = r * N,
          s = a * S,
          c = o * A;
        for (let r = 0; r < t.length; r++)
          if (n[r] >= i && t[r].energy >= s && e[r] >= c) return r;
        return null;
      })(p, n)),
      null !== d && d / p.length >= 0.3)
    ) {
      const t = z(p, d);
      if (null !== t) {
        let e = 1 / 0;
        for (let n = t + 8; n < d; n++)
          p[n].energy < e && (e = p[n].energy);
        const n = p[d].energy >= 1.2 * p[t].energy;
        e < 0.5 * p[t].energy && !n && (d = t);
      }
    }
  }
  const b = null !== d ? p[d].startTime : null,
    g = U(p, d),
    y = (function (t, e) {
      if (null === e) return 0;
      const n = t[e].energy;
      if (0 === n) return 0;
      let r = 0,
        a = 0;
      for (let n = e + 1; n < Math.min(t.length, e + 9); n++)
        ((r += t[n].energy), a++);
      return a > 0 ? r / n / a : 0;
    })(p, d),
    k = (function (t, e) {
      if (null === e) return 0;
      const n = t[e].energy;
      if (0 === n) return 0;
      let r = 0,
        a = 0;
      for (let n = Math.max(0, e - 8); n < e; n++)
        ((r += t[n].energy), a++);
      const o = a > 0 ? r / a : 0;
      return n / Math.max(1e-9, o);
    })(p, d);
  let x = null;
  if (null !== f && null !== b)
    if (Math.abs(f - b) <= L) {
      if (((x = b), null !== d && d / p.length >= 0.25)) {
        const t = (function (t, e) {
          const n = Math.max(...t.map((t) => t.energy)),
            r = e - 16;
          for (let a = 4; a < r; a++) {
            const r = t[a].energy;
            if (r < 0.7 * n) continue;
            let o = 0;
            for (let e = a + 1; e < a + 9 && e < t.length; e++)
              t[e].energy >= 0.7 * r && o++;
            if (o < 4) continue;
            let i = 1 / 0;
            for (let e = Math.max(0, a - 4); e < a; e++)
              t[e].energy < i && (i = t[e].energy);
            if (i >= 0.3 * r) continue;
            let s = 1 / 0;
            for (let n = a + 9; n < e; n++)
              t[n].energy < s && (s = t[n].energy);
            if (!(s > 0.2 * r)) return a;
          }
          return null;
        })(p, d);
        null !== t && (x = p[t].startTime);
      }
      if (
        null !== u &&
        null !== d &&
        u < d &&
        d - u >= 2 &&
        d - u <= 12 &&
        u >= 2
      ) {
        const t = p[u].energy,
          e = Math.max(...p.map((t) => t.energy)),
          n = p[u - 1].energy,
          r = p[u - 2].energy;
        t >= 0.5 * e &&
          Math.min(n, r) < 0.15 * t &&
          (x = p[u].startTime);
      }
    } else if (b > f) {
      const t = null !== u ? u / p.length : 0,
        e = null !== d ? d / p.length : 0;
      if (
        null !== u &&
        null !== d &&
        t < 0.15 &&
        e >= 0.15 &&
        e <= 0.4 &&
        g >= I &&
        y >= 0.8 &&
        k >= 1.3
      )
        x = b;
      else {
        const t = h >= I,
          e = g >= I,
          n = g - h >= W,
          r = y >= q,
          a = k >= O,
          o =
            null !== u && u < p.length && p.length > 0
              ? p[u].energy /
                Math.max(1e-9, Math.max(...p.map((t) => t.energy)))
              : 1,
          i = r && a && (t ? n || (e && o < H) : e),
          s = null !== u ? u / p.length : 0;
        let c = t && s >= 0.15 && s <= 0.7 && o >= 0.55 && o < 0.75;
        if (c && null !== u && null !== d && u < d) {
          const t = 0.5 * p[u].energy;
          let e = 0,
            n = 0;
          for (let r = u + 4; r < d; r++)
            p[r].energy < t ? (e++, e > n && (n = e)) : (e = 0);
          n >= 4 && (c = !1);
        }
        x = i && !c ? b : f;
      }
    } else {
      const t = h >= I,
        e = g >= I,
        n = y >= q,
        r = k >= O,
        a =
          null !== u && u < p.length && p.length > 0
            ? p[u].energy /
              Math.max(1e-9, Math.max(...p.map((t) => t.energy)))
            : 1,
        o = t && e && n && r && a < H;
      let i = !1;
      if (null !== u && null !== d && e && n && r) {
        const t = u / p.length,
          e = d / p.length;
        t >= 0.55 && e >= 0.08 && e <= 0.45 && (i = !0);
      }
      x = o || i ? b : f;
    }
  else x = null !== f ? f : b;
  if (null !== x && p.length > 0) {
    let t = -1,
      e = 1 / 0;
    for (let n = 0; n < p.length; n++) {
      const r = Math.abs(p[n].startTime - x);
      r < e && ((e = r), (t = n));
    }
    const n = (function (t, e) {
      if (e > 4) return null;
      const n = Math.max(...t.map((t) => t.energy));
      if (n <= 0) return null;
      const r = 0.3 * n;
      let a = -1;
      for (let n = e + 1; n < Math.min(t.length, e + 24); n++)
        if (t[n].energy < r) {
          a = n;
          break;
        }
      if (-1 === a) return null;
      const o = e >= 1 ? 7 : 12,
        i = t[e].energy;
      let s = 0,
        c = 1 / 0,
        l = -1;
      for (let n = a; n < Math.min(t.length, a + 40); n++) {
        if (!(t[n].energy < r)) {
          s >= (0 === e && i > 0 && c < 0.1 * i ? 7 : o) && (l = n);
          break;
        }
        (s++, t[n].energy < c && (c = t[n].energy));
      }
      if (-1 === l) return null;
      const u = 0.7 * n;
      for (let e = l; e < Math.min(t.length, l + 8); e++)
        if (t[e].energy >= u) return e;
      return null;
    })(p, t);
    if (null !== n) {
      ((x = p[n].startTime), (t = n));
      const e = V(p, t);
      null !== e && (x = p[e].startTime);
    } else {
      const e = V(p, t);
      if (null !== e) x = p[e].startTime;
      else {
        const e = (function (t, e) {
          if (e <= 4) return null;
          if (t.length < 32) return null;
          const n = Math.max(...t.map((t) => t.energy));
          if (n <= 0) return null;
          const r = Math.max(t[0].energy, t[1]?.energy ?? 0);
          if (r < 0.85 * n) return null;
          for (let e = 0; e < 30 && e < t.length; e++)
            if (t[e].energy < 0.5 * n) return null;
          const a = t[e].energy;
          return a < 0.85 * r || a > 1.3 * r ? null : 0;
        })(p, t);
        null !== e && (x = p[e].startTime);
      }
    }
  }
  if (null !== x && p.length > 0) {
    let t = -1,
      e = 1 / 0;
    for (let n = 0; n < p.length; n++) {
      const r = Math.abs(p[n].startTime - x);
      r < e && ((e = r), (t = n));
    }
    if (t >= 2) {
      const e = p[t].energy,
        n = p[t - 1].energy,
        r = p[t - 2].energy;
      n >= 0.7 * e &&
        r < 0.6 * e &&
        ((x = p[t - 1].startTime), (t -= 1));
    }
    if (t >= 0) {
      const e = w(p, t);
      e !== t && ((x = p[e].startTime), (t = e));
    }
    if (t >= 4) {
      const e = p[0]?.duration ?? 240 / n,
        r = p[t].energy,
        a = Math.round((p[t].startTime - o) / e);
      if (0 !== ((a % 4) + 4) % 4 && r > 0)
        for (let e = 3; e >= 1; e--) {
          const n = t - e;
          if (n < 1) continue;
          if ((((a - e) % 4) + 4) % 4 != 0) continue;
          const o = p[n].energy;
          if (p[n - 1].energy >= 0.7 * o) continue;
          let i = !0;
          for (let e = n; e <= t; e++)
            if (p[e].energy < 0.9 * r) {
              i = !1;
              break;
            }
          if (i) {
            ((x = p[n].startTime), (t = n));
            break;
          }
        }
    }
    if (t >= 4 && t + 7 < p.length) {
      const e = p[t].energy,
        n = Math.max(...p.map((t) => t.energy)),
        r = p[t - 1].energy;
      if (e > 0 && r <= 0.2 * e && e >= 0.85 * n) {
        const n = p[t + 1].energy,
          r = p[t + 2].energy,
          a = p[t + 3].energy;
        if (n <= e && r <= n && a <= r && a <= 0.8 * e) {
          let n = 0,
            r = 1 / 0,
            a = 0;
          for (let e = t + 4; e <= t + 7; e++) {
            const t = p[e].energy;
            (t > n && (n = t), t < r && (r = t), (a += t));
          }
          const o = a / 4;
          r > 0 &&
            n <= 1.2 * r &&
            o <= 0.9 * e &&
            o >= 0.55 * e &&
            ((x = p[t + 4].startTime), (t += 4));
        }
      }
    }
    if (t >= 0) {
      const e = p[0]?.duration ?? 240 / n,
        r = ((Math.round((p[t].startTime - o) / e) % 4) + 4) % 4;
      let a = 0;
      if (0 === r && t + 8 < p.length) {
        const e = p[t].energy,
          n = p[t + 4].energy;
        let r = 1 / 0;
        for (let e = t + 4; e <= t + 7; e++)
          p[e].energy < r && (r = p[e].energy);
        const o = t >= 1 ? p[t - 1].energy : 0;
        e > 0 &&
          n >= 1.2 * e &&
          r >= 1.1 * e &&
          o >= 0.7 * e &&
          o <= e &&
          (a = 4);
      }
      if (1 === r) a = -1;
      else if (3 === r) a = 1;
      else if (2 === r && t + 5 < p.length && t >= 1) {
        const e = p[t].energy,
          n = p[t - 1].energy,
          r = p[t + 1].energy;
        let o = 0;
        for (let e = t + 2; e <= t + 5; e++)
          p[e].energy > o && (o = p[e].energy);
        o > 0 &&
          ((e < 0.92 * o && n >= 0.5 * e && n < 0.9 * e) ||
            (n >= 0.9 * e && r < 0.8 * e && o >= 1.1 * e)) &&
          (a = 2);
      }
      if (0 !== a) {
        const e = t + a;
        e >= 0 && e < p.length && ((x = p[e].startTime), (t = e));
      }
    }
  }
  const B = p[0]?.duration ?? 240 / n,
    $ = M(s);
  if (null !== $ && null !== x) {
    const r = _(v(t, e, 80, 6), e, n, o, 4, 0.02);
    if (r.length > 0) {
      const t = r[0].startTime,
        e = (function (t, e, n) {
          if (0 === e.length) return null;
          const r = Math.max(...e.map((t) => t.energy));
          if (r <= 0) return null;
          const a = "techno" === t ? J(e, 0.35, 3) : J(e, 0.3, 4);
          if (!a) return null;
          if ("hardcore" === t) {
            if (n > 0.4 * e.length || a[0] <= n) return null;
          } else if (n > 4) return null;
          for (let t = a[1] + 1; t < e.length; t++)
            if (e[t].energy >= 0.7 * r) return t;
          return null;
        })($, r, Math.round((x - t) / B));
      null !== e && (x = r[e].startTime);
    }
  }
  const P = null !== x && x < o ? x : o,
    D = {
      markerType: r.Start,
      startTimeSeconds: P,
      duration: B,
      setByUser: !1,
    };
  ((D.color = m.Red), (D.cueType = a.MemoryCue), (D.name = "Start"));
  let E = null;
  return (
    null !== x &&
      ((E = {
        markerType: r.Drop,
        startTimeSeconds: x,
        duration: B,
        setByUser: !1,
      }),
      (E.color = m.Red),
      (E.cueType = a.MemoryCue),
      (E.name = "Drop")),
    { drop: E, start: D }
  );
}
const Y = 64,
  Q = 16,
  Z = 1,
  X = 30,
  tt = 16,
  et = 16,
  nt = 0.48,
  rt = 0.18,
  at = 192,
  ot = 96,
  it = 30,
  st = 0.25,
  ct = 0.1,
  lt = 8,
  ut = 0.85,
  ft = 0.2,
  ht = 0.4,
  pt = 8,
  dt = 0.3,
  mt = 0.5,
  bt = 0.55,
  gt = 16,
  yt = 4,
  kt = 0.4,
  Mt = 0.3,
  _t = 6;
function vt(t, e, n, o, i, s, c, l = null, u = !1) {
  const f = "hardcore" !== M(l),
    h = _(v(t, e, 300, 6), e, n, o, 4, 0.02);
  if (0 === h.length) return { breakdown: null };
  const p = x(h, i),
    d = B(h, p),
    b = [],
    g = h[p].startTime / s;
  let y = !1;
  if (g < 0.1)
    for (let t = Z; t <= X; t++) {
      const e = p + t * Q;
      if (e + tt > h.length) break;
      const n = h[e].startTime / s;
      if (n < 0.35 || n > 0.57) continue;
      let r = 0;
      for (let t = e; t < e + tt; t++) r += h[t].energy;
      r /= tt;
      let a = 0,
        o = 0;
      for (let t = Math.max(p, e - et); t < e; t++)
        ((a += h[t].energy), o++);
      a = o > 0 ? a / o : 0;
      if (
        Math.max(0, 1 - r / d) * Math.max(0, (Math.min(a, d) - r) / d) >
        0.3
      ) {
        y = !0;
        break;
      }
    }
  for (let t = Z; t <= X; t++) {
    const e = p + t * Q;
    if (e + tt > h.length) break;
    let n = 0;
    for (let t = e; t < e + tt; t++) n += h[t].energy;
    n /= tt;
    let r = 0,
      a = 0;
    for (let t = Math.max(p, e - et); t < e; t++)
      ((r += h[t].energy), a++);
    r = a > 0 ? r / a : 0;
    const o = Math.max(0, 1 - n / d),
      i = Math.min(r, d),
      c = Math.max(0, (i - n) / d),
      l = h[e].startTime / s,
      m = (l - nt) / rt,
      k = Math.exp(-0.5 * m * m),
      M = (t * Y - at) / ot,
      _ = Math.exp(-0.5 * M * M),
      v = Math.min(1, r / d);
    let F = o * c * (f ? Math.max(k, _) : _) * v;
    s - h[e].startTime < it && (F = 0);
    if (
      (l < (u ? ct : g < 0.1 && o > 0.75 && !y ? 0.15 : st) && (F = 0),
      l > bt)
    ) {
      let t = 0,
        n = !1;
      for (let r = p + 1; r < e; r++)
        if (h[r].energy < d * dt) {
          if ((t++, t >= pt)) {
            n = !0;
            break;
          }
        } else t = 0;
      n && (F *= mt);
    }
    b.push({ sectionIndex: e, startTime: h[e].startTime, score: F });
  }
  if (0 === b.length) return { breakdown: null };
  let k = b[0];
  for (const t of b) t.score > k.score && (k = t);
  if (0 === k.score) return { breakdown: null };
  const F = (function (t, e, n, r, a) {
      const o = Ft(t, e, n, r, a, yt, 0);
      if (-1 !== o) return o;
      const i = Math.min(t.length, e + _t);
      let s = 0;
      for (let n = e; n < i; n++) s += t[n].energy;
      if (s / Math.max(1, i - e) < 0.5 * n) return e;
      const c = Ft(t, e, n, r, a, yt, yt / 2);
      return -1 !== c ? c : e;
    })(h, k.sectionIndex, d, p, s),
    w = F + 1,
    G = Math.min(h.length, w + lt);
  let N = 0;
  for (let t = w; t < G; t++) N += h[t].energy;
  if (((N /= Math.max(1, G - w)), N >= d * ut))
    return { breakdown: null };
  const S = Math.min(h.length, w + Math.floor(lt / 2));
  let A = 0,
    R = 0;
  for (let t = w; t < S; t++) ((A += h[t].energy), R++);
  if (((A = R > 0 ? A / R : 0), A >= 0.8 * d))
    return { breakdown: null };
  let T = 0,
    $ = 0;
  for (let t = Math.max(p, F - et); t < F; t++)
    ((T += h[t].energy), $++);
  if (((T = $ > 0 ? T / $ : 0), T - N < d * ft))
    return { breakdown: null };
  let P = !1;
  for (let t = F + tt; t < h.length; t++)
    if (h[t].energy >= d * ht) {
      P = !0;
      break;
    }
  if (!P) return { breakdown: null };
  let D = F;
  const E = (F > 0 ? h[F - 1].energy : 0) >= 0.7 * d;
  let C = !1;
  if (!E) {
    const t = Math.max(p + 1, F - 12);
    for (let e = F; e > t; e--) {
      const t = h[e - 1].energy,
        n = h[e].energy;
      if (t >= 0.7 * d && n <= 0.6 * t) {
        let t = 0;
        for (let n = e + 1; n < F; n++) h[n].energy >= 0.85 * d && t++;
        if (t < 3) {
          ((D = e), (C = !0));
          break;
        }
      }
    }
  }
  let L = !1;
  if (!E && !C) {
    const t = Math.max(p + 1, F - 24);
    for (let e = F; e > t && !(h[e].energy >= 0.5 * d); e--) {
      const t = h[e - 1].energy,
        n = h[e].energy;
      if (t >= 0.5 * d && n > 0 && t / n >= 1.5) {
        ((D = e), (L = !0));
        break;
      }
    }
  }
  if (D > p + 1 && D < F + 1) {
    const t = 0.3 * d,
      e = 0.7 * d,
      n = 0.7 * d;
    let r = 0;
    for (let n = D - 1; n > p; n--) {
      const a = h[n].energy;
      if (!(a >= t && a < e)) break;
      r++;
    }
    if (r >= 8) {
      const t = Math.max(p + 1, D - r);
      for (let e = D - 1; e >= t; e--)
        if (h[e - 1].energy >= n) {
          D = e;
          break;
        }
    }
  }
  const I = Math.round((D - p) / yt) * yt;
  if (((D = Math.max(p + yt, p + I)), !E && !C && !L)) {
    let t = -1,
      e = 0;
    for (let n = F + 1; n < h.length; n++)
      if (h[n].energy >= d * ut) {
        if ((e++, e >= 8)) {
          t = n - 7;
          break;
        }
      } else e = 0;
    if (t > F) {
      let e = t,
        n = 1;
      for (let r = t - 1; r > F; r--)
        if (h[r].energy >= d * ut) ((e = r), (n = 1));
        else {
          if (!(n > 0)) break;
          n--;
        }
      t = e;
      let r = -1;
      const a = Math.max(p, D - 64);
      for (let t = D; t > a; t--)
        if (h[t].energy >= 0.5 * d) {
          r = t;
          break;
        }
      if (r > p) {
        const e = t - r,
          n = 16,
          a = Math.round(e / n) * n;
        a > 0 && (D = Math.max(p + yt, t - a));
      }
    }
  }
  let W = -1,
    q = 0;
  for (let t = F + 1; t < h.length; t++)
    if (h[t].energy >= d * ut) {
      if ((q++, q >= 8)) {
        W = t - 8 + 1;
        break;
      }
    } else q = 0;
  if (-1 !== W) {
    const t = 4 * (W - D);
    if (16 * Math.ceil(t / 16) < c) return { breakdown: null };
  }
  const O = h[0]?.duration ?? 240 / n,
    H = {
      markerType: r.Breakdown,
      startTimeSeconds: h[D].startTime,
      duration: O,
      setByUser: !1,
    };
  return (
    (H.color = m.Blue),
    (H.cueType = a.MemoryCue),
    (H.name = "Breakdown"),
    { breakdown: H }
  );
}
function Ft(t, e, n, r, a, o, i) {
  const s = e - ((((e - r) % o) + o) % o) + i;
  let c = s - gt;
  for (; c <= r; ) c += o;
  const l = s + gt;
  for (let e = c; e <= l; e += o) {
    if (e <= 0) continue;
    if (e + _t > t.length) break;
    if (a - t[e].startTime < it) continue;
    const r = t[e].energy,
      o = Math.max(0, e - 4);
    let i = 0;
    for (let n = o; n < e; n++) i += t[n].energy;
    const s = i / Math.max(1, e - o);
    if (r >= s * kt) continue;
    if (s < 0.5 * n) continue;
    const c = t[e + 1].energy;
    if (r >= n * Mt && c >= n * Mt) continue;
    let l = 0;
    for (let n = e; n < e + _t; n++) l += t[n].energy;
    if (!(l / _t >= n * Mt)) return e;
  }
  return -1;
}
const xt = 4,
  Bt = 64,
  wt = 320,
  Gt = 96,
  Nt = 192,
  St = 16,
  At = 16,
  Rt = 0.6,
  Tt = 0.7,
  $t = 0.65,
  Pt = 0.1,
  Dt = 128,
  Et = 64,
  Ct = 0.82,
  Lt = 128,
  It = 384,
  Wt = 0.65;
function qt(t, e, n) {
  if (0 === e.length) return t;
  let r = e[Math.min(e.length - 1, n)],
    a = Math.abs(r.startTime - t);
  for (let o = n; o < e.length; o++) {
    if ((o - n) % xt != 0) continue;
    const i = Math.abs(e[o].startTime - t);
    i < a && ((a = i), (r = e[o]));
  }
  return r.startTime;
}
function Ot(t, e, n, r, a, o) {
  if (null !== r && r > n) {
    const n = r + Lt * o;
    if (n < a) return qt(n, t, e);
  }
  const i = n + It * o;
  return qt(i < a ? i : a * Wt, t, e);
}
function Ht(t, e, n, r, a, o, i) {
  const s = _(v(t, e, 300, 6), e, n, r, 4, 0.02),
    c = 60 / n;
  if (0 === s.length) {
    return {
      secondDrop: jt(Ot([], 0, a, o, i, c), s, n),
      tier: "heuristic-fallback",
    };
  }
  const l = x(s, a),
    u = B(s, l),
    f = Gt / 4,
    h = Bt / 4,
    p = Nt / 4,
    d = null !== o && o > a ? x(s, o) : l + p,
    m = l + f,
    b = d + h;
  let g = Math.max(m, b);
  const y = g - l;
  g = l + Math.ceil(y / xt) * xt;
  const k = null !== o && o > a ? o : a + Nt * c,
    M = [],
    F = wt / 4;
  for (
    let t = g;
    t + St < s.length && !(t - d > F) && !(s[t].startTime / i > Ct);
    t += xt
  ) {
    const e = (s[t].startTime - k) / c;
    let n = 0;
    for (let e = t; e < t + St; e++) n += s[e].energy;
    n /= St;
    let r = 0,
      a = 0;
    for (let e = Math.max(0, t - At); e < t; e++)
      ((r += s[e].energy), a++);
    r = a > 0 ? r / a : 0;
    const o = n >= u * Rt,
      f = o && r <= u * Tt,
      h = t >= 1 ? s[t - 1].energy : 0,
      p = s[t].energy,
      d = f || (o && p >= 0.85 * u && h < 0.4 * u),
      m = Math.min(1, n / u),
      b = Math.max(0, (n - r) / u),
      g = (s[t].startTime / i - $t) / Pt;
    let y = Math.exp(-0.5 * g * g);
    const _ = (e - Dt) / Et,
      v = Math.exp(-0.5 * _ * _),
      F = k / i;
    ((s[l].startTime / i < 0.15 && F < 0.35) ||
      (k + Dt * c) / i < 0.55) &&
      (y = 0);
    const x = m * b * Math.max(y, v);
    M.push({
      sectionIndex: t,
      startTime: s[t].startTime,
      score: x,
      candEnergy: n,
      passes: d,
    });
  }
  const w = M.filter((t) => t.passes);
  if (w.length > 0) {
    let t = w[0];
    for (const e of w) e.score > t.score && (t = e);
    return {
      secondDrop: jt(s[zt(s, t.sectionIndex, l, u)].startTime, s, n),
      tier: "principled",
    };
  }
  if (null === o) return { secondDrop: null, tier: "no-result" };
  if (M.length > 0) {
    let t = M[0].candEnergy;
    for (const e of M) e.candEnergy > t && (t = e.candEnergy);
    const e = 0.95 * t;
    let r = M[0];
    for (const t of M)
      if (t.candEnergy >= e) {
        r = t;
        break;
      }
    return {
      secondDrop: jt(s[zt(s, r.sectionIndex, l, u)].startTime, s, n),
      tier: "energy-fallback",
    };
  }
  return {
    secondDrop: jt(Ot(s, l, a, o, i, c), s, n),
    tier: "heuristic-fallback",
  };
}
function zt(t, e, n, r) {
  let a = (function (t, e, n) {
    if (e <= n + 1) return e;
    const r = t[e].energy;
    if (r <= 0) return e;
    const a = 16;
    let o = e;
    for (; o > n + 1 && e - o < a; ) {
      const n = t[o - 1].energy;
      if (n < 0.85 * r) {
        if (n < 0.6 * r) return o;
        if (
          o - 2 >= 0 &&
          t[o - 2].energy >= 0.85 * r &&
          t[o - 2].energy >= 1.2 * n
        ) {
          o -= 2;
          continue;
        }
        return e;
      }
      o--;
    }
    return e;
  })(t, e, n);
  return (
    (a = w(t, a)),
    (a = (function (t, e) {
      if (e + 8 >= t.length) return e;
      const n = t[e].energy;
      if (n <= 0) return e;
      let r = 0;
      for (let n = e; n < e + 4; n++) r += t[n].energy;
      if (((r /= 4), r >= 0.75 * n)) return e;
      const a = e + 8,
        o = Math.min(t.length, e + 24);
      if (o - a < 8) return e;
      let i = 0;
      for (let e = a; e < o; e++) i += t[e].energy;
      if (((i /= o - a), i <= 0 || i < 1.3 * n)) return e;
      const s = Math.min(t.length - 4, e + 16);
      for (let n = e + 1; n < s; n++) {
        let e = 0;
        for (let r = n; r < n + 4; r++) e += t[r].energy;
        if (((e /= 4), e < 0.85 * i)) continue;
        let r = !0;
        for (let e = n; e < n + 4; e++)
          if (t[e].energy < 0.7 * i) {
            r = !1;
            break;
          }
        if (r) return n;
      }
      return e;
    })(t, a)),
    (a = (function (t, e, n) {
      if (e + 12 >= t.length) return e;
      const r = t[e].energy;
      if (r <= 0) return e;
      if (r >= 0.83 * n) return e;
      if (e >= 1 && e + 1 < t.length) {
        const a = t[e - 1].energy,
          o = t[e + 1].energy;
        if (a < 0.25 * n && r >= 0.65 * n && o >= 1.25 * r) return e;
      }
      const a = Math.min(t.length, e + 17);
      let o = 0;
      for (let n = e + 1; n + 4 <= a; n++) {
        let e = 0;
        for (let r = n; r < n + 4; r++) e += t[r].energy;
        ((e /= 4), e > o && (o = e));
      }
      if (o <= 0) return e;
      if (r >= 0.8 * o) return e;
      const i = Math.min(t.length, e + 9);
      let s = r;
      for (let n = e + 1; n < i; n++) {
        const r = t[n].energy;
        if (r < 0.65 * s) return e;
        s = r;
      }
      for (let n = e + 1; n + 4 <= a; n++)
        if (t[n].energy >= 0.98 * o) return n;
      return e;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 24 >= t.length) return e;
      const r = t[e].energy;
      if (r < 0.85 * n) return e;
      let a = 0;
      for (
        let r = e;
        r < Math.min(t.length, e + 8) && t[r].energy >= 0.85 * n;
        r++
      )
        a++;
      if (a >= 8) return e;
      let o = 0;
      for (let r = e; r < Math.min(t.length, e + 16); r++)
        t[r].energy >= 0.85 * n && o++;
      if (o >= 13) return e;
      const i = Math.min(t.length - 16, e + 20);
      for (let r = e + 4; r <= i; r++) {
        let a = !0,
          o = 0;
        for (let e = r; e < r + 16; e++) {
          if (t[e].energy < 0.85 * n) {
            a = !1;
            break;
          }
          o += t[e].energy;
        }
        if (!a) continue;
        if (o / 16 < 0.95 * n) continue;
        let i = !1;
        for (let a = e + 1; a < r; a++)
          if (t[a].energy < 0.3 * n) {
            i = !0;
            break;
          }
        return i ? r : e;
      }
      return e;
    })(t, a, r)),
    (a = (function (t, e) {
      if (e + 12 >= t.length) return e;
      const n = t[e].energy;
      if (n <= 0) return e;
      for (let r = 3; r <= 5; r++) {
        const a = e + r;
        if (a + 8 >= t.length) break;
        const o = t[a].energy;
        if (o < 1.2 * n) continue;
        let i = !1;
        for (let r = e + 1; r < a; r++)
          if (t[r].energy < 0.75 * n) {
            i = !0;
            break;
          }
        if (!i) continue;
        let s = !0;
        for (let e = a; e < a + 8; e++)
          if (t[e].energy < n) {
            s = !1;
            break;
          }
        if (!s) continue;
        let c = !0;
        for (let n = e; n < e + r; n++)
          if (!(t[n].energy >= 0.9 * o)) {
            c = !1;
            break;
          }
        if (!c) return a;
      }
      return e;
    })(t, a)),
    (a = (function (t, e, n) {
      if (e + 7 >= t.length || e < 3) return e;
      const r = t[e].energy;
      if (r < 0.75 * n) return e;
      let a = 0;
      for (let n = e + 4; n <= e + 7; n++) a += t[n].energy;
      const o = a / 4,
        i =
          t[e - 1].energy < 0.3 * n &&
          t[e - 2].energy < 0.3 * n &&
          t[e - 3].energy < 0.3 * n;
      if (i && o >= 0.85 * n) {
        let n = !1;
        for (let a = e + 1; a <= e + 3; a++)
          if (t[a].energy <= 0.3 * r) {
            n = !0;
            break;
          }
        if (n) return e + 4;
      }
      if (o >= 1.1 * r && o >= 0.95 * n) {
        let n = !1;
        for (let a = e + 1; a <= e + 3; a++)
          if (t[a].energy <= 0.5 * r) {
            n = !0;
            break;
          }
        if (n) return e + 4;
      }
      if (e + 7 < t.length && e + 3 < t.length) {
        let r = !0;
        for (let n = 0; n < 3; n++)
          if (t[e + n + 1].energy > 0.9 * t[e + n].energy) {
            r = !1;
            break;
          }
        if (r) {
          let r = 0;
          for (let n = e; n <= e + 3; n++) r += t[n].energy;
          const a = r / 4;
          if (a >= 0.6 * n && a <= 0.95 * n) {
            let r = 0;
            for (let n = e + 4; n <= e + 7; n++) r += t[n].energy;
            const o = r / 4;
            if (o >= 0.95 * n && o >= 1.2 * a) return e + 4;
          }
        }
      }
      if (e + 11 < t.length) {
        let a = 0;
        for (let n = e; n <= e + 6; n++) a += t[n].energy;
        const o = a / 7;
        if (
          o >= 0.7 * n &&
          o <= 0.95 * n &&
          t[e + 7].energy <= 0.55 * r
        ) {
          let r = 0;
          for (let n = e + 8; n <= e + 11; n++) r += t[n].energy;
          const a = r / 4;
          if (a >= 0.95 * n && a >= 1.2 * o) return e + 8;
        }
      }
      return e;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 5 >= t.length || e < 2) return e;
      const r = t[e].energy;
      if (r < 0.7 * n) return e;
      const a = t[e - 1].energy,
        o = t[e - 2].energy;
      if (a < 0.4 * n || a > 0.85 * n) return e;
      if (o >= 0.85 * n) return e;
      const i = t[e + 1].energy,
        s = t[e + 2].energy;
      if (i >= 0.85 * r) return e;
      if (s < 0.95 * r) return e;
      if (s < 0.85 * n) return e;
      let c = 0;
      for (let n = e + 2; n <= e + 5; n++) c += t[n].energy;
      return c / 4 < 0.85 * n ? e : e + 2;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 16 >= t.length) return e;
      const r = t[e].energy;
      if (r <= 0) return e;
      if (r >= 0.85 * n) return e;
      let a = -1;
      for (let n = e + 4; n <= e + 16 && n < t.length; n++)
        if (t[n].energy < 0.55 * r) {
          a = n;
          break;
        }
      if (-1 === a) return e;
      for (let o = a + 1; o <= e + 24 && o + 4 < t.length; o++) {
        if ((o - e) % xt != 0) continue;
        let a = 0;
        for (let e = o; e < o + 4; e++) a += t[e].energy;
        const i = a / 4;
        if (i >= 1.3 * r && i >= 0.95 * n) return o;
      }
      return e;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 2 >= t.length) return e;
      const r = t[e].energy;
      if (r <= 0) return e;
      if (r >= 0.3 * n) return e;
      const a = t[e + 1].energy;
      if (a >= 3 * r && a >= 0.95 * n) return e + 1;
      const o = t[e + 2].energy;
      return o >= 3 * r && o >= 0.95 * n ? e + 2 : e;
    })(t, a, r)),
    (a = (function (t, e, n, r) {
      if (e + 5 >= t.length || e < 1) return e;
      const a = e - n;
      if (((a % xt) + xt) % xt != 2) return e;
      const o = t[e].energy;
      if (o < 0.85 * r) return e;
      const i = t[e + 1].energy,
        s = t[e + 2].energy;
      if (i >= 0.92 * o) return e;
      const c = i < 0.4 * o && s >= 0.85 * r;
      if (!(s >= 1.05 * o || c)) return e;
      let l = 0;
      for (let n = e + 2; n <= e + 5; n++) l += t[n].energy;
      return l / 4 < 0.9 * r ? e : e + 2;
    })(t, a, n, r)),
    (a = (function (t, e, n) {
      if (e + 11 >= t.length) return e;
      const r = t[e].energy;
      if (r < 0.7 * n) return e;
      let a = 1 / 0;
      for (let n = e + 1; n <= e + 5; n++)
        t[n].energy < a && (a = t[n].energy);
      const o = a >= 0.8 * r;
      if (!(a >= 0.7 * r)) return e;
      const i = Math.min(t[e + 6].energy, t[e + 7].energy);
      if (i > (o ? 0.45 * r : 0.25 * r)) return e;
      const s = t[e + 8].energy;
      if (s < 1.08 * r) return e;
      let c = 0;
      for (let n = e + 8; n <= e + 11; n++) c += t[n].energy;
      return c / 4 < 0.92 * r ? e : e + 8;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 7 >= t.length || e < 4) return e;
      const r = t[e].energy;
      if (r < 0.55 * n) return e;
      let a = 1 / 0;
      for (let n = e - 4; n < e; n++)
        t[n].energy < a && (a = t[n].energy);
      if (a > 0.55 * r) return e;
      const o = Math.max(t[e + 1].energy, t[e + 2].energy);
      if (o < 1.3 * r) return e;
      const i = t[e + 3].energy;
      if (i > 0.85 * o) return e;
      if (i < 0.4 * r) return e;
      let s = 0,
        c = 1 / 0,
        l = 0;
      for (let n = e + 4; n <= e + 7; n++) {
        const e = t[n].energy;
        (e > s && (s = e), e < c && (c = e), (l += e));
      }
      if (c <= 0 || s > 1.2 * c) return e;
      const u = l / 4;
      return u < 0.95 * r || u < 0.8 * n ? e : e + 4;
    })(t, a, r)),
    (a = (function (t, e, n) {
      if (e + 7 >= t.length || e < 1) return e;
      const r = t[e].energy;
      if (r <= 0) return e;
      if (r < 0.85 * n) return e;
      const a = t[e - 1].energy;
      if (a > 0.2 * r) return e;
      const o = t[e + 1].energy,
        i = t[e + 2].energy,
        s = t[e + 3].energy;
      if (!(o <= r && i <= o && s <= i && s <= 0.8 * r)) return e;
      let c = 0,
        l = 1 / 0,
        u = 0;
      for (let n = e + 4; n <= e + 7; n++) {
        const e = t[n].energy;
        (e > c && (c = e), e < l && (l = e), (u += e));
      }
      if (l <= 0 || c > 1.2 * l) return e;
      const f = u / 4;
      return f > 0.9 * r || f < 0.55 * r ? e : e + 4;
    })(t, a, r)),
    a
  );
}
function jt(t, e, n) {
  const o = e[0]?.duration ?? 240 / n,
    i = {
      markerType: r.SecondDrop,
      startTimeSeconds: t,
      duration: o,
      setByUser: !1,
    };
  return (
    (i.color = m.Red),
    (i.cueType = a.MemoryCue),
    (i.name = "Second Drop"),
    i
  );
}
const Ut = 0.1,
  Vt = 0.75,
  Jt = 0.55,
  Kt = 0.3,
  Yt = 16,
  Qt = 0.15,
  Zt = 0.35,
  Xt = 0.45,
  te = 64,
  ee = 6,
  ne = 16,
  re = 4;
function ae(t, e, n, r) {
  const a = Math.max(0, Math.floor((n - r) * e)),
    o = Math.min(t.length, Math.floor((n + r) * e));
  if (o <= a) return 0;
  let i = 0;
  for (let e = a; e < o; e++) i += t[e] * t[e];
  return Math.sqrt(i / (o - a));
}
function oe(t, e) {
  const n = [...t].sort((t, e) => t - e);
  return n[Math.floor(n.length * e)];
}
const ie = 0.1,
  se = 12,
  ce = 0.5;
function le(t, e, n, o, i, c, l = null, u = {}) {
  const f = t.length / e,
    h = [],
    p = K(t, e, n, o, l, u.genre ?? null);
  let d = p.start,
    b = p.drop,
    g = b?.startTimeSeconds ?? null,
    y = null,
    k = null,
    M = null,
    F = null;
  if (null !== g) {
    const c = vt(t, e, n, o, g, f, i, u.genre ?? null);
    ((y = c.breakdown), (k = c.breakdown?.startTimeSeconds ?? null));
    const l = Ht(t, e, n, o, g, k, f);
    if (
      ((M = l.secondDrop),
      (F = l.secondDrop?.startTimeSeconds ?? null),
      u.dropAtStart === s.Never && null !== F && f > 0)
    ) {
      let s = !1;
      if (g / f < ie && g < se) s = !0;
      else if (g / f < ce && F / f < ce) {
        const r = vt(t, e, n, o, F, f, i, u.genre ?? null),
          a = Ht(
            t,
            e,
            n,
            o,
            F,
            r.breakdown?.startTimeSeconds ?? null,
            f,
          );
        s = null !== a.secondDrop && "principled" === a.tier;
      }
      if (s) {
        if (
          (function (t) {
            if (t.length < 8) return !1;
            const e = t.map((t) => t.energy),
              n = e.slice().sort((t, e) => t - e),
              r = n[Math.floor(0.95 * (n.length - 1))] || 1e-9;
            let a = 0,
              o = e.length - 1;
            for (; a < o && e[a] < 0.05 * r; ) a++;
            for (; o > a && e[o] < 0.05 * r; ) o--;
            const i = e.slice(a, o + 1);
            if (i.length < 8) return !1;
            const s = i.filter((t) => t >= 0.5 * r).length / i.length;
            let c = 0,
              l = 0,
              u = 0;
            for (const t of i)
              t < 0.2 * r ? (l++, u++, l > c && (c = l)) : (l = 0);
            const f = u / i.length;
            return s >= 0.55 && c <= 4 && f >= 0.075;
          })(_(v(t, e, 300, 6), e, n, o, 4, 0.02))
        ) {
          const r = vt(t, e, n, o, g, f, i, u.genre ?? null, !0);
          ((y = r.breakdown),
            (k = r.breakdown?.startTimeSeconds ?? null));
          const a = Ht(t, e, n, o, g, k, f);
          ((M = a.secondDrop),
            (F = a.secondDrop?.startTimeSeconds ?? null),
            (s = !1));
        }
      }
      if (s) {
        const s = F,
          c = {
            markerType: r.Drop,
            startTimeSeconds: s,
            duration: M.duration,
            setByUser: !1,
          };
        ((c.color = m.Red),
          (c.cueType = a.MemoryCue),
          (c.name = "Drop"),
          (b = c),
          (g = s),
          d.startTimeSeconds > s &&
            (d = { ...d, startTimeSeconds: s }));
        const l = vt(t, e, n, o, s, f, i, u.genre ?? null);
        ((y = l.breakdown),
          (k = l.breakdown?.startTimeSeconds ?? null));
        const h = Ht(t, e, n, o, s, k, f);
        ((M = h.secondDrop),
          (F = h.secondDrop?.startTimeSeconds ?? null));
      }
    }
  }
  (h.push(d), b && h.push(b), y && h.push(y), M && h.push(M));
  let w = null;
  if (null !== F) {
    let s = vt(t, e, n, o, F, f, i, u.genre ?? null);
    if (
      (s.breakdown ||
        (s = (function (t, e, n, o, i, s) {
          const c = _(v(t, e, 300, 6), e, n, o, 4, 0.02);
          if (0 === c.length) return { breakdown: null };
          const l = x(c, i),
            u = B(c, l);
          if (u <= 0) return { breakdown: null };
          const f = (t, e) => {
            let n = 0,
              r = 0;
            for (
              let a = Math.max(0, t);
              a < Math.min(c.length, t + e);
              a++
            )
              ((n += c[a].energy), r++);
            return r ? n / r : 0;
          };
          for (let t = l + Q; t + 8 < c.length; t += Q) {
            const e = f(t, 8);
            if (e > 0.7 * u) continue;
            if (f(t - 8, 8) - e < 0.25 * u) continue;
            let o = !1;
            for (let e = t; e < c.length; e++)
              if (c[e].energy > 0.95 * u) {
                o = !0;
                break;
              }
            if (o) continue;
            let i = t;
            const s = Math.max(l + 1, t - Q);
            for (; i > s && c[i - 1].energy <= 0.7 * u; ) i--;
            const h = c[0]?.duration ?? 240 / n,
              p = {
                markerType: r.Breakdown,
                startTimeSeconds: c[i].startTime,
                duration: h,
                setByUser: !1,
              };
            return (
              (p.color = m.Blue),
              (p.cueType = a.MemoryCue),
              (p.name = "Breakdown"),
              { breakdown: p }
            );
          }
          return { breakdown: null };
        })(t, e, n, o, F)),
      s.breakdown)
    ) {
      const t = {
        ...s.breakdown,
        markerType: r.SecondBreakdown,
        name: "Second Breakdown",
      };
      (h.push(t), (w = s.breakdown.startTimeSeconds));
    }
  }
  const G = (function (t, e, n, o, i) {
    const s = 60 / n,
      c = Ut / 2,
      l = Math.max(0, Math.floor((i - o) / s));
    if (l < 4) return { lastBeat: null };
    const u = v(t, e, 300, 6),
      f = [],
      h = [];
    for (let n = 0; n < l; n++) {
      const r = o + n * s;
      (f.push(ae(t, e, r, c)), h.push(ae(u, e, r, c)));
    }
    const p = oe(f, Vt),
      d = oe(h, Vt);
    if (p <= 0) return { lastBeat: null };
    const b = [],
      g = [];
    for (let t = 0; t < l; t++) {
      let e = 0,
        n = 0;
      for (let r = Math.max(0, t - re + 1); r <= t; r++)
        (f[r] > e && (e = f[r]), h[r] > n && (n = h[r]));
      (b.push(e), g.push(n));
    }
    const y = p * Jt,
      k = d * Kt,
      M = p * Qt,
      _ = p * Zt,
      F = (t) => b[t] >= y && g[t] >= k;
    let x = -1,
      B = 0,
      w = -1;
    for (let t = 0; t < l; t++)
      F(t) ? (B++, (w = t)) : (B >= ne && (x = w), (B = 0));
    if ((B >= ne && (x = w), -1 === x))
      for (let t = l - 1; t >= 0; t--)
        if (b[t] >= y) {
          x = t;
          break;
        }
    if (-1 === x) return { lastBeat: null };
    let G = x;
    for (let t = Math.min(l - 1, x + te); t > x; t--) {
      if (f[t] < _) continue;
      let e = !1;
      for (let n = t + 1; n <= Math.min(l - 1, t + ee); n++)
        if (f[n] < M) {
          e = !0;
          break;
        }
      if (!e) continue;
      let n = !0;
      for (let e = t + ee + 1; e < l; e++)
        if (f[e] >= _) {
          n = !1;
          break;
        }
      if (!n) continue;
      let r = 0,
        a = 0;
      for (let e = x + 1; e <= t; e++) ((r += f[e]), a++);
      if (a > 0 && r / a >= p * Xt) {
        G = t;
        break;
      }
    }
    let N = Math.round(G / Yt) * Yt;
    (N >= l && (N = Math.floor(G / Yt) * Yt), N < 0 && (N = 0));
    const S = o + N * s,
      A = {
        markerType: r.Lastbeat,
        startTimeSeconds: S,
        duration: s,
        setByUser: !1,
      };
    return (
      (A.color = m.Orange),
      (A.cueType = a.MemoryCue),
      (A.name = "Last beat"),
      { lastBeat: A }
    );
  })(t, e, n, o, f);
  if ((G.lastBeat && h.push(G.lastBeat), c && null !== g)) {
    const i = (function (t, e, n) {
      if (0 === t.length) return { startTime: -1, beats: -1 };
      const r = t.findIndex((t) => t.startTime >= e);
      let a = t.findIndex((t) => t.startTime >= n);
      if ((-1 === a && (a = t.length), r < 0 || a <= r))
        return { startTime: -1, beats: -1 };
      const o = a - 4 - 4,
        i = r + 4;
      if (o < i) return { startTime: -1, beats: -1 };
      let s = 0;
      for (let e = i; e < a; e++)
        t[e] && t[e].energy > s && (s = t[e].energy);
      if (s <= 0) return { startTime: -1, beats: -1 };
      function c(e) {
        let n = 0,
          r = 1 / 0,
          a = 0;
        for (let o = 0; o < 4; o++) {
          const i = t[e + o];
          if (!i) return { ok: !1, score: -1 / 0 };
          ((n += i.energy),
            i.energy < r && (r = i.energy),
            i.energy > a && (a = i.energy));
        }
        const c = n / 4;
        if (c <= 0) return { ok: !1, score: -1 / 0 };
        const l = c / s,
          u = (a - r) / c;
        if (l < 0.45 || u > 0.7) return { ok: !1, score: -1 / 0 };
        let f = 0,
          h = 0;
        if (t[e].beatEnergies)
          for (let n = 0; n < 4; n++) {
            let r = 1 / 0,
              a = 0,
              o = 0;
            for (let i = 0; i < 4; i++) {
              const s = t[e + i].beatEnergies;
              if (!s) continue;
              const c = s[n];
              ((o += c), c < r && (r = c), c > a && (a = c));
            }
            const i = o / 4;
            i > 0 && ((f += (a - r) / i), h++);
          }
        return (
          (f = h > 0 ? f / h : 0),
          {
            ok: !0,
            score:
              ((e - i) / Math.max(1, o - i)) * 1 +
              0.5 * l -
              0.8 * u -
              0.6 * f,
          }
        );
      }
      let l = -1,
        u = -1 / 0,
        f = -1;
      for (let e = o; e >= i; e -= 4) {
        -1 === f && t[e] && (f = e);
        const { ok: n, score: r } = c(e);
        n && r > u && ((u = r), (l = e));
      }
      return (
        -1 === l && (l = f),
        -1 === l
          ? { startTime: -1, beats: -1 }
          : { startTime: t[l].startTime, beats: 16 }
      );
    })(
      _(v(t, e, 300, 6), e, n, o, 4, 0.02),
      F ?? g,
      w ?? G.lastBeat?.startTimeSeconds ?? f,
    );
    if (-1 !== i.startTime) {
      const t = {
        markerType: r.EmergencyLoop,
        startTimeSeconds: i.startTime,
        duration: 0,
        setByUser: !1,
        loopDuration: i.beats,
        enabled: !0,
      };
      ((t.color = m.Magenta),
        (t.cueType = a.ActiveLoop),
        (t.name = "S.O.S."),
        h.push(t));
    }
  }
  return (h.sort((t, e) => t.startTimeSeconds - e.startTimeSeconds), h);
}
```
