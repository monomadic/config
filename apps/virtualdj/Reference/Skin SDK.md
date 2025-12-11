# VirtualDJ Skin SDK Reference

Comprehensive reference for creating and modifying VirtualDJ 8+ skins.

## Overview

A VirtualDJ skin is a `.zip` file containing:
- `skin.xml` - The XML file defining the skin structure
- `skin.png` - The graphics file (or same name as XML)
- `window_images.png` - Optional graphics for window elements

## The `<skin>` Element

The root element of every skin with these attributes:

| Attribute | Description | Values |
|-----------|-------------|--------|
| `name` | Skin name (can differ from filename) | text |
| `version` | SDK version | 8 for VirtualDJ 8+ |
| `width` | Skin width in pixels | number |
| `height` | Skin height in pixels | number |
| `nbdecks` | Number of decks | 2, 4, etc (optional) |
| `comment` | Extra information | text (optional) |
| `author` | Author name | text (optional) |
| `image` | Graphics filename | filename (optional if matches XML name) |
| `preview` | Preview screenshot | filename (optional) |

### Breaklines

Optional `<breakline>` elements define y-coordinates for browser stretching:
```xml
<breakline y="300"/>
<breakline y="800"/>
```

## Skin Children Elements

Available elements as children of `<skin>`:

### Containers
- `<deck>` - Groups elements for a specific deck
- `<panel>` - Container for show/hide element groups
- `<group>` - Generic container for organizing elements
- `<stack>` - Container with fadein/fadeout effects

### Interactive Elements
- `<button>` - Clickable button
- `<slider>` - Movable slider control
- `<switch>` - Multi-state toggle
- `<knob>` - Rotary knob control

### Display Elements
- `<visual>` - Static/dynamic graphics display
- `<textzone>` - Text display area
- `<video>` - Video output display
- `<led>` - LED indicator
- `<equalizer>` - Spectrum analyzer

### Window Elements
- `<window>` - Popup/separate window
- `<browser>` - File browser interface

### Simple Shapes
- `<square>` - Rectangle/rounded rectangle
- `<circle>` - Circle/ellipse
- `<line>` - Line drawing

### Special Elements
- `<define>` - Define reusable element templates

---

## Element Details

### `<deck>`

Groups elements for a specific deck without adding `deck="x"` to each child.

**Syntax:** `<deck deck="">`

**Attributes:**
- `deck` - Define the deck: `"1|2|3|4"`, `"left|right"`, `"leftvideo|rightvideo"`, `"master"`, `"default"`

**Children:** Any skin element

**Example:**
```xml
<deck deck="left">
    <button action="play_pause">
        <pos x="100" y="100"/>
        <size width="80" height="45"/>
    </button>
</deck>
```

---

### `<panel>`

Container for grouping elements that can be shown/hidden together. Panels are very useful for switching between groups on the same screen position with buttons or shortcuts.

**Syntax:** `<panel visible="" name="" group="" visibility="">`

**Attributes:**
- `visible=""` or `visibility=""` - Initial visibility: `"yes"` (shown at start) or `"no"` (hidden). Can also be a VDJScript action returning true/false for dynamic visibility
- `name=""` - Panel identifier. Elements with matching `panel=""` attribute belong to this panel. Name also stores visibility state across sessions
- `group=""` - (optional) Group name. Only one panel from a common group can be shown at a time. Showing a panel hides others in the same group

**Children:** Any skin element

**Usage Patterns:**

**1. Visibility-based (Automatic):**
Elements automatically show/hide based on VDJScript conditions.
```xml
<panel visibility="loop">
    <!-- Displayed only when deck is in loop -->
    <button action="loop_exit">
        <pos x="100" y="100"/>
        <size width="80" height="45"/>
    </button>
</panel>

<panel visibility="not loop">
    <!-- Displayed when deck is not in loop -->
    <button action="loop 4">
        <pos x="100" y="100"/>
        <size width="80" height="45"/>
    </button>
</panel>
```

**2. Name/Group-based (Manual):**
User manually switches between panels using buttons or shortcuts. Current state persists across sessions.
```xml
<panel group="loops" name="autoloops" visible="yes">
    <!-- Auto loop buttons (shown by default) -->
    <button action="loop 1">
        <pos x="100" y="100"/>
    </button>
    <button action="loop 2">
        <pos x="180" y="100"/>
    </button>
</panel>

<panel group="loops" name="manualloops" visible="no">
    <!-- Manual loop buttons (hidden by default) -->
    <button action="loop_in">
        <pos x="100" y="100"/>
    </button>
    <button action="loop_out">
        <pos x="180" y="100"/>
    </button>
</panel>

<!-- Button to toggle between panels -->
<button action="skin_panelgroup 'loops' 'autoloops'">
    <pos x="100" y="50"/>
    <text text="Auto"/>
</button>
<button action="skin_panelgroup 'loops' 'manualloops'">
    <pos x="180" y="50"/>
    <text text="Manual"/>
</button>
```

**Performance Tip:** If multiple elements share the same visibility condition, nest them in a single `<panel>` instead of adding `visibility=""` to each element individually.

---

### `<stack>`

Container that displays multiple items with smooth fade transitions. Shows only the last N visible items based on number of slots. Perfect for temporary notifications and context-sensitive UI panels that appear/disappear with visual feedback.

**Syntax:** `<stack fadein="" fadeout="">`

**Attributes:**
- `fadein=""` - (optional) Time in ms for items to fade in from nothing to full display. Example: `fadein="200ms"`
- `fadeout=""` - (optional) Time in ms for items to fade out from full display to nothing. Example: `fadeout="500ms"`

**Children:**
- `<size width="" height=""/>` - Define width and height of each slot
- `<slot x="" y="">` - Multiple slots allowed. Each slot has x/y position parameters defining where items appear
- `<item>` - Multiple items allowed (typically more items than slots)
  - `visibility=""` - VDJScript condition determining when item should be visible
  - `class=""` - Reference to a defined class
  - Any skin element can be nested inside `<item></item>`

**How it Works:**
- Stack displays only the last N "visible" items (where N = number of slots)
- Items appear in slots based on their visibility conditions
- When an item becomes visible, it fades in
- When an item becomes invisible, it fades out
- Perfect for stacking temporary UI feedback messages

**Example:**
```xml
<stack fadein="200ms" fadeout="500ms">
    <size width="370" height="170"/>
    
    <!-- Define 3 slots (bottom to top) -->
    <slot x="-370" y="170+20+170+20+170" />  <!-- Bottom slot -->
    <slot x="-370" y="170+20+170"/>          <!-- Middle slot -->
    <slot x="-370" y="170"/>                 <!-- Top slot -->
    
    <!-- Define items (panels that can appear in slots) -->
    <item class="looppanel" visibility="is_using 'loop' 8000ms"></item>
    <item class="eqpanel" visibility="is_using 'equalizer' 1000ms"></item>
    <item class="filterpanel" visibility="is_using 'filter' 1000ms"></item>
    <item class="cuepanel" visibility="is_using 'cue' 1000ms"></item>
    <item class="samplerpanel" visibility="is_using 'sample' 1000ms 8000ms"></item>
    <item class="fxpanel" visibility="is_using 'effect' 1000ms 8000ms"></item>
    <item class="padspanel" visibility="is_using 'pads' 1000ms"></item>
    <item class="nexttrackpanel" visibility="is_using 'load' 5000ms"></item>
</stack>
```

**Common Use Cases:**
- Temporary feedback panels ("Loop Active", "Effect Engaged", "Track Loading")
- Context-sensitive control panels that appear when using specific features
- Status notifications that stack and auto-dismiss
- Progressive disclosure UI where multiple contextual panels may be visible simultaneously

---

### `<define>`

Define reusable element templates to avoid repetition.

**Syntax:** `<define class="" classdeck="" [element attributes]>`

**Attributes:**
- `class` - Template name (e.g., `"small_button"`)
- `classdeck` - Optional deck specification
- Plus any attributes of the element being defined

**Children:** Children of the element being defined

**Example:**
```xml
<!-- Define a button template -->
<define class="mybutton" classdeck="left">
    <size height="45" width="80"/>
    <on x="100" y="125"/>
    <off x="100" y="170"/>
    <over x="100" y="215"/>
</define>

<!-- Use the template -->
<button class="mybutton" action="play_pause">
    <pos x="100" y="50"/>
</button>
```

**Color Defines:**
```xml
<define color="deckcolorbright" value="#1e7b96" deck="1"/>
<define color="deckcolorbright" value="#b73841" deck="2"/>
```

**Placeholders:** Use `$1`, `$2`, etc. in defines and pass values when using:
```xml
<define class="mytext">
    <text text="$1 - $2" color="$3"/>
</define>

<textzone class="mytext" $1="Artist" $2="Title" $3="white">
    <pos x="100" y="50"/>
</textzone>
```

---

### `<button>`

Clickable button with multiple states and support for image graphics or vector shapes. Buttons can have text/icon overlays and support various mouse interactions.

**Syntax:** `<button action="" leftclick="" middleclick="" rightclick="" dblclick="" query="">`

**Attributes:**
- `action=""` - VDJScript action performed on button press (default click)
- `leftclick=""` - Different action for left mouse button
- `middleclick=""` - Different action for middle mouse button  
- `rightclick=""` - Different action for right mouse button
- `dblclick=""` - Different action for double-click
- `query=""` - VDJScript query that enables `<on>` graphics when true (alternative to `action` for state determination)

**Children:**
- `<tooltip>` - Tooltip text (supports `\n` for multiple lines)
- `<pos x="" y=""/>` - Position on screen
- `<size width="" height=""/>` - Button dimensions

**Graphics States (Image-based):**
Reference coordinates in skin.png file:
- `<up x="" y=""/>` - Default up state  
- `<down x="" y=""/>` - Pressed state
- `<on x="" y=""/>` - Active/enabled state
- `<off x="" y=""/>` - Inactive/disabled state
- `<over x="" y=""/>` - Mouse hover state
- `<selected x="" y=""/>` - Selected state
- `<downselected x="" y=""/>` - Pressed while selected
- `<overselected x="" y=""/>` - Hover while selected

**Graphics States (Vector-based):**
Draw buttons with code instead of referencing PNG coordinates. Attributes:
- `shape=""` - `"square"` (default) or `"circle"`
- `color=""` - Fill color (hex, RGB, or named)
- `border=""` - Border color
- `border_size=""` - Border thickness in pixels
- `radius=""` - Corner radius for rounded corners
- `gradient=""` - `"horizontal"`, `"vertical"`, or `"circular"` (requires `color2`)
- `color2=""` - End color for gradient (start color is `color`)

**Drawing and Mouse Masks:**
- `<clipmask x="" y=""/>` - B&W graphic used as clip mask (avoid if possible - use transparent PNG)
- `<mousemask x="" y=""/>` - B&W graphic mask to determine if mouse is over button

**Text Overlays:**
- `<text>` - Text overlay on button (see textzone for full attributes)
- `<textover>` - Text when mouse is over
- `<textdown>` - Text when button is pressed
- `<textselected>` - Text when button is selected

**Icon Overlays:**
- `<icon>` - Icon overlay (can use custom or system icons)

**Example 1: Image-based Button**
```xml
<button action="loop">
    <pos x="125" y="220"/>
    <size width="70" height="44"/>
    <up x="120" y="1890" />
    <over x="120" y="1990" />
    <down x="120" y="2090" />
    <selected x="120" y="2190" />
    <tooltip>Loop 4 beats</tooltip>
</button>
```

**Example 2: Vector Graphics Button**
```xml
<button action="loop">
    <pos x="125" y="220"/>
    <size width="70" height="44"/>
    <up radius="6" border_size="2" border="black" color="#2F3034" />
    <over radius="6" border_size="2" border="black" color="#2C3B47" />
    <down radius="6" border_size="2" border="black" color="#1287E0"/>
    <selected radius="6" border_size="2" border="black" color="#1287E0"/>
    <text size="15" color="#909090" align="center" weight="bold" text="LOOP"/>
    <textover size="15" color="#BBBBBB" align="center" weight="bold" text="LOOP"/>
    <textdown size="15" color="white" align="center" weight="bold" text="LOOP"/>
    <textselected size="15" color="white" align="center" weight="bold" text="LOOP"/>
</button>
```

**Example 3: Button with Gradient**
```xml
<button action="play_pause">
    <pos x="100" y="100"/>
    <size width="80" height="45"/>
    <off radius="4" border="black" border_size="1" 
         color="#404040" color2="#202020" gradient="vertical"/>
    <on radius="4" border="black" border_size="1" 
        color="#00FF00" color2="#008800" gradient="vertical"/>
</button>
```

---

### `<slider>` and `<knob>`

Movable slider control for faders, knobs, and other continuous value adjustments. Sliders can be horizontal, vertical, or circular (knobs).

**Syntax:** `<slider action="" dblclick="" rightclick="" orientation="" direction="" frommiddle="" relative="">`

**Attributes:**
- `action=""` - VDJScript action performed by the slider
- `leftclick=""` - Different action for left mouse button
- `rightclick=""` - Different action for right mouse button
- `dblclick=""` - Different action for double-click
- `orientation=""` - Slider type:
  - `"horizontal"` - Horizontal slider (default)
  - `"vertical"` - Vertical slider
  - `"circle"` or `"round"` - Circular slider/knob
- `direction=""` - Movement direction: `"normal"` or `"reversed"`
- `frommiddle=""` - `"true"` to split graphics at midpoint (useful for EQ knobs that go ±)
- `relative=""` - `"yes"` for relative movement, `"no"` for absolute positioning

**Children:**
- `<pos x="" y=""/>` - Position on screen
- `<size width="" height=""/>` - Slider dimensions (defines range)

**Linear Slider Graphics:**
- `<off>` or `<background>` - Background/track graphics (image or vector)
- `<on>` or `<fill>` - Fill/progress indicator (image or vector)
- `<fader>` or `<cursor>` - Moving handle/cursor (image or vector)
  - Has its own `<size>` if different from slider size
  - Can have `<off>`, `<over>` states
- `<over>` - Slider background when mouse is over

**Round Slider/Knob Graphics:**
- `<off>` - Knob background (circle shape with vector graphics)
- `<fader>` - Moving indicator/pointer
  - `anglemin=""` - Start angle in degrees (e.g., `-150`)
  - `anglemax=""` - End angle in degrees (e.g., `150`)
  - `color=""` - Indicator color
  - `width=""` - Indicator width
  - `height=""` - Indicator length/height
  - `radius=""` - Corner radius for indicator
- `<fill>` - (for round sliders only) Ring that shows value
  - `<off x="" y=""/>` - Ring graphic at 0%
  - `<on x="" y=""/>` - Ring graphic at 100%

**Mouse Control:**
- `<mouserect x="" y="" width="" height=""/>` - Define mouse-sensitive area (if different from slider size)

**Example 1: Vertical Fader (Vector Graphics)**
```xml
<slider action="level" rightclick="temporary" orientation="vertical">
    <pos x="23" y="100"/>
    <size width="6" height="124"/>
    <!-- Background track -->
    <off height="-21" color="faderinoff" shape="square" 
         border="darker" border_size="1" radius="3"/>
    <!-- Fill indicator -->
    <on height="-21" color="faderin" shape="square" 
        border="darker" border_size="1" radius="3"/>
    <!-- Mouse sensitive area (wider than visual) -->
    <mouserect x="-20" y="0" width="40" height="120"/>
    <!-- Moving fader handle -->
    <fader>
        <size width="40" height="21"/>
        <off x="236" y="266"/>
    </fader>
</slider>
```

**Example 2: Horizontal Fader (Image Graphics)**
```xml
<slider action="crossfader" orientation="horizontal">
    <pos x="400" y="600"/>
    <size width="300" height="30"/>
    <background x="0" y="800"/>
    <cursor>
        <size width="40" height="40"/>
        <off x="340" y="800"/>
        <over x="380" y="800"/>
    </cursor>
</slider>
```

**Example 3: Round Knob (EQ-style)**
```xml
<slider action="eq_high" frommiddle="true" orientation="round" relative="no">
    <pos x="400" y="200"/>
    <size width="48" height="48"/>
    <!-- Knob body (vector circle) -->
    <off width="40" height="40" shape="circle" 
         color="#3a3b3e" color2="#252628" gradient="vertical" 
         border="#1e1e20" border_size="2"/>
    <!-- Rotating indicator line -->
    <fader color="#aaaaaa" width="3" height="17" radius="2" 
           anglemin="-150" anglemax="150"/>
</slider>
```

**Example 4: Round Knob with Ring Fill**
```xml
<slider action="filter" orientation="round">
    <pos x="500" y="200"/>
    <size width="60" height="60"/>
    <!-- Knob background -->
    <off width="50" height="50" shape="circle" color="#333333"/>
    <!-- Moving indicator -->
    <fader color="white" width="4" height="20" 
           anglemin="-140" anglemax="140"/>
    <!-- Progress ring -->
    <fill>
        <off x="0" y="900"/>   <!-- Empty ring graphic -->
        <on x="60" y="900"/>   <!-- Full ring graphic -->
    </fill>
</slider>
```

**Notes:**
- **Knobs** are just sliders with `orientation="round"` or `orientation="circle"`
- Use `frommiddle="true"` for EQ-style knobs that adjust from center position
- `anglemin` and `anglemax` define the rotation range (typically -150° to +150° for 300° total rotation)
- Linear sliders can use vector graphics (`shape`, `color`, `border`) or image references (`x`, `y`)
- For better mouse control, use `<mouserect>` to define a larger hit area than the visual slider

---

### `<visual>`

Display zone for static graphics or dynamic visual feedback. Visuals change their display based on various data sources to reflect deck status, volume levels, position, etc.

**Syntax:** `<visual source="" type="" orientation="" direction="" granularity="">`

**Attributes:**
- `source=""` - Data source driving the visual:
  - `"beat"` - Beat intensity
  - `"rotation"` - Disc rotation angle (depends on position and RPM speed)
  - `"arm"` - Turntable arm position (moves on PLAY and PAUSE)
  - `"volume"` - Volume level (depends on crossfader and level values)
  - `"position"` - Position in song
  - Any `get_*` VDJScript action that returns a numeric value (e.g., `"get_level"`, `"get_bpm"`)

- `type=""` - Display mode:
  - `"onoff"` - Binary on/off display (shows `<on>` graphic if source≥2048, `<off>` if source<2048)
  - `"linear"` - Smooth progression between `<off>` and `<on>` graphics
  - `"color"` - Solid color display (no graphics files needed)
  - `"custom"` - Custom display mode

- `orientation=""` - Direction of progression:
  - `"horizontal"` - Left to right
  - `"vertical"` - Bottom to top

- `direction=""` - Alternative specification for progression:
  - `"left"` - Progress from left
  - `"right"` - Progress from right
  - `"up"` - Progress from bottom up
  - `"down"` - Progress from top down

- `granularity=""` - (for type=linear) Number of sections to divide visual into instead of smooth progression. Useful for VU-meters with discrete segments

**Children:**
- `<pos x="" y=""/>` - Position on screen
- `<size width="" height=""/>` - Visual dimensions
- `<clipmask x="" y=""/>` - (optional) B&W graphic used as clip mask for drawing
- `<off x="" y=""/>` - (all types except "custom") Graphic for low/minimum value state
- `<on x="" y=""/>` - (all types except "custom") Graphic for high/maximum value state

**How Linear Visuals Work:**
The visual progressively reveals the `<on>` graphic as the source value increases, creating smooth transitions for meters and progress indicators.

**Example 1: Volume Meter (Vertical Linear)**
```xml
<visual source="volume" type="linear" orientation="vertical">
    <pos x="100" y="100"/>
    <size width="30" height="200"/>
    <off x="0" y="300"/>   <!-- Empty/low volume graphic -->
    <on x="30" y="300"/>   <!-- Full/high volume graphic -->
</visual>
```

**Example 2: Beat Intensity Indicator (On/Off)**
```xml
<visual source="beat" type="onoff">
    <pos x="50" y="50"/>
    <size width="40" height="40"/>
    <off x="0" y="100"/>   <!-- No beat graphic -->
    <on x="40" y="100"/>   <!-- Beat active graphic -->
</visual>
```

**Example 3: Song Position (Horizontal Linear)**
```xml
<visual source="position" type="linear" orientation="horizontal">
    <pos x="100" y="500"/>
    <size width="800" height="20"/>
    <off x="0" y="600"/>   <!-- Start position graphic -->
    <on x="0" y="620"/>    <!-- End position graphic -->
</visual>
```

**Example 4: VU Meter with Discrete Segments**
```xml
<visual source="get_vu_meter" type="linear" orientation="vertical" granularity="10">
    <pos x="50" y="100"/>
    <size width="20" height="200"/>
    <off x="0" y="400"/>
    <on x="20" y="400"/>
</visual>
```

**Example 5: Rotation Disc Visual**
```xml
<visual source="rotation" type="linear" orientation="horizontal">
    <pos x="200" y="200"/>
    <size width="300" height="300"/>
    <off x="0" y="800"/>   <!-- Disc at 0° rotation -->
    <on x="300" y="800"/>  <!-- Full rotation graphic -->
</visual>
```

**Example 6: Using VDJScript Query as Source**
```xml
<visual source="`get_level`" type="linear" orientation="vertical">
    <pos x="100" y="100"/>
    <size width="40" height="150"/>
    <off x="0" y="500"/>
    <on x="40" y="500"/>
</visual>
```

**Common Use Cases:**
- VU meters showing audio levels
- Progress bars for song position
- Beat flash indicators
- Vinyl rotation displays
- Crossfader position indicators
- Volume level meters
- Effect wet/dry indicators

**Notes:**
- For `type="onoff"`, the threshold value is 2048 (half of 4096, the typical maximum)
- `type="linear"` provides smooth transitions proportional to the source value
- `granularity` creates stepped/segmented displays instead of smooth
- Can use clipmasks for complex shaped meters

---

### `<textzone>`

Display area for static or dynamic text.

**Syntax:** `<textzone deck="" resetcounter="" action="" group="horizontal">`

**Attributes:**
- `deck` - Deck number
- `resetcounter` - `"true"` to reset counter on click
- `action` - VDJScript action on click
- `group` - `"horizontal"` to display nested texts inline

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions
- `<background color=""/>` or `<background x="" y=""/>` - Background
- `<text>` - Text elements (see below)

**Text Element Attributes:**
- `font` - Font name (default: Arial)
- `weight` - Font weight: `"bold"`, `"normal"`
- `fontsize` - Size in pixels
- `color` - Text color
- `align` - `"left"`, `"center"`, `"right"`
- `valign` - `"top"`, `"center"`, `"bottom"`
- `text` - Static text or VDJScript query in backticks
- `format` - Format string for dynamic text
- `width` - Max width before wrapping
- `multiline` - `"true"` for multi-line text
- `action` - VDJScript action for clickable text

**Example:**
```xml
<textzone>
    <pos x="100" y="50"/>
    <size width="300" height="30"/>
    <text font="Arial" fontsize="18" color="white" 
          text="`get_artist` - `get_title`" align="center"/>
</textzone>
```

---

### `<group>`

Generic container for organizing elements.

**Syntax:** `<group name="" x="" y="">`

**Attributes:**
- `name` - Group identifier
- `x`, `y` - Position offset for all children

**Children:** Any skin element

**Example:**
```xml
<group name="Deck Controls" x="100" y="50">
    <button action="play_pause">
        <pos x="+0" y="+0"/>
    </button>
    <button action="cue">
        <pos x="+90" y="+0"/>
    </button>
</group>
```

---

### `<stack>`

Container with fade-in/fade-out transition effects for switching between items.

**Syntax:** `<stack fadein="" fadeout="">`

**Attributes:**
- `fadein` - Fade-in duration (e.g., `"200ms"`)
- `fadeout` - Fade-out duration (e.g., `"500ms"`)

**Children:**
- `<pos>`, `<size>` - Container dimensions
- `<slot>` - Positioning for items
- `<item class="" visibility="">` - Individual stack items

**Example:**
```xml
<stack fadein="200ms" fadeout="500ms">
    <pos x="100" y="100"/>
    <size width="300" height="200"/>
    <slot x="+0" y="+0"/>
    
    <item class="looppanel" visibility="is_using 'loop' 1000ms"/>
    <item class="hotcuepanel" visibility="is_using 'cue' 1000ms"/>
    <item class="defaultpanel"/>
</stack>
```

---

### `<square>`

Vector graphics rectangle with optional rounded corners.

**Syntax:** `<square color="" radius="" visibility="">`

**Attributes:**
- `color` - Fill color (hex or predefined)
- `radius` - Corner radius in pixels
- `visibility` - VDJScript visibility condition

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions

**Example:**
```xml
<square color="#1e7b96" radius="10">
    <pos x="100" y="50"/>
    <size width="200" height="100"/>
</square>

<!-- Dynamic visibility -->
<square color="red" radius="20" visibility="not play">
    <pos x="100" y="50"/>
    <size width="100" height="50"/>
</square>

<square color="green" radius="20" visibility="play">
    <pos x="100" y="50"/>
    <size width="100" height="50"/>
</square>
```

---

### `<circle>`

Vector graphics circle or ellipse.

**Syntax:** `<circle color="" visibility="">`

**Attributes:**
- `color` - Fill color (hex or predefined)
- `visibility` - VDJScript visibility condition

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions (equal for circle, different for ellipse)

**Example:**
```xml
<circle color="#ffffff">
    <pos x="100" y="100"/>
    <size width="50" height="50"/>
</circle>
```

---

### `<line>`

Vector graphics line drawing.

**Syntax:** `<line color="" width="" visibility="">`

**Attributes:**
- `color` - Line color
- `width` - Line thickness in pixels
- `visibility` - VDJScript visibility condition

**Children:**
- `<pos x="" y=""/>` - Start position
- `<pos2 x="" y=""/>` - End position

**Example:**
```xml
<line color="white" width="2">
    <pos x="100" y="100"/>
    <pos2 x="200" y="150"/>
</line>
```

---

### `<video>`

Display video output.

**Syntax:** `<video source="" canstretch="">`

**Attributes:**
- `source` - Video source:
  - `"deck"` - Current deck
  - `"master"` - Master output
  - `"1"`, `"2"`, etc. - Specific deck
- `canstretch` - `"true"` to allow resizing

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions

**Example:**
```xml
<video source="master" canstretch="true">
    <pos x="100" y="100"/>
    <size width="640" height="480"/>
</video>
```

---

### `<led>`

LED indicator that changes based on conditions.

**Syntax:** `<led brightness="">`

**Attributes:**
- `brightness` - VDJScript query for brightness level

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions
- `<off x="" y=""/>` - Off state graphics
- `<on x="" y=""/>` - On state graphics

**Example:**
```xml
<led brightness="`get_level`">
    <pos x="100" y="100"/>
    <size width="20" height="20"/>
    <off x="0" y="400"/>
    <on x="20" y="400"/>
</led>
```

---

### `<equalizer>`

Spectrum analyzer visualization.

**Syntax:** `<equalizer type="" nb="" color="" source="">`

**Attributes:**
- `type` - Display type:
  - `"bar"` - Bar graph
  - `"line"` - Line graph
- `nb` - Number of bands
- `color` - Color or color gradient
- `source` - Audio source:
  - `"deck"` - Current deck
  - `"master"` - Master output

**Children:**
- `<pos x="" y=""/>` - Position
- `<size width="" height=""/>` - Dimensions

**Example:**
```xml
<equalizer type="bar" nb="32" color="#00ff00" source="master">
    <pos x="100" y="500"/>
    <size width="800" height="100"/>
</equalizer>
```

---

### `<browser>`

File browser interface element displaying VirtualDJ's folder tree, song list, sideview, and info panels. Browsers can be nested in panels and have customizable visibility and appearance.

**Syntax:** `<browser panel="" visibility="" toolbar="" sideview="" folders="" infos="" effects="" searchbar="" lineheight="" showzoom="">`

**Attributes:**
- `visibility=""` - Define transparency (0-100%) or VDJScript action returning true/false for conditional display
- `panel=""` - Name of panel this browser belongs to (browser only shown when panel visible). Recommended to nest browser inside `<panel>` container instead
- `toolbar=""` - Show left toolbar: `"yes"` (default) or `"no"`
- `sideview=""` - Show sideview panel: `"yes"` (default) or `"no"`
- `folders=""` - Show folder list: `"yes"` (default) or `"no"`
- `infos=""` - Show info panel: `"yes"` (default) or `"no"`
- `effects=""` - Show effects section: `"yes"` (default) or `"no"`
- `searchbar=""` - Show search toolbar above file list: `"yes"` (default) or `"no"`
- `lineheight=""` - Height multiplier between browser lines (default: 1.0). Example: `lineheight="2.0"` for double height
- `showzoom=""` - Show zoom toggle button in folder toolbar: `"yes"` or `"no"`

**Children:**
- `<pos x="" y=""/>` - Position on screen
- `<size width="" height=""/>` - Browser dimensions
- `<colors background="">` - Color customization (see below)

**Color Customization:**
The `<colors>` element has these child elements:
- `background=""` - Background color (use `"transparent"` for transparent background)
  - **Note:** With transparent background, strongly recommended to set skin `breakline` and `breakline2` to prevent stretching issues

Browser color children (all optional, VirtualDJ uses defaults if not specified):
- Various color properties for text, highlights, borders, etc. (see example)

**Example 1: Basic Browser**
```xml
<browser>
    <pos x="50" y="50"/>
    <size width="600" height="400"/>
</browser>
```

**Example 2: Browser with Custom Options**
```xml
<browser toolbar="yes" sideview="yes" folders="yes" 
         infos="yes" searchbar="yes" lineheight="1.2">
    <pos x="50" y="50"/>
    <size width="700" height="500"/>
</browser>
```

**Example 3: Browser in Panel with Visibility**
```xml
<panel name="browserpanel" visible="yes">
    <browser>
        <pos x="50" y="50"/>
        <size width="600" height="400"/>
    </browser>
</panel>
```

**Example 4: Minimal Browser (No Sidebars)**
```xml
<browser toolbar="no" sideview="no" folders="yes" 
         infos="no" effects="no" searchbar="yes">
    <pos x="100" y="100"/>
    <size width="500" height="600"/>
</browser>
```

**Example 5: Browser with Transparent Background**
```xml
<browser>
    <pos x="50" y="50"/>
    <size width="600" height="400"/>
    <colors background="transparent"/>
</browser>

<!-- In <skin> element, add breaklines: -->
<skin ... breakline="100" breakline2="550">
```

**Example 6: Browser with Custom Line Height**
```xml
<browser lineheight="1.5">
    <pos x="50" y="50"/>
    <size width="600" height="400"/>
</browser>
```

**Custom Browsers with `<split>` Panels:**
Advanced skins can create custom browser layouts using `<split>` panels to arrange browser components in unique ways.

**Browser Sections:**
- **Folder List** - Tree view of folders and playlists
- **Song List** - Main file list with columns
- **Sideview** - Context panel showing recommendations, similar tracks, etc.
- **Info Panel** - Track information and waveform preview
- **Toolbar** - Left sidebar with navigation buttons
- **Search Bar** - Search input at top of file list
- **Effects Section** - Effect selection area

**Skin Breaklines:**
When using browser in skins, define breaklines in the `<skin>` element to specify where browser can stretch vertically when resizing:
```xml
<skin breakline="100" breakline2="550">
```

- **breakline** - Y-coordinate where stretching begins (top of browser)
- **breakline2** - Y-coordinate where stretching ends (bottom of browser)
- Area between breaklines will stretch; ensure no fixed-position buttons in this area
- Browser cannot be resized smaller than breakline 1 position

**Notes:**
- Browser automatically handles scrolling, sorting, filtering
- Most skins have one main browser, but multiple browsers are supported
- Use `visibility=""` attribute or nest in `<panel>` for conditional display
- Transparent backgrounds require careful breakline setup
- Custom browser colors rarely needed - VirtualDJ defaults work well

---

### `<window>`

Create a separate popup window.

**Syntax:** `<window name="" visible="">`

**Attributes:**
- `name` - Window identifier
- `visible` - Initial visibility: `"yes"` or `"no"`

**Children:** Any skin element

**Example:**
```xml
<window name="effects" visible="no">
    <size width="400" height="300"/>
    <!-- Window contents -->
</window>
```

---

## Global Element Attributes

These attributes can be applied to most elements:

| Attribute | Description | Values |
|-----------|-------------|--------|
| `visibility` | Show/hide based on condition | VDJScript query |
| `deck` | Target deck | `"1"`, `"2"`, `"left"`, `"right"`, `"master"` |
| `panel` | Parent panel name | panel name |
| `os` | OS-specific display | `"windows"`, `"mac"` |
| `canstretch` | Allow stretching on resize | `"true"`, `"false"` |

---

## Position & Size

### Position Element: `<pos>`

```xml
<pos x="100" y="200"/>
```

- Absolute: `x="100"` `y="200"`
- Relative: `x="+50"` `y="-25"` (relative to parent)
- Calculated: `x="1920-330"` `y="1080/2"`

### Size Element: `<size>`

```xml
<size width="300" height="150"/>
```

- Fixed: `width="300"` `height="150"`
- Calculated: `width="1920-50"` `height="1080/2"`

---

## Graphics References

Graphics are referenced by coordinates in the PNG file:

### Button States
- `<up x="" y=""/>` - Default up state
- `<down x="" y=""/>` - Pressed state
- `<on x="" y=""/>` - Active/on state
- `<off x="" y=""/>` - Inactive/off state
- `<over x="" y=""/>` - Mouse hover state
- `<selected x="" y=""/>` - Selected state

### Visual States
- `<off x="" y=""/>` - Low/minimum value
- `<on x="" y=""/>` - High/maximum value

---

## Predefined Colors

Colors can be specified as:
- Hex: `"#FF0000"`
- RGB: `"255,0,0"`
- Named: `"red"`, `"white"`, `"blue"`, etc.
- Defined: Use `<define color="" value="">` to create custom color names

---

## Default Icons

VirtualDJ provides built-in icons for common functions. Reference them without graphics coordinates:

```xml
<button action="play_pause">
    <pos x="100" y="100"/>
    <size width="40" height="40"/>
    <!-- No graphics coordinates needed for default icon -->
</button>
```

---

## Best Practices

### Organization
- Use `<define>` for repeated elements
- Group related elements in `<deck>` or `<group>` containers
- Use descriptive `name` attributes for debugging

### Graphics
- Use transparent PNG with alpha channel
- Organize graphics efficiently in sprite sheet
- Use vector shapes (`<square>`, `<circle>`) to reduce file size

### Performance
- Minimize use of complex visuals
- Use vector shapes where possible
- Keep image file size reasonable

### Visibility
- Use `visibility` attribute for conditional display
- Use `<panel>` for show/hide groups
- Use `<stack>` for smooth transitions

---

## Example: Complete Button Definition

```xml
<define class="playbutton">
    <size width="80" height="45"/>
    <off x="0" y="200"/>
    <on x="80" y="200"/>
    <over x="160" y="200"/>
    <text font="Arial" fontsize="12" color="white" text="PLAY" align="center"/>
</define>

<deck deck="left">
    <button class="playbutton" action="play_pause">
        <pos x="100" y="100"/>
    </button>
</deck>

<deck deck="right">
    <button class="playbutton" action="play_pause">
        <pos x="900" y="100"/>
    </button>
</deck>
```

---

## Additional Resources

- **VirtualDJ Manual**: https://www.virtualdj.com/manuals/virtualdj.html
- **VDJScript Reference**: See VDJScript Verbs document
- **Skin Examples**: Extract default skin from Settings > Interface
- **Forums**: https://www.virtualdj.com/forums/

---

## Notes

- Skin files must be zipped with `.zip` extension
- XML file and PNG file should have matching names (or use `image=""` attribute)
- Test on multiple screen resolutions
- Backup original skin before modifying
- Use VirtualDJ's skin creator for visual editing
