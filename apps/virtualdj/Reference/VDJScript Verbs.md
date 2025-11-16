# VirtualDJ VDJScript Verbs Reference

Comprehensive reference for VDJScript verbs organized by category.

## Flow Control

| Verb | Description | Example |
|------|-------------|---------|
| `nothing` | Do nothing | `nothing` |
| `up` | Execute action on key press/release | `up ? action1 : action2` |
| `down` | Execute action on key press/release | `down ? action1 : action2` |
| `isrepeat` | Check if key is auto-repeating | `isrepeat ? nothing : goto_cue` |

## Parameters & Constants

| Verb | Description | Example |
|------|-------------|---------|
| `true` / `on` / `yes` | Returns true | `true` |
| `false` / `no` / `off` | Returns false | `false` |
| `constant` / `get_constant` | Return specified value | `get constant 75%` |
| `dim` | Equivalent to `constant 0.1` | `dim` |
| `color_mix` | Mix two colors based on action | `color_mix white red \`get_limiter\`` |
| `color` | Return color value | `color "red"`, `color "#C08040"`, `color 0.8 0.5 0.25` |

## Parameter Comparison & Math

| Verb | Description | Example |
|------|-------------|---------|
| `param_bigger` / `param_greater` | Check if value is bigger | `param_bigger 0 ? action1 : action2` |
| `param_equal` | Check if value equals something | `param_equal \`get_browsed_song 'type'\` "audio"` |
| `param_contains` | Check if value contains string | `param_contains` |
| `param_smaller` | Check if value is smaller | `param_smaller 0 ? action1 : action2` |
| `param_add` | Add values | `param_add \`get_var a\` \`get_var b\`` |
| `param_multiply` | Multiply value | `param_multiply 300% & effect slider` |
| `param_1_x` | Invert value (1/x) | `param_1_x & effect slider` |
| `param_pow` | Power calculation | `param_pow 0.5` (square root) |
| `param_invert` | Invert value (1-x) | `param_invert & pitch_slider` |
| `param_mod` | Wrap value | `param_mod` |
| `param_pingpong` | Linear to forth-and-back scale | `param_pingpong` |
| `param_cast` | Cast to new type | `param_cast "percentage"` |
| `param_delta` | Transform absolute to relative | `param_delta` |
| `param_uppercase` | Convert to uppercase | `param_uppercase` |
| `param_lowercase` | Convert to lowercase | `param_lowercase` |
| `param_ucfirst` | First letter uppercase | `param_ucfirst` |

### Cast Types
- `integer`, `float`, `percentage`, `ms`, `boolean`, `beats`, `text`
- `int_trunc` - integer part without rounding
- `frac` - decimal part
- `relative`, `absolute` - change parameter type

## Timing & Animation

| Verb | Description | Example |
|------|-------------|---------|
| `blink` | Toggle LED on/off | `blink 1000ms`, `blink 1bt` |
| `fadeout` | Fade out when condition ends | `fadeout 10000ms 3000ms \`loop\`` |
| `pulse` | True for duration when action turns true | `is_using 'equalizer' & pulse 2000ms` |
| `param_make_discrete` | Make smooth encoder discrete | `param_make_discrete 0.1` |

## Repeat & Delay

| Verb | Description | Example |
|------|-------------|---------|
| `repeat` | Repeat action while pressed | `repeat 1000ms & browser_scroll +1` |
| `repeat_start` | Start repeating action | `repeat_start 'name' 1000ms 5 & action` |
| `repeat_start_instant` | Start repeating immediately | `repeat_start_instant 'name' 1000ms` |
| `repeat_stop` | Stop repeat | `repeat_stop 'name'` |
| `wait` | Wait between actions | `wait 1bt & pause` |
| `holding` | Execute if held long | `holding ? automix : mix_now` |
| `doubleclick` | Execute if double-clicked | `doubleclick ? automix : mix_now` |

## Skin Control

| Verb | Description | Example |
|------|-------------|---------|
| `skin_panel` | Show/hide panel | `skin_panel 'my_panel' on` |
| `skin_panelgroup` | Change panel in group | `skin_panelgroup 'groupname' 'panelname'` |
| `skin_panelgroup_available` | Set panel availability | `skin_panelgroup_available` |
| `lock_panel` | Acts on split elements | `lock_panel` |
| `show_splitpanel` | Show/hide split panel | `show_splitpanel 'sidelist'` |
| `rack` | Open/close rack unit | `rack 'rack1' 'unit1'` |
| `rack_solo` | Open unit full size | `rack_solo 'rack1' 'unit1'` |
| `rack_prioritize` | Prioritize unit | `rack_prioritize 'rack1' 'unit1'` |
| `zoom` / `zoom_scratch` | Zoom horizontal | `zoom` |
| `zoom_vertical` | Zoom vertical | `zoom_vertical` |
| `load_skin` | Load new skin/variation | `load_skin ':newvariation'` |

## Custom Buttons & Multi-buttons

| Verb | Description | Example |
|------|-------------|---------|
| `custom_button` | Custom button action | `custom_button` |
| `custom_button_name` | Get/set button name | `custom_button_name` |
| `has_custom_button` | Check if has action | `has_custom_button` |
| `custom_button_edit` | Open editor | `custom_button_edit` |
| `multibutton` | Click multibutton | `multibutton "my_button"` |
| `multibutton_select` | Open selection menu | `multibutton_select "my_button"` |

## System Info

| Verb | Description | Example |
|------|-------------|---------|
| `get_cpu` | CPU activity | `get_cpu` |
| `get_clock` | Current time | `get_clock`, `get_clock 12` (AM/PM) |
| `get_date` | Current date | `get_date "%Y/%m/%d"` |
| `is_pc` / `is_windows` | Check if PC | `is_pc` |
| `is_mac` / `is_macos` | Check if Mac | `is_mac` |
| `has_notch` | Check for display notch | `has_notch` |
| `get_battery` | Battery level | `get_battery` |
| `is_battery` | Running on battery | `is_battery` |
| `has_battery` | Has batteries | `has_battery` |
| `show_keyboard` | Show onscreen keyboard | `show_keyboard` |
| `system_volume` | Change system volume | `system_volume` |
| `has_system_volume` | Can modify system volume | `has_system_volume` |

## Variables

| Verb | Description | Example |
|------|-------------|---------|
| `var` | Conditional based on variable | `var "my_var" ? action1 : action2` |
| `var_equal` | Check equality | `var_equal "my_var" 42 ? action1 : action2` |
| `var_not_equal` | Check inequality | `var_not_equal "my_var" 42` |
| `var_smaller` | Check less than | `var_smaller "my_var" 42` |
| `var_greater` | Check greater than | `var_greater "my_var" 42` |
| `set_var_dialog` | Dialog to set var | `set_var_dialog 'varname'` |
| `set` | Set variable value | `set 'varname' 5` |
| `toggle` | Toggle true/false | `toggle "my_var"` |
| `cycle` | Increment with wrap | `cycle "my_var" 42` |
| `get_var` | Get variable value | `get_var "varname"` |
| `set_var` | Set variable value | `set_var` |
| `var_list` | Show variables window | `var_list` |
| `controllervar` | Controller-unique variable | `controllervar` |

## Window Control

| Verb | Description | Example |
|------|-------------|---------|
| `close` | Close application | `close` |
| `minimize` | Minimize to taskbar | `minimize` |
| `maximize` | Maximize/fullscreen/windowed | `maximize 'fullscreen'` |
| `show_window` | Show/hide window | `show_window` |

## Audio Playback

| Verb | Description | Example |
|------|-------------|---------|
| `song_pos` | Position in song (slider) | `song_pos` |
| `goto` | Change position | `goto +10ms`, `goto -4`, `goto 20%` |
| `goto_bar` | Jump to beat after downbeat | `goto_bar 4` |
| `songpos_remain` | Remaining time | `songpos_remain 500ms ? blink` |
| `songpos_warning` | Last 30s warning | `songpos_warning` |
| `seek` | Move while pressed | `seek +2`, `seek +420ms` |
| `reverse` | Play backward | `reverse` |
| `dump` | Reverse temporarily | `dump`, `dump quantized` |
| `goto_first_beat` | Jump to first beat | `goto_first_beat` |
| `goto_start` | Go to start | `goto_start` |

## Deck Management

| Verb | Description | Example |
|------|-------------|---------|
| `swap_decks` | Swap deck 1 and 2 | `swap_decks` |
| `clone_deck` | Clone deck | `clone_deck` |
| `clone_from_deck` | Clone from other deck | `clone_from_deck` |
| `move_deck` | Move song to other deck | `move_deck` |
| `stems_split` | Split stems to decks | `stems_split vocal target` |
| `stems_split_unlink` | Unlink split stems | `stems_split_unlink` |
| `dualdeckmode` | Toggle dual deck mode | `dualdeckmode` |
| `beatjump` | Jump beats | `beatjump +1` |
| `beatjump_select` | Set jump size | `beatjump_select 4` |
| `beatjump_page` | Change jump offset | `beatjump_page` |
| `beatjump_pad` | Execute jump | `beatjump_pad` |

## Play Controls

| Verb | Description | Example |
|------|-------------|---------|
| `play` | Start deck | `play` |
| `play_stutter` | Start or restart | `play_stutter` |
| `play_pause` | Toggle play/pause | `play_pause` |
| `pause_stop` | Pause or stop | `pause_stop` |
| `stop` | Stop to cue/beginning | `stop` |
| `pause` | Pause deck | `pause` |
| `play_button` | Depends on play_mode | `play_button` |
| `stop_button` | Depends on play_mode | `stop_button` |
| `emergency_play` | Play something | `emergency_play` |

## Audio Inputs

| Verb | Description | Example |
|------|-------------|---------|
| `mic` / `microphone` | Toggle microphone | `mic` |
| `mic_talkover` | Lower decks, activate mic | `mic_talkover 20% 1000ms` |
| `mic_eq_low/mid/high` | Mic EQ | `mic_eq_low` |
| `mic_volume` | Set mic volume | `mic_volume` |
| `linein` | Activate line input | `deck 1 linein 2 on` |
| `linein_rec` | Record line input | `linein_rec` |
| `mic_rec` | Record microphone | `mic_rec` |

## Scratch & Jogwheel

| Verb | Description | Example |
|------|-------------|---------|
| `touchwheel` / `scratchwheel` | Jogwheel with touch | `touchwheel +1.0` |
| `touchwheel_touch` | Touch detection | `touchwheel_touch` |
| `jogwheel` / `jog` | Jogwheel without touch | `jogwheel +1.0` |
| `motorwheel` | Motorized jogwheel | `motorwheel "move" +1.0` |
| `speedwheel` | Position and speed | `speedwheel +1.0 1.5` |
| `vinyl_mode` | Set vinyl/CD mode | `vinyl_mode` |
| `wheel_mode` | Change wheel mode | `wheel_mode +1` |
| `hold` / `scratch_hold` | Stop for scratching | `hold on` |
| `scratch` | Scratch forward/back | `scratch +120ms` |
| `nudge` | Nudge position | `nudge +120ms` |
| `slip_mode` | Slip mode | `slip_mode` |
| `slip` | Global slip mode | `slip` |
| `scratch_dna` | Execute DNA scratch | `scratch_dna` |
| `scratch_dna_editor` | Open DNA editor | `scratch_dna_editor` |

## Volume & Mixing

| Verb | Description | Example |
|------|-------------|---------|
| `crossfader` | Move crossfader | `crossfader 50%` |
| `auto_crossfade` | Auto crossfade | `auto_crossfade 2000ms` |
| `level` / `volume` | Set deck volume | `level` |
| `mute` | Mute deck | `mute` |
| `gain` | Set gain | `gain` |
| `set_gain` | Set gain to dBA | `set_gain 0` |
| `master_volume` | Master volume | `master_volume` |
| `headphone_volume` | Headphone volume | `headphone_volume` |
| `headphone_mix` | PFL mix | `headphone_mix` |
| `crossfader_curve` | Crossfader curve | `crossfader_curve "scratch"` |
| `get_limiter` | Check compression | `get_limiter` |
| `get_level` | Signal level | `get_level` |
| `get_vu_meter` | VU meter level | `get_vu_meter` |
| `is_audible` | Deck on-air | `is_audible` |

## Automix

| Verb | Description | Example |
|------|-------------|---------|
| `automix` | Start/stop automix | `automix` |
| `automix_dualdeckmode` | Use both decks | `automix_dualdeckmode` |
| `automix_skip` | Skip current song | `automix_skip` |
| `mix_now` | Crossfade with sync | `mix_now 4000ms` |
| `mix_now_nosync` | Crossfade without sync | `mix_now_nosync` |
| `mix_selected` | Mix to selected | `mix_selected` |
| `mix_next` | Mix to next | `mix_next` |
| `mix_and_load_next` | Mix and load next | `mix_and_load_next` |
| `playlist_randomize` | Shuffle playlist | `playlist_randomize` |
| `playlist_repeat` | Repeat playlist | `playlist_repeat` |
| `playlist_clear` | Empty playlist | `playlist_clear` |

## Browser

| Verb | Description | Example |
|------|-------------|---------|
| `browser_scroll` | Scroll songs/folders | `browser_scroll +1` |
| `browser_move` | Move song in playlist | `browser_move +1` |
| `browser_folder` | Focus folders | `browser_folder` |
| `browser_enter` | Load or focus songs | `browser_enter` |
| `browser_open_folder` | Expand/collapse folder | `browser_open_folder` |
| `browser_remove` | Remove from playlist | `browser_remove` |
| `browser_window` | Change browser zone | `browser_window 'folders'` |
| `search` | Focus search or search text | `search "text"` |
| `clear_search` | Clear search | `clear_search` |
| `browser_gotofolder` | Go to folder | `browser_gotofolder "/path"` |
| `browser_sort` | Sort browser | `browser_sort "artist"` |
| `grid_view` | Grid view mode | `grid_view` |
| `file_info` | Open tag editor | `file_info` |
| `browsed_file_color` | Set file color | `browsed_file_color "red"` |
| `browsed_file_analyze` | Reanalyze file | `browsed_file_analyze` |

## Loading

| Verb | Description | Example |
|------|-------------|---------|
| `load` | Load song | `load`, `load "path"` |
| `load_pulse` | Brief pulse on load | `load_pulse` |
| `loaded` | Check if loaded | `loaded` |
| `undo_load` | Reload previous | `undo_load` |
| `unload` | Unload song | `unload` |
| `load_next` | Load next track | `load_next` |
| `load_previous` | Load previous track | `load_previous` |

## Cue Points

| Verb | Description | Example |
|------|-------------|---------|
| `cue_stop` | Cue with preview | `cue_stop`, `cue_stop 1` |
| `cue_play` | Cue with hold-to-play | `cue_play 1 1000ms` |
| `cue` | Jump to cue | `cue`, `cue 1` |
| `hot_cue` / `hotcue` | Set or jump to cue | `hot_cue 1` |
| `silent_cue` | Mute until cue activated | `silent_cue` |
| `cue_select` | Select default cue | `cue_select` |
| `set_cue` | Store cue position | `set_cue 1 500ms` |
| `goto_cue` | Jump to cue | `goto_cue 1` |
| `delete_cue` | Delete cue | `delete_cue 1` |
| `cue_pos` | Get cue position | `cue_pos 1` |
| `cue_name` | Get/set cue name | `cue_name 1` |
| `has_cue` | Check if cue exists | `has_cue 1` |
| `cue_color` | Get/set cue color | `cue_color 1 'yellow'` |
| `cue_loop` | Jump and loop | `cue_loop` |
| `lock_cues` | Lock/unlock cues | `lock_cues` |

## Deck Selection

| Verb | Description | Example |
|------|-------------|---------|
| `select` | Select working deck | `select` |
| `masterdeck` | Select/unselect master | `masterdeck` |
| `masterdeck_auto` | Auto masterdeck | `masterdeck_auto` |
| `leftdeck` | Select left deck | `leftdeck +1` |
| `rightdeck` | Select right deck | `rightdeck +1` |
| `invert_deck` | Swap left/right deck | `invert_deck` |
| `leftcross` | Assign to left crossfader | `leftcross` |
| `rightcross` | Assign to right crossfader | `rightcross` |
| `pfl` | Send to headphones | `pfl`, `pfl 75%` |
| `get_deck_color` | Get deck color | `get_deck_color 50%` |

## Equalizer & Stems

| Verb | Description | Example |
|------|-------------|---------|
| `eq_mode` | Select EQ behavior | `eq_mode +1`, `eq_mode frequency` |
| `mute_stem` | Mute stem | `mute_stem vocal` |
| `only_stem` | Isolate stem | `only_stem vocal` |
| `stem_pad` | Mute/isolate stem pad | `stem_pad vocal` |
| `has_stems` | Check if has stems | `has_stems "ready"` |
| `eq_high` | High EQ/HiHat/Vocal | `eq_high` |
| `eq_mid` | Mid EQ/Melody/Vocals | `eq_mid` |
| `eq_low` | Low EQ/Kick | `eq_low` |
| `stem` | Control stem amount | `stem "vocal" 50%` |
| `eq_kill_high/mid/low` | Kill EQ band | `eq_kill_high` |
| `filter` | Apply color FX | `filter` |
| `filter_select` / `colorfx` | Select color effect | `filter_select` |

### Stem Names
- Individual: `Vocal`, `HiHat`, `Bass`, `Instru`, `Kick`
- Aggregate: `Melody` (Instru+Bass), `Rhythm` (HiHat+Kick), `MeloRhythm`, `Acapella`, `Instrumental`

## Get (Query Actions)

| Verb | Description | Example |
|------|-------------|---------|
| `get_beatpos` | Beat position | `get_beatpos` |
| `get_bpm` | Song BPM | `get_bpm`, `get_bpm absolute` |
| `get_time` | Elapsed time | `get_time "remain" "short"` |
| `get_rotation` | Disc angle | `get_rotation` |
| `get_position` | Song position | `get_position` |
| `get_deck` | Deck number | `get_deck` |
| `get_artist` | Artist tag | `get_artist` |
| `get_title` | Title tag | `get_title` |
| `get_album` | Album tag | `get_album` |
| `get_genre` | Genre tag | `get_genre` |
| `get_key` | Song key | `get_key "musical"` |
| `get_browsed_song` | Browsed file property | `get_browsed_song 'title'` |
| `get_loaded_song` | Loaded file property | `get_loaded_song 'album'` |

## Karaoke

| Verb | Description | Example |
|------|-------------|---------|
| `karaoke` | Start/stop karaoke | `karaoke` |
| `karaoke_show` | Show singer list | `karaoke_show` |
| `get_next_karaoke_song` | Get upcoming track info | `get_next_karaoke_song "singer" +1` |
| `is_karaoke_idle` | Karaoke idle check | `is_karaoke_idle` |
| `is_karaoke_playing` | Karaoke playing check | `is_karaoke_playing` |

## Key & Pitch

| Verb | Description | Example |
|------|-------------|---------|
| `key` | Change key (semitones) | `key +1` |
| `key_smooth` | Change key (smooth) | `key_smooth +0.5` |
| `key_move` | Move key by semitones | `key_move +1` |
| `set_key` | Match exact key | `set_key "A#m"` |
| `match_key` | Match compatible key | `match_key` |
| `key_lock` / `keylock` | Lock key | `key_lock` |
| `pitch` | Set pitch | `pitch 112%`, `pitch +0.1%` |
| `pitch_zero` | Reset to 0% | `pitch_zero` |
| `pitch_reset` | Slowly return to 0% | `pitch_reset 5%` |
| `pitch_range` | Set pitch range | `pitch_range 12%` |
| `pitch_bend` | Temporary bend | `pitch_bend +3%` |
| `master_tempo` | Toggle master tempo | `master_tempo` |
| `get_pitch` | Get pitch value | `get_pitch` |

## Loops

| Verb | Description | Example |
|------|-------------|---------|
| `loop` | Set/remove loop | `loop 4`, `loop 10ms`, `loop 200%` |
| `loop_in` | Set loop start | `loop_in` |
| `loop_out` | Set loop end | `loop_out` |
| `loop_length` | Change loop length | `loop_length 0.5` |
| `loop_move` | Move loop | `loop_move +2` |
| `loop_double` | Double loop | `loop_double` |
| `loop_half` | Halve loop | `loop_half` |
| `loop_exit` | Remove loop | `loop_exit` |
| `reloop` | Jump to loop start | `reloop` |
| `reloop_exit` | Remove or reactivate | `reloop_exit` |
| `loop_save` | Save loop | `loop_save 1`, `loop_save "name"` |
| `loop_load` | Load saved loop | `loop_load 1` |
| `saved_loop` | Load or set loop | `saved_loop 1` |
| `loop_roll` | Loop roll | `loop_roll 0.25` |
| `slicer` | Slicer effect | `slicer 1` |
| `loop_adjust` | Adjust loop with jog | `loop_adjust 'move'` |

## Pads

| Verb | Description | Example |
|------|-------------|---------|
| `pad` | Activate pad | `pad 1` |
| `pad_page` | Activate page | `pad_page 1`, `pad_page 'hotcues'` |
| `pad_edit` | Edit page | `pad_edit` |
| `pad_param` | Change param 1 | `pad_param` |
| `pad_color` | Get pad color | `pad_color 1` |
| `pad_button_color` | Controller button color | `pad_button_color 1` |
| `padfx` | Activate named effect | `padfx "echo" 40% 90%` |
| `padfx_single` | Activate single padfx | `padfx_single "reverb"` |

## Effects

| Verb | Description | Example |
|------|-------------|---------|
| `effect_select` | Select effect (deactivate previous) | `effect_select 1 "echo"` |
| `effect_select_multi` | Select effect (keep previous) | `effect_select_multi 2 "flanger"` |
| `effect_active` | Activate/deactivate | `effect_active 1 on` |
| `effect_slider` | Move effect slider | `effect_slider 1 2 50%` |
| `effect_button` | Press effect button | `effect_button 1 2` |
| `video_fx_select` | Select video effect | `video_fx_select "my_plugin"` |
| `effect_beats` | Set beat parameter | `effect_beats` |
| `get_effect_name` | Get effect name | `get_effect_name` |

## POI & BPM

| Verb | Description | Example |
|------|-------------|---------|
| `beat_tap` | Tap to set BPM | `beat_tap` |
| `edit_poi` | Open POI editor | `edit_poi` |
| `edit_bpm` | Open BPM editor | `edit_bpm` |
| `set_bpm` | Set BPM | `set_bpm 129.3`, `set_bpm 50%` |
| `adjust_cbg` | Adjust beat grid | `adjust_cbg +2` |
| `set_firstbeat` | Set first beat | `set_firstbeat` |
| `reanalyze` | Reanalyze file | `reanalyze multi` |

## Sampler

| Verb | Description | Example |
|------|-------------|---------|
| `sampler_play` | Play sample | `sampler_play 4` |
| `sampler_stop` | Stop sample | `sampler_stop 4`, `sampler_stop all` |
| `sampler_pad` | Trigger sample | `sampler_pad 1` |
| `sampler_volume` | Set volume | `sampler_volume 1` |
| `sampler_rec` | Record sample | `sampler_rec "mic"` |
| `sampler_loop` | Set sample loop | `sampler_loop 1 1` |
| `sampler_bank` | Select bank | `sampler_bank "birthday"` |
| `sampler_mode` | Set trigger mode | `sampler_mode 1 'stutter'` |
| `sampler_output` | Select output | `sampler_output "headphones"` |

### Sampler Modes
- `on/off` - Toggle
- `hold` - Play while held
- `stutter` - Restart on press
- `unmute` - Unmute mode

## Sync

| Verb | Description | Example |
|------|-------------|---------|
| `sync` | Synchronize with other deck | `sync` |
| `match_bpm` | Match BPM only | `match_bpm` |
| `play_sync` | Play synchronized | `play_sync` |
| `beatlock` | Keep synchronized | `beatlock` |
| `smart_fader` | Sync while crossfading | `smart_fader` |
| `phrase_sync` | Match phrase | `phrase_sync 16` |
| `quantize_all` | Set all quantize options | `quantize_all` |

## Video

| Verb | Description | Example |
|------|-------------|---------|
| `leftvideo` | Assign left video | `leftvideo +1` |
| `rightvideo` | Assign right video | `rightvideo +1` |
| `video` | Open/close video window | `video` |
| `video_output` | Select monitor | `video_output 1` |
| `video_crossfader` | Video crossfader | `video_crossfader` |
| `video_transition` | Launch transition | `video_transition 1000ms` |
| `is_video` | Check if has video | `is_video` |

## Recording & Broadcasting

| Verb | Description | Example |
|------|-------------|---------|
| `record` | Start recording | `record` |
| `record_cut` | Cut to new file | `record_cut` |
| `broadcast` | Start/stop broadcast | `broadcast "video"` |
| `get_record_time` | Recording time | `get_record_time` |

## Controllers

| Verb | Description | Example |
|------|-------------|---------|
| `action_deck` | Check button deck | `action_deck 1 ? actionA : actionB` |
| `set_deck` | Affect which deck | `set_deck \`get_var varname\` & play` |
| `device_side` | Left/right device action | `device_side 'left' ? action1 : action2` |
| `assign_controller` | Assign controller to deck | `deck 1 assign_controller "CDJ400" 2` |
| `shift` | Built-in shift variable | `shift` |
| `menu_button` | Changeable button | `menu_button 1 "hotcue,sampler"` |

## Configuration

| Verb | Description | Example |
|------|-------------|---------|
| `settings` / `config` | Open config window | `settings` |
| `smart_loop` | Auto-adjust loops | `smart_loop` |
| `smart_play` / `auto_sync` | Auto-sync on play | `smart_play` |
| `smart_cue` | Auto-sync on cue | `smart_cue` |
| `auto_match_bpm` | Auto-match BPM on load | `auto_match_bpm` |
| `auto_match_key` | Auto-match key on load | `auto_match_key` |
| `setting` | Read/write setting | `setting "jogSensitivityScratch" 80%` |
| `save_config` | Save config now | `save_config` |

## Timecode

| Verb | Description | Example |
|------|-------------|---------|
| `timecode_active` | Enable timecode control | `timecode_active 1 on` |
| `timecode_mode` | Set mode | `timecode_mode 'smart'` |
| `timecode_bypass` | Use as line input | `timecode_bypass` |
| `get_hastimecode` | Check if has timecode | `get_hastimecode` |

## Macros

| Verb | Description | Example |
|------|-------------|---------|
| `macro_record` | Record macro | `macro_record` |
| `macro_play` | Play macro | `macro_play` |

## Sandbox

| Verb | Description | Example |
|------|-------------|---------|
| `sandbox` | Toggle sandbox mode | `sandbox` |
| `can_sandbox` | Check if can sandbox | `can_sandbox` |

## Text Queries

| Verb | Description | Example |
|------|-------------|---------|
| `get_text` | Get formatted text | `get_text 'You are listening to \`get loaded_song "title"\`'` |
| `stopwatch` | Stopwatch | `stopwatch` |
| `countdown` | Count down to date/time | `countdown '2025/01/01 00:00'` |

## Common Patterns

### Conditional Execution
```
condition ? action_if_true : action_if_false
```

### Backtick Queries
Use backticks to execute queries within actions:
```
set 'varname' `play`
param_equal `get_browsed_song 'type'` "audio"
```

### Time Units
- `ms` - milliseconds
- `bt` - beats
- `%` - percentage

### Deck Specification
Prefix action with deck number:
```
deck 1 play
deck 2 volume 50%
deck master get_level
```
