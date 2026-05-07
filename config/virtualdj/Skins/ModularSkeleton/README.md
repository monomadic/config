# ModularSkeleton

Minimal reference skin demonstrating build-time modularity for VirtualDJ.

## What this is

VirtualDJ expects a flat installed skin package — a single `skin.xml` plus image files.
The official SDK does not support runtime XInclude or multi-file skin loading.

This skeleton uses **build-time XInclude** to keep source XML split into logical modules,
then flattens everything into one `build/skin.xml` before installation.
That output is what VirtualDJ sees.

## Structure

```
ModularSkeleton/
  build/          — flattened, installable output
    skin.xml      — single flat skin file (the only thing VirtualDJ reads)
    skin.png      — background image
    preview.png   — preview screenshot
  assets/         — source image assets
    skin.png
    preview.png
```

The `build/skin.xml` in this repo is the already-flattened output, kept as a working reference.
It was produced from a source tree using `xmllint --xinclude`.

## The class system

`<define class="...">` declares a reusable shape/text template. Apply it with `class="..."` on any element.

```xml
<!-- declaration -->
<define class="TRANSPORT_BUTTON" placeholders="*text,*accent">
  <size width="120" height="42"/>
  <off shape="square" radius="10" color="button_off" border="[ACCENT]" border_size="2"/>
  <on  shape="square" radius="10" color="[ACCENT]"   border="[ACCENT]" border_size="2"/>
  <text fontsize="14" weight="bold" align="center" text="[TEXT]" color="text_main" colorselected="button_text_on"/>
</define>

<!-- usage — placeholders fill [TEXT] and [ACCENT] -->
<button class="TRANSPORT_BUTTON" x="72" y="320"
        action="play_pause" query="play"
        accent="accent_1" text="PLAY"/>
```

Rules:
- `placeholders="*text,*accent"` names the placeholders (the `*` prefix marks them required).
- In the template body, placeholders appear as `[TEXT]` and `[ACCENT]` (uppercased, no asterisk).
- Usage attributes (`text=`, `accent=`) supply the values at instantiation.
- `<define color="name" value="#hex">` declares a named color usable as a value anywhere colors are accepted.
- Deck-scoped colors use `deck="1"` on the define: `<define color="accent_1" value="#63D2FF" deck="1"/>`.

## Named colors

```xml
<define color="app_bg"    value="#11151A"/>
<define color="panel_bg"  value="#1A2129"/>
<define color="text_main" value="#E7EEF7"/>
<define color="accent_1"  value="#63D2FF" deck="1"/>
<define color="accent_2"  value="#FF9C6B" deck="2"/>
```

Once defined, use the name anywhere a color value is accepted: `color="app_bg"`, `border="panel_border"`, etc.

## Build workflow

To regenerate `build/skin.xml` from a source tree with XInclude:

```bash
xmllint --xinclude --output build/skin.xml src/skin.xml
```

To install the built skin:

```bash
cp -r build/ ~/Library/Application\ Support/VirtualDJ/Skins/ModularSkeleton/
```

Or with `just` if you add a justfile:

```
install:
    xmllint --xinclude --output build/skin.xml src/skin.xml
    cp -r build/ ~/Library/Application\ Support/VirtualDJ/Skins/ModularSkeleton/
```

## Key patterns in build/skin.xml

- `<panel name="..." visible="yes">` — named panel, visible by default, switchable via `skin_panelgroup`
- `<deck deck="1">` / `<deck deck="2">` — deck-scoped containers; elements inside inherit deck context
- `<visual source="get_level" type="linear" ...>` — level meter driven by a VDJScript action
- `<button class="TRANSPORT_BUTTON" ... query="play">` — query drives the `<on>` state of the button

## Sources

- [VirtualDJ Skin SDK](https://www.virtualdj.com/wiki/Skin_SDK.html)
- [Skin Panel SDK](https://www.virtualdj.com/wiki/Skin%20SDK%20Panel.html)
- [Additional XML for Skins — forum wish thread](https://virtualdj.com/forums/248589/Wishes_and_new_features/Aditional_xml_for_Skins.html) (runtime XInclude is not supported; build-time flattening is the workaround)
