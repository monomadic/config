# VirtualDJ Example Pad XML Pages

Sampler-focused pad page examples for files stored in `Documents/VirtualDJ/Pads`.

These examples were checked against the current VirtualDJ manual plus recent forum guidance for sampler sub-pages, deck sync behavior, and custom sampler layouts.

## Page-Aware 8-Pad Sampler

Use this pattern when pads `1-8` should follow the current sampler sub-page (`1 to 8`, `9 to 16`, `17 to 24`, etc). For the visible pad label, use `sampler_pad <n>` in the `name=` field instead of `get_sample_name <n>`, otherwise the name can drift back to the absolute slot instead of following the current page.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<page name="SAMPLER PAGE-AWARE">
  <pad1 name="`sampler_loaded 1 'auto' ? sampler_pad 1 : '(empty 1)'`" color="sampler_loaded 1 'auto' ? sampler_color 1 'auto' : dim" query="sampler_play 1 'auto' ? blink 1bt : on">sampler_loaded 1 'auto' ? sampler_pad 1 'auto' : sampler_rec 1 'auto'</pad1>
  <pad2 name="`sampler_loaded 2 'auto' ? sampler_pad 2 : '(empty 2)'`" color="sampler_loaded 2 'auto' ? sampler_color 2 'auto' : dim" query="sampler_play 2 'auto' ? blink 1bt : on">sampler_loaded 2 'auto' ? sampler_pad 2 'auto' : sampler_rec 2 'auto'</pad2>
  <pad3 name="`sampler_loaded 3 'auto' ? sampler_pad 3 : '(empty 3)'`" color="sampler_loaded 3 'auto' ? sampler_color 3 'auto' : dim" query="sampler_play 3 'auto' ? blink 1bt : on">sampler_loaded 3 'auto' ? sampler_pad 3 'auto' : sampler_rec 3 'auto'</pad3>
  <pad4 name="`sampler_loaded 4 'auto' ? sampler_pad 4 : '(empty 4)'`" color="sampler_loaded 4 'auto' ? sampler_color 4 'auto' : dim" query="sampler_play 4 'auto' ? blink 1bt : on">sampler_loaded 4 'auto' ? sampler_pad 4 'auto' : sampler_rec 4 'auto'</pad4>
  <pad5 name="`sampler_loaded 5 'auto' ? sampler_pad 5 : '(empty 5)'`" color="sampler_loaded 5 'auto' ? sampler_color 5 'auto' : dim" query="sampler_play 5 'auto' ? blink 1bt : on">sampler_loaded 5 'auto' ? sampler_pad 5 'auto' : sampler_rec 5 'auto'</pad5>
  <pad6 name="`sampler_loaded 6 'auto' ? sampler_pad 6 : '(empty 6)'`" color="sampler_loaded 6 'auto' ? sampler_color 6 'auto' : dim" query="sampler_play 6 'auto' ? blink 1bt : on">sampler_loaded 6 'auto' ? sampler_pad 6 'auto' : sampler_rec 6 'auto'</pad6>
  <pad7 name="`sampler_loaded 7 'auto' ? sampler_pad 7 : '(empty 7)'`" color="sampler_loaded 7 'auto' ? sampler_color 7 'auto' : dim" query="sampler_play 7 'auto' ? blink 1bt : on">sampler_loaded 7 'auto' ? sampler_pad 7 'auto' : sampler_rec 7 'auto'</pad7>
  <pad8 name="`sampler_loaded 8 'auto' ? sampler_pad 8 : '(empty 8)'`" color="sampler_loaded 8 'auto' ? sampler_color 8 'auto' : dim" query="sampler_play 8 'auto' ? blink 1bt : on">sampler_loaded 8 'auto' ? sampler_pad 8 'auto' : sampler_rec 8 'auto'</pad8>

  <param1 name="BANK `get_sampler_bank`">sampler_bank +1</param1>
  <param2 name="PAGE `sampler_pad_page`">sampler_pad_page +1</param2>
  <menu>`sampler_options`</menu>
</page>
```

### What This Example Does

- Empty pads record into the currently visible sampler page.
- Loaded pads trigger through `sampler_pad`, so the sample's own trigger mode still applies.
- The visible pad labels come from `sampler_pad <n>`, which tracks the active sampler page more reliably than `get_sample_name <n>`.
- The pad colors follow the visible sampler page through `sampler_color`.
- `param1` cycles banks and `param2` cycles sampler sub-pages.

## Sampler Utility Page

This is a compact companion page for common sampler tasks: lock/unlock the bank, switch StemSwap on/off, change routing, change trigger mode, and trim sampler master/PFL levels.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<page name="SAMPLER UTILITY">
  <pad1 name="Lock / Unlock">sampler_options "locked"</pad1>
  <pad2 name="StemSwap">sampler_options "stemswap"</pad2>
  <pad3 name="Output">sampler_output "popup"</pad3>
  <pad4 name="Trigger Mode">sampler_mode +1</pad4>
  <pad5 name="Main +5%">sampler_volume_master +5%</pad5>
  <pad6 name="Main -5%">sampler_volume_master -5%</pad6>
  <pad7 name="HP +5%">sampler_pfl +5%</pad7>
  <pad8 name="HP -5%">sampler_pfl -5%</pad8>
</page>
```

## Useful Variations

- Always sync sampler pads to the active deck:

```text
deck active sampler_pad 1 "auto"
```

- Always sync sampler pads to the master deck:

```text
deck master sampler_pad 1 "auto"
```

- `deck master` means the current master deck, not a separate global sampler scope. If a custom skin or pad title/query path behaves differently from an explicit deck number, resolve the master deck explicitly:

```text
deck 1 masterdeck ? deck 1 sampler_pad 1 : deck 2 masterdeck ? deck 2 sampler_pad 1 : deck 3 masterdeck ? deck 3 sampler_pad 1 : deck 4 masterdeck ? deck 4 sampler_pad 1 : sampler_pad 1
```

- Page the sampler bank forward or backward from a controller or custom button:

```text
sampler_pad_page +1
sampler_pad_page -1
```

- Use the currently visible sampler pad volume instead of an absolute slot:

```text
sampler_pad_volume 1 75%
```

- Address a fixed absolute slot, regardless of the visible sampler page:

```text
sampler_volume 9 75%
get_sample_name 9
get_sample_color 9
```

- Use `sampler_pad <n>` to show the currently visible sample name in a page-aware label:

```text
sampler_pad 1
```

- Add stock stop/delete shift behavior to a sampler pad:

```xml
<shift_pad1>sampler_pad_shift 1</shift_pad1>
```

- If you need an explicit custom matrix instead of Automatic layout, staff guidance shows patterns such as:

```text
sampler_pad 1 "4x4x1"
```

## Practical Notes

- If you want deck 1 to expose `1-8` and deck 2 to expose `9-16` automatically, enable `samplerSpanAcrossDecks`.
- If you want both decks to start on `1-8`, leave `samplerSpanAcrossDecks` off and page manually with `sampler_pad_page`.
- `sampler_pad`, `sampler_color`, and `sampler_pad_volume` are the safest page-aware helpers.
- In pad `name=` fields, `sampler_pad <n>` is the safest way to show the current visible sample name on the active page.
- In recent testing, `sampler_pad <n>` has been most reliable for paged names when the deck context is explicit. If `deck master` behaves oddly, prefer an explicit deck number or an explicit `masterdeck` resolver.
- `sampler_play`, `sampler_stop`, `sampler_volume`, `get_sample_name`, and `get_sample_color` are best treated as absolute-slot helpers.

## Source Notes

- Official pads behavior: [Pads manual](https://www.virtualdj.com/manuals/virtualdj/interface/decks/decksadvanced/pads.html)
- Official sampler verbs: [VDJScript verbs](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html)
- Official sampler options: [Options list](https://www.virtualdj.com/manuals/virtualdj/appendix/optionslist.html)
- Trigger modes and loop sync details: [Sample Editor](https://www.virtualdj.com/manuals/virtualdj/editors/sampleeditor.html)
- Page-aware custom page examples: [Custom Sampler Pad Page](https://www.virtualdj.com/forums/253061/General_Discussion/Custom_Sampler_Pad_Page_%28Recording__Looping__Adjust_Beatgrid_and_more%29.html)
- Deck sync guidance: [problem with (pad pages) pads sampler sync](https://virtualdj.com/forums/224203/VirtualDJ_Technical_Support/problem_with_%28pad_pages%29_pads_sampler_sync%21_please_help___is_it_a_bug%3F%3F.html)
- Master-deck sampler quirks in newer builds: [Virtual Dj 2025 Sampler Sync](https://virtualdj.com/forums/265522/VirtualDJ_Technical_Support/Virtual_Dj_2025_Sampler_Sync.html)
- Paging and `9-16` workflow: [No longer possible to access 16 samples from controllers with 8 x 2 pads?](https://virtualdj.com/forums/261416/VirtualDJ_Technical_Support/No_longer_possible_to_access_16_samples_from_controllers_with_8_x_2_pads_.html)
- Matrix/layout hint: [Using Xone K2 to control the sampler](https://www.virtualdj.com/forums/261102/VirtualDJ_Technical_Support/Using_Xone_K2_to_control_the_sampler.html)
