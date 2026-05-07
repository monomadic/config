# VirtualDJ Development Environment

This folder is a local reference and development environment for VirtualDJ skinning, pad pages, and VDJScript.
VirtualDJ does not publish comprehensive developer documentation; this repo fills that gap.

## Where to start

```
Reference/README.md          — index of all reference docs, source label policy, current status
Reference/VirtualDJ Reference.md — Quick Decisions guide: preferred methods, rationale, examples
Reference/VDJScript Verbs.md — curated API reference: canonical names, aliases, surfaces
Reference/Official VDJScript Coverage Audit.md — which of the 989 official verbs are still undocumented here
```

## What is here

```
Pads/                        — working pad page XML files installed from ~/Library/Application Support/VirtualDJ/Pads
Skins/                       — skin source trees and reference skins
Reference/                   — Markdown documentation
```

## Key facts for AI agents

- VirtualDJ skins are XML. The scripting language inside attributes is **VDJScript**.
- `action=""` attributes take VDJScript actions. `query=""` takes a boolean/value expression.
- `&` chains actions in XML attributes and must be written `&amp;` inside XML.
- Backtick-wrapped expressions (`` `verb` ``) evaluate and return a value in string/color contexts.
- Pad pages live in `Pads/*.xml`. Skins live in `Skins/*/`.
- The official VDJScript verb appendix has 989 entries. This repo documents ~618 of them.
- Source labels (`Official`, `Official forum`, `Community`, `Published skin`, `Local test`, `Inference`) appear throughout the reference docs and indicate how reliable each claim is.

## Preferred patterns (quick version)

- **Slot FX**: `effect_select <slot> 'Name'`, `effect_slider <slot> <param> <value>`, `effect_active <slot>` — not name-based toggling.
- **ColorFX**: `filter_selectcolorfx 'Name'` + `filter` for the main deck filter knob. `effect_colorfx <1-4>` + `effect_colorslider` for extra dedicated controls.
- **Sampler (page-aware)**: `sampler_pad <n>`, `sampler_color <n> 'auto'`, `sampler_loaded <n> 'auto'`.
- **Sampler (fixed slot)**: `sampler_play <n>`, `get_sample_name <n>`, `get_sample_color <n>`.
- **Panels**: `<panel visibility="...">` for query-driven; `name=""` + `skin_panelgroup` for persistent manual switching.
- **Dynamic text color**: one `<text color="`action`">`, not per-state color attributes.
- **Dynamic border color**: not supported (CTO confirmed). Use fill or background instead.
- **Time mode**: `display_time 'remain,elapsed'` + `get_time`, not custom skin vars.

## Working examples in this repo

| File | Demonstrates |
| --- | --- |
| [Pads/Reference - Slot FX.xml](Pads/Reference%20-%20Slot%20FX.xml) | Canonical slot-based audio FX pads |
| [Pads/Reference - ColorFX.xml](Pads/Reference%20-%20ColorFX.xml) | Canonical filter + ColorFX selection |
| [Pads/Reference - Page Aware Sampler.xml](Pads/Reference%20-%20Page%20Aware%20Sampler.xml) | Page-aware sampler labels, colors, actions |
| [Pads/2. EFFECTS.xml](Pads/2.%20EFFECTS.xml) | Name-based FX toggle page with stems routing |
| [Pads/2. COLOR FX.xml](Pads/2.%20COLOR%20FX.xml) | ColorFX selection with stems context |
| [Skins/ModularSkeleton/build/skin.xml](Skins/ModularSkeleton/build/skin.xml) | Minimal modular skin scaffold |
| [Skins/Haunting Pro Edit/Pro.xml](Skins/Haunting%20Pro%20Edit/Pro.xml) | Published full-size skin for reference |

## macOS paths

| Path | Content |
| --- | --- |
| `~/Library/Application Support/VirtualDJ/Pads/` | Installed pad pages |
| `~/Library/Application Support/VirtualDJ/Skins/` | Installed skins |
| `~/Library/Application Support/VirtualDJ/Mappers/` | Controller/keyboard mappings |
| `~/Library/Application Support/VirtualDJ/database.xml` | Main track database |

See [Reference/Application Internals.md](Reference/Application%20Internals.md) for the full path map, database structure, and stem sidecar layout.
