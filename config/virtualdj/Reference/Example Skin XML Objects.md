# Example Skin XML Objects

Sampler-focused skin XML examples for building custom panels in a VirtualDJ skin.

## Full Sampler Panel Chunk

This chunk is designed to be pasted inside a deck section such as `<deck deck="left">...</deck>`. It gives you:

- Current sampler bank display
- Current sampler page display (`1 to 8`, `9 to 16`, etc.)
- Bank previous/next buttons
- Page previous/next buttons
- Mode and options buttons
- 8 visible sampler pads that follow the current sampler page
- Empty-pad recording behavior similar to the stock sampler pad page

Use this when you want a skin-native sampler panel instead of relying on the SideView or the deck Pads area.

For the visible sample labels, this chunk uses `sampler_pad <n>` inside the text `format=` fields instead of `get_sample_name <n>`, so the text follows the current sampler page.

```xml
<deck deck="left">
  <panel name="sampler_panel_left" visible="yes">
    <group name="sampler_header" x="24" y="24">
      <textzone>
        <pos x="+0" y="+0"/>
        <size width="220" height="20"/>
        <text font="Arial" fontsize="14" weight="bold" color="#D7DCE4" text="SAMPLER"/>
      </textzone>

      <textzone>
        <pos x="+0" y="+24"/>
        <size width="220" height="22"/>
        <text font="Arial" fontsize="15" weight="bold" color="white" format="`get_sampler_bank`"/>
      </textzone>

      <textzone>
        <pos x="+240" y="+24"/>
        <size width="120" height="22"/>
        <text font="Arial" fontsize="13" align="center" color="#AEB6C2" format="PAGE `sampler_pad_page`"/>
      </textzone>

      <button action="sampler_bank -1">
        <pos x="+0" y="+56"/>
        <size width="46" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="BANK -" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>

      <button action="sampler_bank +1">
        <pos x="+52" y="+56"/>
        <size width="46" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="BANK +" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>

      <button action="sampler_pad_page -1">
        <pos x="+240" y="+56"/>
        <size width="56" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="PAGE -" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>

      <button action="sampler_pad_page +1">
        <pos x="+302" y="+56"/>
        <size width="56" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="PAGE +" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>

      <button action="sampler_mode +1">
        <pos x="+380" y="+24"/>
        <size width="74" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text fontsize="11" align="center" weight="bold" color="#D7DCE4" format="MODE `sampler_mode`"/>
      </button>

      <button action="sampler_options">
        <pos x="+460" y="+24"/>
        <size width="76" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="OPTIONS" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>

      <button action="sampler_output 'popup'">
        <pos x="+542" y="+24"/>
        <size width="70" height="26"/>
        <up radius="4" border="#4B5563" border_size="1" color="#1F232B"/>
        <over radius="4" border="#6B7280" border_size="1" color="#2A3140"/>
        <down radius="4" border="#6B7280" border_size="1" color="#394559"/>
        <text text="OUTPUT" align="center" fontsize="11" weight="bold" color="#D7DCE4"/>
      </button>
    </group>

    <group name="sampler_grid" x="24" y="126">
      <button action="sampler_loaded 1 'auto' ? sampler_pad 1 'auto' : sampler_rec 1 'auto'" query="sampler_play 1 'auto'">
        <pos x="+0" y="+0"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+8" y="+10"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 1 'auto' ? sampler_color 1 'auto' : color '#8993A1'`" format="`sampler_loaded 1 'auto' ? sampler_pad 1 : '(empty 1)'`"/>
      </textzone>

      <button action="sampler_loaded 2 'auto' ? sampler_pad 2 'auto' : sampler_rec 2 'auto'" query="sampler_play 2 'auto'">
        <pos x="+156" y="+0"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+164" y="+10"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 2 'auto' ? sampler_color 2 'auto' : color '#8993A1'`" format="`sampler_loaded 2 'auto' ? sampler_pad 2 : '(empty 2)'`"/>
      </textzone>

      <button action="sampler_loaded 3 'auto' ? sampler_pad 3 'auto' : sampler_rec 3 'auto'" query="sampler_play 3 'auto'">
        <pos x="+312" y="+0"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+320" y="+10"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 3 'auto' ? sampler_color 3 'auto' : color '#8993A1'`" format="`sampler_loaded 3 'auto' ? sampler_pad 3 : '(empty 3)'`"/>
      </textzone>

      <button action="sampler_loaded 4 'auto' ? sampler_pad 4 'auto' : sampler_rec 4 'auto'" query="sampler_play 4 'auto'">
        <pos x="+468" y="+0"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+476" y="+10"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 4 'auto' ? sampler_color 4 'auto' : color '#8993A1'`" format="`sampler_loaded 4 'auto' ? sampler_pad 4 : '(empty 4)'`"/>
      </textzone>

      <button action="sampler_loaded 5 'auto' ? sampler_pad 5 'auto' : sampler_rec 5 'auto'" query="sampler_play 5 'auto'">
        <pos x="+0" y="+76"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+8" y="+86"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 5 'auto' ? sampler_color 5 'auto' : color '#8993A1'`" format="`sampler_loaded 5 'auto' ? sampler_pad 5 : '(empty 5)'`"/>
      </textzone>

      <button action="sampler_loaded 6 'auto' ? sampler_pad 6 'auto' : sampler_rec 6 'auto'" query="sampler_play 6 'auto'">
        <pos x="+156" y="+76"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+164" y="+86"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 6 'auto' ? sampler_color 6 'auto' : color '#8993A1'`" format="`sampler_loaded 6 'auto' ? sampler_pad 6 : '(empty 6)'`"/>
      </textzone>

      <button action="sampler_loaded 7 'auto' ? sampler_pad 7 'auto' : sampler_rec 7 'auto'" query="sampler_play 7 'auto'">
        <pos x="+312" y="+76"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+320" y="+86"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 7 'auto' ? sampler_color 7 'auto' : color '#8993A1'`" format="`sampler_loaded 7 'auto' ? sampler_pad 7 : '(empty 7)'`"/>
      </textzone>

      <button action="sampler_loaded 8 'auto' ? sampler_pad 8 'auto' : sampler_rec 8 'auto'" query="sampler_play 8 'auto'">
        <pos x="+468" y="+76"/>
        <size width="146" height="64"/>
        <off radius="6" border="#3E4552" border_size="1" color="#171A20"/>
        <on radius="6" border="#6A7382" border_size="1" color="#253041"/>
        <over radius="6" border="#7A8597" border_size="1" color="#2D394D"/>
        <down radius="6" border="#7A8597" border_size="1" color="#41506A"/>
      </button>
      <textzone>
        <pos x="+476" y="+86"/>
        <size width="130" height="42"/>
        <text font="Arial" fontsize="13" multiline="true" align="center" valign="center" color="`sampler_loaded 8 'auto' ? sampler_color 8 'auto' : color '#8993A1'`" format="`sampler_loaded 8 'auto' ? sampler_pad 8 : '(empty 8)'`"/>
      </textzone>
    </group>
  </panel>
</deck>
```

## How This Works

- The chunk is deck-scoped, so sampler paging and sync stay tied to the deck that contains it.
- `sampler_pad 1 'auto'` through `sampler_pad 8 'auto'` follow the current sampler page instead of always hard-targeting slots `1-8`.
- `sampler_pad_page -1` and `sampler_pad_page +1` move through sample ranges such as `1 to 8`, `9 to 16`, and later pages.
- Empty pads call `sampler_rec n 'auto'` so unlocked banks can record directly from the skin panel, matching the stock sampler workflow.
- The visible pad labels come from `sampler_pad <n>` in text `format=` fields, so the labels follow the current sampler page.
- `query="sampler_play n 'auto'"` lights the pad while that visible sample is playing.

## Variations

### Make the Chunk Global Instead of Deck-Scoped

If you move this chunk outside a `<deck>` block, make the pad actions explicit so sync behavior is predictable:

```text
deck active sampler_pad 1 "auto"
deck master sampler_pad 1 "auto"
```

- Use `deck active` if the sampler should follow the selected/working deck.
- Use `deck master` if synced samples should always lock to the current master deck.

### Follow the Current Master Deck Explicitly

If a skin panel needs to follow the current master deck and raw `deck master` behaves oddly in sampler title or page text, resolve the master deck number explicitly instead of relying on the alias:

```xml
<textzone>
  <size width="220" height="22"/>
  <text font="Arial" fontsize="13" color="#D7DCE4" format="`deck 1 masterdeck ? deck 1 sampler_pad 1 : deck 2 masterdeck ? deck 2 sampler_pad 1 : deck 3 masterdeck ? deck 3 sampler_pad 1 : deck 4 masterdeck ? deck 4 sampler_pad 1 : sampler_pad 1`"/>
</textzone>

<button action="deck 1 masterdeck ? deck 1 sampler_pad_page +1 : deck 2 masterdeck ? deck 2 sampler_pad_page +1 : deck 3 masterdeck ? deck 3 sampler_pad_page +1 : deck 4 masterdeck ? deck 4 sampler_pad_page +1 : sampler_pad_page +1">
  <size width="56" height="26"/>
</button>
```

This is heavier than raw `deck master`, but it avoids the alias in the exact place where sampler title/page text has proven less reliable in recent testing.

### Add a Progress Bar to Each Pad

If you want a progress bar, add a `visual` after each button:

```xml
<visual source="sampler_position 1" type="linear" orientation="horizontal">
  <pos x="+8" y="+54"/>
  <size width="130" height="4"/>
  <off shape="square" color="#0D0F13"/>
  <on shape="square" color="#4E6FA8"/>
</visual>
```

Place it in the same group as the pad and adjust the coordinates to match the pad location.

### Split `1-8` and `9-16` Across Two Deck Panels

If you want the classic two-panel workflow:

- Leave the left deck panel on the default `1 to 8`.
- Put the same chunk on the right deck and either:
  - set `samplerSpanAcrossDecks=yes`, or
  - trigger `sampler_pad_page +1` once on startup/user action for the right side.

## Practical Notes

- `sampler_pad`, `sampler_loaded`, and `sampler_color` are the safest page-aware helpers for skin-based sampler panels.
- In skin text `format=` fields, `sampler_pad <n>` is the safest way to show the currently visible sample name on the active sampler page.
- `deck master` means the current master deck, not a separate global sampler scope. In some sampler title/query paths, an explicit `deck 1 masterdeck ? ... : deck 2 masterdeck ? ...` resolver is more reliable than raw `deck master`.
- `sampler_loaded <n>` is the cleanest way to drive row visibility or empty-slot logic around that page-aware display pattern, while `sampler_rec <n> 'auto'` is still useful for direct recording into the visible slot.
- If you need fixed absolute slots regardless of the current page, swap `sampler_pad` for `sampler_play`, and swap page-aware name/color helpers for absolute ones such as `get_sample_name 9` and `get_sample_color 9`.
- Duplicate this chunk under `<deck deck="right">` if you want a second, independently positioned sampler panel.
- Class names are matched case-insensitively in practice. A good convention is `class="SAMPLER_ROW"` in `<define>` and `class="sampler_row"` at the call site, while keeping placeholder tokens uppercase, e.g. `[INDEX]`.

## Source Notes

- Official Skin SDK overview: [Skin SDK](https://virtualdj.com/wiki/skinsdk.html)
- Official panel syntax: [Skin SDK Panel](https://www.virtualdj.com/wiki/Skin%20SDK%20Panel.html)
- Official textzone syntax: [Skin SDK Textzone](https://www.virtualdj.com/wiki/Skin%20SDK%20Textzone.html)
- Official sampler verbs: [VDJScript verbs](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html)
- Official sampler pad behavior: [Pads manual](https://www.virtualdj.com/manuals/virtualdj/interface/decks/decksadvanced/pads.html)
- Page-aware custom sampler pad patterns: [Custom Sampler Pad Page](https://www.virtualdj.com/forums/253061/General_Discussion/Custom_Sampler_Pad_Page_%28Recording__Looping__Adjust_Beatgrid_and_more%29.html)
- Skin-side sampler naming/color examples: [How to read the sample icon in a button?](https://virtualdj.com/forums/258227/VirtualDJ_Skins/How_to_read_the_sample_icon_in_a_button%3F.html)
- Deck-sync guidance for sampler pads: [problem with (pad pages) pads sampler sync!](https://virtualdj.com/forums/224203/VirtualDJ_Technical_Support/problem_with_%28pad_pages%29_pads_sampler_sync%21_please_help___is_it_a_bug%3F%3F.html)
- Master-deck sampler quirks in newer builds: [Virtual Dj 2025 Sampler Sync](https://virtualdj.com/forums/265522/VirtualDJ_Technical_Support/Virtual_Dj_2025_Sampler_Sync.html)
