# VirtualDJ Options Reference

Comprehensive reference for VirtualDJ configuration options organized by category.

## Smart Features

| Option                     | Description                               | Values                      |
| -------------------------- | ----------------------------------------- | --------------------------- |
| `autoBPMMatch`             | Auto-change pitch to match BPM on load    | yes, no, smart (within 10%) |
| `autoPitchLock`            | Auto-engage pitch_lock when BPMs matched  | yes, no                     |
| `autoGain`                 | Auto-set gain to 0dBA on load             | yes, no                     |
| `autoKey`                  | Auto-match key on load (up to 1 semitone) | yes, no                     |
| `autoCue`                  | Auto-jump to first cue/beat on load       | yes, no, off                |
| `smartPlay`                | Auto-start playing beatmatched            | yes, no                     |
| `smartPlayLimitPitchRange` | Limit smartPlay to ±10% BPM               | yes, no                     |
| `smartCue`                 | Auto-adjust jump to keep beatmatched      | yes, no                     |
| `smartLoop`                | Auto-adjust loops to be seamless          | yes, no                     |
| `quantizeLoop`             | Auto-quantize loop position               | yes, no                     |
| `quantizeSetCue`           | Auto-quantize cue position                | yes, no                     |
| `quantizeScratch`          | Keep synced after scratching              | yes, no                     |
| `globalQuantize`           | Quantize to beats/measures                | 0.25, 1, 4                  |
| `smartScratch`             | Auto-mute backward scratching             | yes, no                     |
| `cueLoopAutoSync`          | Quantize cue_loop to beat                 | yes, no                     |

## Reset Options (On Load)

| Option             | Description                   | Values  |
| ------------------ | ----------------------------- | ------- |
| `resetPitchOnLoad` | Reset pitch to 0% on load     | yes, no |
| `resetEqOnLoad`    | Reset EQ/filter to 0 on load  | yes, no |
| `resetStemsOnLoad` | Reset stems on load           | yes, no |
| `resetFXOnLoad`    | Stop all effects on load      | yes, no |
| `resetKeyOnLoad`   | Reset key to normal on load   | yes, no |
| `resetGainOnLoad`  | Reset gain and apply autoGain | yes, no |

## Auto-Routing

| Option           | Description                                   | Values  |
| ---------------- | --------------------------------------------- | ------- |
| `autoHeadphones` | Auto-switch headphones to loaded/touched song | yes, no |
| `pflOnSelect`    | Auto-switch PFL when deck selected            | yes, no |

## Key Matching

| Option        | Description       | Values                      |
| ------------- | ----------------- | --------------------------- |
| `keyMatching` | Key matching mode | standard, fuzzy, fuzzy full |

- **standard**: ±1 semitone for harmonic match
- **fuzzy**: Allows major/minor key changes
- **fuzzy full**: Up to ±2 semitones

## Controls

| Option                   | Description                   | Values                 |
| ------------------------ | ----------------------------- | ---------------------- |
| `playMode`               | Play/stop button behavior     | numark, pioneer        |
| `cueMode`                | Cue button behavior           | cue, cue-hold, cue-cup |
| `hotcueMode`             | Hotcue button behavior        | play, stutter          |
| `updateHotCueOnCueCombo` | Update cue when held with CUE | yes, no                |
| `hotcueSavesLoop`        | Save loop with hotcue         | yes, no                |
| `loopBackMode`           | Loop start/end behavior       | yes, no, smart         |
| `loopAutoMove`           | Move loop when cue called     | yes, no                |
| `loopDefault`            | Default loop beats            | number                 |
| `loopRollDefault`        | Default loop roll beats       | number                 |
| `beatjump`               | Beatjump distance in beats    | number                 |
| `keepPlayingPastEnd`     | Play past song end            | yes, no                |
| `keepPlayStatusOnLoad`   | Auto-play on load if playing  | yes, no                |

### Play/Cue Modes

- **numark**: play-stutter, pause-stop
- **pioneer**: play-pause, stop
- **cue-hold**: Play if held >2s
- **cue-cup**: Play on release, stop on push

## EQ & Stems

| Option                 | Description                              | Values                              |
| ---------------------- | ---------------------------------------- | ----------------------------------- |
| `eqMode`               | EQ knob algorithm                        | frequency, modernEQ, ezRemix, stems |
| `eqModeDual`           | Use both frequency & modern EQ on 4-deck | yes, no                             |
| `stemsBleedMuteVocal`  | Vocal mute bleed tolerance               | 0-100%                              |
| `stemsBleedMuteInstru` | Instrument mute bleed tolerance          | 0-100%                              |
| `stemsBleedOnlyVocal`  | Vocal isolation bleed tolerance          | 0-100%                              |
| `stemsBleedOnlyInstru` | Instrument isolation bleed tolerance     | 0-100%                              |
| `stemsSplitLeftRight`  | Stem split direction                     | yes (1→2,3→4), no (1→3,2→4)         |

### Bleed Tolerance

- **0%**: Strict separation (removes more)
- **100%**: Less strict (preserves more original)

## Pitch & Tempo

| Option            | Description                   | Values                                          |
| ----------------- | ----------------------------- | ----------------------------------------------- |
| `vinylMode`       | Jogwheel behavior             | vinyl (scratch), cd (bend)                      |
| `masterTempo`     | Lock key when pitching        | yes, no                                         |
| `pitchRange`      | Pitch slider range            | 6%, 8%, 10%, 12%, 16%, 20%, 25%, 33%, 50%, 100% |
| `autoPitchRange`  | Auto-adjust range when needed | yes, no                                         |
| `pitchResetSpeed` | Pitch reset speed             | %/second (default: 1)                           |

## Faders

| Option              | Description               | Values                     |
| ------------------- | ------------------------- | -------------------------- |
| `faderStart`        | Auto-play when fader up   | yes, no                    |
| `faderStartStop`    | Auto-stop when fader down | yes, no                    |
| `crossfaderCurve`   | Crossfader curve          | smooth, full, scratch, cut |
| `crossfaderDisable` | Disable crossfader        | yes, no                    |
| `crossfaderCustom`  | Custom crossfader config  | custom string              |
| `crossfaderHamster` | Invert crossfader         | yes, no                    |
| `levelfaderHamster` | Invert level fader        | yes, no                    |

## Effects

| Option          | Description                         | Values        |
| --------------- | ----------------------------------- | ------------- |
| `effects`       | Remember deck effect slots          | config string |
| `masterEffects` | Remember master effect slots        | config string |
| `mixFx`         | Last selected crossfader mix effect | effect name   |

## Pads

| Option                    | Description                     | Values        |
| ------------------------- | ------------------------------- | ------------- |
| `padsPagesOrder`          | Pad pages order                 | page list     |
| `padsPagesHidden`         | Hidden pages                    | page list     |
| `sixteenPadsMode`         | 16-pad mode                     | yes, no, auto |
| `padsPagesChanged`        | Modified from defaults          | yes, no       |
| `padsSkinIndependent`     | Separate skin/controller pads   | yes, no       |
| `loopPadPage`             | Selected loop pad page per deck | page numbers  |
| `autoSortCues`            | Auto-sort cues chronologically  | yes, no       |
| `autoBpmTransitionLength` | Auto BPM transition length      | beats         |

## Skins

| Option                              | Description                       | Values                                      |
| ----------------------------------- | --------------------------------- | ------------------------------------------- |
| `skinWaveformType`                  | Main waveform type                | horizontal, vertical, circular, etc         |
| `coloredWaveforms`                  | Waveform color scheme             | stems, frequency, beat                      |
| `skinOverviewType`                  | Overview waveform type            | auto, horizontal, etc                       |
| `skinWaveformScratchType`           | Scratch wave type                 | type name                                   |
| `waveUseFrequency`                  | Use frequency vs stems for colors | yes, no                                     |
| `waveGrayOnKill`                    | Gray killed stems on wave         | yes, no                                     |
| `waveformCenter`                    | Waveform centering                | yes, no                                     |
| `skinPlayheadShadow`                | Show playhead shadow              | yes, no                                     |
| `showGridLines`                     | Show grid lines                   | yes, no                                     |
| `beatCounterRange`                  | Beat counter range                | beats (default: 16)                         |
| `rhythmZoom`                        | Horizontal rhythm zoom            | value                                       |
| `scratchZoomVertical`               | Vertical scratch zoom             | value                                       |
| `touchScreenMode`                   | Force touchscreen options         | yes, no                                     |
| `multiTouchTwoFingerScroll`         | Two-finger scroll                 | yes, no                                     |
| `onScreenKeyboard`                  | Show onscreen keyboard            | yes, no                                     |
| `keyboardShowKeymapOverlay`         | Show keymap on CTRL/ALT           | yes, no                                     |
| `keyboardKeymapOverlayOnStickyKeys` | Keep overlay on sticky keys       | yes, no                                     |
| `skinEmptyButtons`                  | Show customizable buttons         | yes, no                                     |
| `customButtons`                     | Custom button actions             | action strings                              |
| `skin3FxLayout`                     | 3 FX layout                       | 1fx3knobs, 3fx1knob                         |
| `skin6FxLayout`                     | 6 FX layout                       | enabled/disabled                            |
| `vuMeter`                           | VU meter mode                     | system, audio                               |
| `clockDisplay`                      | Clock display format              | 12, 24                                      |
| `dateFormat`                        | Date format                       | format string                               |
| `cueDisplay`                        | Hotcue button display             | name, number, etc                           |
| `savedLoopDisplay`                  | Saved loop button display         | name, number, etc                           |
| `displayTime`                       | Time display mode                 | elapsed, remain, total                      |
| `keyDisplay`                        | Key display format                | musical, harmonic                           |
| `cpuMeter`                          | CPU meter mode                    | system, audio                               |
| `hideSongInfo`                      | Hide title/artist                 | yes, no                                     |
| `tooltip`                           | Show tooltips                     | yes, no                                     |
| `tooltipDelay`                      | Tooltip delay                     | ms                                          |
| `showCoverForDragDrop`              | Show cover when dragging          | yes, no                                     |
| `RPM`                               | Dome rotation speed               | RPM (default: 33)                           |
| `dialogsColorTheme`                 | Dialog color theme                | light, dark, auto                           |
| `cleartype`                         | Use ClearType                     | yes, no                                     |
| `maximized`                         | Window mode                       | 0 (windowed), 1 (maximized), 2 (fullscreen) |
| `skin`                              | Current skin file                 | filename                                    |
| `skinWindows`                       | Skin windows config               | config string                               |
| `skinPanels`                        | Panel visibility                  | config string                               |
| `skinTextzones`                     | Text zones config                 | config string                               |
| `skinSplitState`                    | Split state                       | config string                               |
| `skinRacks`                         | Rack state                        | config string                               |

## Audio

| Option                       | Description                 | Values                |
| ---------------------------- | --------------------------- | --------------------- |
| `audioAutoDetect`            | Auto-detect soundcard       | yes, no               |
| `exclusiveAudioAccess`       | Exclusive soundcard control | yes, no               |
| `splitHeadphones`            | Master left, PFL right      | yes, no               |
| `equalizerInHeadphones`      | EQ on PFL                   | yes, no               |
| `headphonesGain`             | Headphone output gain       | dB                    |
| `prelistenOutput`            | Prelisten deck              | 0 (auto), deck number |
| `boothMicrophone`            | Mic on booth output         | yes, no               |
| `microphoneToMaster`         | Mic on master output        | yes, no               |
| `metronomeVolume`            | Metronome volume            | %                     |
| `gainSliderIncludesAutoGain` | 100% = auto gain            | yes, no               |
| `faderCurve`                 | Volume slider curve         | 0-1 (linear to log)   |
| `zeroDB`                     | Additional headroom         | dB                    |
| `rampStartTime`              | Vinyl startup time          | ms                    |
| `rampStopTime`               | Vinyl brake time            | ms                    |
| `rampScratchTime`            | Vinyl touch reaction time   | ms                    |
| `equalizerFrequencySpread`   | EQ band spread              | default, full kill    |
| `equalizerLowFrequency`      | Low EQ frequency            | Hz                    |
| `equalizerMidFrequency`      | Mid EQ frequency            | Hz                    |
| `equalizerHighFrequency`     | High EQ frequency           | Hz                    |
| `filterDefaultResonance`     | Filter resonance            | %                     |
| `fxProcessing`               | Effect processing point     | pre-fader, post-fader |

## Video

| Option                        | Description               | Values                  |
| ----------------------------- | ------------------------- | ----------------------- |
| `useVideoSkin`                | Enable video skins        | yes, no                 |
| `videoSkin`                   | Current video skin        | filename                |
| `showVideoSkinOnPreview`      | Show on preview           | yes, no                 |
| `videoLogo`                   | Show logo                 | yes, no                 |
| `videoLogoImage`              | Logo image                | filepath                |
| `videoLogoSize`               | Logo size                 | %                       |
| `videoLogoPosition`           | Logo position             | position string         |
| `videoCrossfader`             | Video crossfader mode     | separate, linked, smart |
| `videoVolumeLink`             | Link video to volume      | yes, no                 |
| `videoTransition`             | Video transition plugin   | plugin name             |
| `videoRandomTransition`       | Random transitions        | yes, no                 |
| `videoFx`                     | Video FX plugin           | plugin name             |
| `videoAudioOnlyVisualisation` | Audio-only visualization  | plugin name             |
| `letterBoxing`                | Aspect ratio handling     | mode                    |
| `videoFPS`                    | Video frame rate          | fps                     |
| `videoMicroFrames`            | Add micro-frames          | yes, no, smart          |
| `videoResampleQuality`        | Rescale quality           | low, high               |
| `videoShaderQuality`          | Shader quality            | low, high               |
| `videoUseDXVA`                | Hardware acceleration     | yes, no                 |
| `videoDriver`                 | Video driver              | driver name             |
| `videoMaxMemory`              | Max video memory          | MB                      |
| `videoDelay`                  | Video/audio delay         | ms                      |
| `videoWindowAlwaysOnTop`      | Keep video on top         | yes, no                 |
| `videoWindowPosition`         | Video window positions    | position string         |
| `videoCreateLinkOnDrop`       | Create video edit on drop | yes, no                 |
| `startVideoOnLoad`            | Open video on load        | yes, no                 |

## Karaoke

| Option                    | Description             | Values                   |
| ------------------------- | ----------------------- | ------------------------ |
| `karaokeBackground`       | Background music source | automix, sidelist, image |
| `karaokeBackgroundMusic`  | Background folder       | folder path              |
| `karaokeBackgroundImage`  | Background image        | filepath                 |
| `karaokeBackgroundVolume` | Background volume       | %                        |
| `karaokeVideoSkin`        | Karaoke video skin      | skin name                |
| `karaokeSkipSilence`      | Auto-skip silence       | yes, no                  |
| `karaokeAutoRemovePlayed` | Remove played tracks    | yes, no                  |
| `karaokeDualDeck`         | Use separate decks      | yes, no                  |

## Controllers

| Option                        | Description                  | Values                             |
| ----------------------------- | ---------------------------- | ---------------------------------- |
| `mixerOrder`                  | Deck order (4-deck)          | e.g., 3124                         |
| `controllerTakeoverMode`      | Slider takeover mode         | instant, pickup, gradual           |
| `controllerTakeoverModePitch` | Pitch takeover mode          | instant, pickup, gradual, relative |
| `touchWheelBackspin`          | Backspin effect              | yes, no                            |
| `touchWheelForwardspin`       | Forwardspin effect           | yes, no                            |
| `touchWheelSpinThreshold`     | Spin threshold speed         | value                              |
| `jogSensitivityScratch`       | Scratch sensitivity          | %                                  |
| `jogSensitivityCue`           | Cue sensitivity              | %                                  |
| `jogSensitivityBend`          | Bend sensitivity             | %                                  |
| `jogVibrationProtection`      | Anti-vibration               | value (0 = off)                    |
| `motorWheelInstantPlay`       | Instant play on motor wheel  | yes, no                            |
| `motorWheelInstantStop`       | Instant stop on motor wheel  | yes, no                            |
| `motorWheelSmoothPercent`     | Motor wheel smoothing        | %                                  |
| `motorWheelLockTime`          | Motor lock time              | seconds                            |
| `controllerRefreshRate`       | Refresh rate                 | ms (0 = 10ms)                      |
| `controllerWaveFormZoom`      | Controller waveform zoom     | value                              |
| `disableBuiltInDefinitions`   | Disable built-in controllers | yes, no                            |
| `createMidiLog`               | Create MIDI log              | yes, no                            |
| `midiLogLevel`                | MIDI log level               | level                              |
| `showControllersSubDevices`   | Show sub-devices             | yes, no                            |

## Timecode

| Option                          | Description                | Values                    |
| ------------------------------- | -------------------------- | ------------------------- |
| `timecodeMode`                  | Needle-drop mode           | relative, absolute, smart |
| `timecodeType`                  | Timecode type              | auto-detected or manual   |
| `timecodeLeadInTime`            | Skip damaged start section | ms                        |
| `timecodeAntiSkip`              | Anti-skip grooves          | grooves                   |
| `timecodeNeedleDropSync`        | Wait before playing        | yes, no                   |
| `timecodePitchSliderIgnoreBend` | Ignore pitch during bend   | yes, no                   |
| `timecodeSilence`               | Silence threshold          | value                     |
| `timecodeCalibrationVolume`     | Calibrated volume          | value                     |
| `timecodeCalibrationPhase`      | Calibrated phase           | value                     |

## Sampler

| Option                            | Description               | Values                        |
| --------------------------------- | ------------------------- | ----------------------------- |
| `samplerBank`                     | Current sample bank       | bank name                     |
| `samplerTriggerMode`              | Default trigger mode      | on/off, hold, stutter, unmute |
| `samplerDefaultLoopMode`          | Default loop sync mode    | mode                          |
| `samplerForceNbColumns`           | Force pad layout          | columns                       |
| `samplerSpanAcrossDecks`          | 16 samples across 2 decks | yes, no                       |
| `samplerExportLossless`           | Save as FLAC              | yes (FLAC), no (OGG)          |
| `samplerDontSaveSource`           | Don't save source path    | yes, no                       |
| `samplerRootFolder`               | Samples folder            | folder path                   |
| `samplerHideDefaultBanks`         | Hide default banks        | yes, no                       |
| `samplerHideLegacyBanks`          | Hide v8 banks             | yes, no                       |
| `samplerOutputDeck`               | Sample output routing     | 0, -2, -1, 1, 2, etc          |
| `samplerApplyEffectsOnDeckOutput` | Apply deck FX to sampler  | yes, no                       |
| `samplerVideoVolumeLink`          | Video fades with volume   | yes, no                       |
| `autoSideview`                    | Auto-switch sideview      | yes, no                       |
| `samplerImageSize`                | Pad image size            | size                          |
| `samplerHeadphones`               | Sampler to headphones     | yes, no                       |
| `samplerShowEffects`              | Show effect toolbar       | yes, no                       |
| `samplerShowWaveform`             | Show waveforms            | yes, no                       |
| `samplerIndependentDeckBanks`     | Deck-specific banks       | yes, no                       |
| `samplerRecordStemsPads`          | Record stems pads config  | config                        |
| `samplerRecordLength`             | Stem record length        | duration                      |

### Sampler Output Routing

- **0**: Master (all decks)
- **-2**: Trigger deck
- **-1**: Headphones
- **1, 2, etc**: Specific deck

## Browser

| Option                           | Description                    | Values                |
| -------------------------------- | ------------------------------ | --------------------- |
| `fileFormats`                    | Displayed file extensions      | extension list        |
| `rootFoldersLocation`            | Root folders                   | folder paths          |
| `browserShortcuts`               | Folder shortcuts               | folder paths          |
| `browserShortcutsIcons`          | Shortcut icons                 | icon indices          |
| `browserShortcutsCustomIconFile` | Custom icon file               | filepath              |
| `browserShortcutsDefaultIcon`    | Default shortcut icon          | icon number           |
| `iTunesDatabaseFile`             | iTunes DB path                 | filepath              |
| `seratoFolder`                   | Serato crates folder           | folder path           |
| `traktorFolder`                  | Traktor folder                 | folder path           |
| `rekordboxFolder`                | Rekordbox library path         | folder path           |
| `importV7Databases`              | Auto-import v7 DBs             | yes, no               |
| `ignoreDrives`                   | Drives to ignore               | drive list            |
| `readOnly`                       | Read-only mode                 | yes, no               |
| `searchInFolder`                 | Search current folder first    | yes, no               |
| `searchInDB`                     | Search all database            | yes, no               |
| `searchInOnlineCatalogs`         | Search online                  | yes, no               |
| `OnlineCatalogsWhenEmpty`        | Online only when empty         | yes, no               |
| `OnlineCatalogs`                 | Online catalogs                | catalog list          |
| `onlineCatalogsContent`          | Content preference             | all, audio, video     |
| `showMusic`                      | Show audio only                | yes, no               |
| `showVideo`                      | Show video only                | yes, no               |
| `showKaraoke`                    | Show karaoke only              | yes, no               |
| `searchFields`                   | Search fields                  | field list            |
| `browserColumns`                 | Browser columns                | column list           |
| `browserSort`                    | Sort column                    | column name           |
| `browserGridColumns`             | Grid view columns              | column list           |
| `infoviewColumns`                | Info panel fields              | field list            |
| `showHorizontalSideList`         | Show horizontal sidelist       | yes, no               |
| `lockFolderOrder`                | Lock folder order              | yes, no               |
| `keepSortOrder`                  | Keep sort on folder change     | yes, no               |
| `rememberRecurse`                | Remember recurse per folder    | yes, no               |
| `browserSearchByFirstLetter`     | First letter search            | yes, no               |
| `lastSelectedFolder`             | Last folder                    | folder path           |
| `coverFlow`                      | Cover display mode             | mode                  |
| `lastTrackListDate`              | Last tracklist date            | date                  |
| `historyDelay`                   | Seconds before "played"        | seconds (default: 45) |
| `writeHistory`                   | Log tracks in history          | yes, no               |
| `prelistenVisible`               | Show prelisten control         | yes, no               |
| `prelistenStopOnChange`          | Stop on browser change         | yes, no               |
| `prelistenStartPos`              | Prelisten start position       | -1 (default), 0-1     |
| `autoSearchDB`                   | Auto-add to search DB          | yes, no               |
| `showZipKaraoke`                 | Check ZIPs for karaoke         | yes, no               |
| `showM3UAsFolders`               | Show M3U as subfolders         | yes, no               |
| `fontSize`                       | Browser font size              | modifier              |
| `browserPadding`                 | Line padding                   | %                     |
| `browserBPMDigits`               | BPM decimal places             | digits                |
| `savePlaylist`                   | Save playlist between sessions | yes, no               |
| `saveUnplayedToSidelist`         | Unplayed to sidelist           | yes, no               |
| `removePlayedFromSidelist`       | Remove played from sidelist    | yes, no               |
| `browserTextFit`                 | Large text display             | mode                  |
| `tracklistFormat`                | Tracklist.txt format           | format string         |
| `shellIcons`                     | Use OS icons                   | yes, no               |
| `sideviewShortcuts`              | Sideview folder shortcuts      | folder list           |
| `sideView`                       | Sideview state                 | state                 |
| `gridView`                       | Grid view mode                 | yes, no               |
| `triggerPadView`                 | Sampler pad view               | yes, no               |
| `sideViewReco`                   | Recommendation panel mode      | mode                  |
| `RemixesViewProvider`            | Remixes provider               | provider              |
| `liveFeedbackProviders`          | Feedback providers             | provider list         |
| `logUnsuccessfulSearches`        | Log failed searches            | yes, no               |
| `chartsCountry`                  | Charts country                 | country code          |
| `filterFolderSplitGenreBySlash`  | Split genre by slash           | yes, no               |
| `browserAutoZoom`                | Auto-zoom on mouseover         | yes, no               |
| `browserFontSizeButtons`         | Font size buttons              | yes, no               |
| `browserPreviousFoldersButton`   | Previous folders button        | yes, no               |
| `browserDaysSongsAreNew`         | Days files are "new"           | days                  |
| `browserShowSideviewInLists`     | Show sideview in MyLists       | yes, no               |
| `disableHotplugForNewLists`      | New lists on main drive        | yes, no               |
| `disableDuplicateForNewLists`    | No duplicates by default       | yes, no               |
| `browserAutoExportM3U`           | Auto-save M3U copies           | yes, no               |
| `browserShowLegacyM3UPlaylists`  | Show old M3U playlists         | yes, no               |
| `browserAutoOpenNewDrive`        | Auto-select new drives         | yes, no               |
| `cdjExportStemsConfig`           | CDJ stems combinations         | config                |
| `cdjExportStems`                 | CDJ export stems               | stems list            |
| `cdjExportCompatibility`         | CDJ compatibility level        | level                 |
| `cdjExportShowAllDrives`         | Show all USB drives            | yes, no               |
| `cdjExportAutoSyncCues`          | Auto-sync CDJ cues             | yes, no               |
| `cdjExportCuesAsMemoryPoints`    | Cues as memory points          | yes, no               |
| `quickFilters`                   | Available quick filters        | filter list           |
| `colorRules`                     | Color rules                    | rule list             |
| `user1FieldName`                 | User field 1 name              | name                  |
| `user2FieldName`                 | User field 2 name              | name                  |
| `favoriteGenres`                 | Favorite genres                | genre list            |
| `favoriteTags1`                  | Favorite user1 tags            | tag list              |
| `favoriteTags2`                  | Favorite user2 tags            | tag list              |

## Tags

| Option                         | Description               | Values  |
| ------------------------------ | ------------------------- | ------- |
| `getTagsAuto`                  | Auto-read file tags       | yes, no |
| `setTagsAuto`                  | Auto-write file tags      | yes, no |
| `coverDownload`                | Auto-download covers      | yes, no |
| `getTitleFromTags`             | Title/artist from tags    | yes, no |
| `getRatingFromTags`            | Rating from tags          | yes, no |
| `getCommentFromTags`           | Comment from tags         | yes, no |
| `getCuesFromTags`              | Cues from tags            | yes, no |
| `getTagFromZip`                | Read tags from ZIPs       | yes, no |
| `getRemixWhenParsingFilenames` | Parse remix from filename | yes, no |
| `useKeyFromTag`                | Prefer tag key            | yes, no |
| `cleanTagsInDeckDisplay`       | Uniform tag formatting    | yes, no |

## Automix

| Option                    | Description         | Values                      |
| ------------------------- | ------------------- | --------------------------- |
| `automixMode`             | Automix mode        | fade, smart, tempo          |
| `fadeLength`              | Fade length         | seconds (negative = delay)  |
| `automixRepeat`           | Repeat playlist     | yes, no                     |
| `automixAutoRemovePlayed` | Remove played songs | yes, no, always             |
| `automixDualDeck`         | Use both decks      | yes, no                     |
| `autoMixBeatMatchOnFade`  | Beatmatch on fade   | yes, no                     |
| `automixSkipLength`       | Skip fade length    | beats                       |
| `automixMaxLength`        | Max song play time  | seconds                     |
| `automixDoubleClick`      | Double-click action | action                      |
| `automixTempoMode`        | Tempo handling      | reset, keep pitch, keep bpm |

## Internet

| Option                      | Description               | Values             |
| --------------------------- | ------------------------- | ------------------ |
| `internetProxyURL`          | Proxy URL                 | URL                |
| `internetProxyPort`         | Proxy port                | port               |
| `internetProxyUsername`     | Proxy username            | username           |
| `internetProxyPassword`     | Proxy password            | password           |
| `stayLoggedIn`              | Stay logged in            | yes, no            |
| `dontLogin`                 | Don't auto-login          | yes, no            |
| `checkUpdates`              | Update check behavior     | mode               |
| `earlyAccessUpdates`        | Early access updates      | yes, no            |
| `sendHistory`               | Save playlists online     | yes, no            |
| `sendAnonymousStats`        | Send anonymous stats      | yes, no            |
| `autoRefreshDRM`            | Auto-refresh DRM          | yes, no            |
| `liveFeedback`              | Show live recommendations | yes, no            |
| `liveFeedbackUseBpm`        | Prioritize similar BPM    | yes, no            |
| `netsearchVideoQuality`     | Online video quality      | quality level      |
| `netsearchAudioQuality`     | Online audio quality      | quality level      |
| `cloudDriveEngine`          | CloudDrive storage        | engine name        |
| `cloudDriveSynchronization` | Sync mode                 | one-way, two-way   |
| `cloudDriveFullAccess`      | Full access mode          | yes, no            |
| `cloudDriveSync`            | Synced folders            | folder list        |
| `cloudDrivePauseSync`       | Pause sync                | yes, no            |
| `iRemote`                   | v7 remote app             | yes, no            |
| `iRemoteList`               | v7 remote clients         | client list        |
| `iRemoteDefaultPort`        | v7 remote port            | port               |
| `vdjRemoteDevices`          | v8+ remote devices        | device list        |
| `vdjRemoteIPs`              | Manual remote IPs         | IP list            |
| `os2l`                      | OS2L DMX sync             | yes, no            |
| `os2lDirectIp`              | OS2L direct IP:port       | IP:port            |
| `os2lBeatOffset`            | OS2L beat offset          | ms                 |
| `askTheDJMonitoring`        | AskTheDJ polling          | always, when shown |
| `askTheDJFrequency`         | AskTheDJ update frequency | seconds            |

## Recording

| Option                            | Description             | Values                         |
| --------------------------------- | ----------------------- | ------------------------------ |
| `recordFile`                      | Record filename         | filename                       |
| `recordFormat`                    | Record format           | mp3, ogg, flac, wav, webm, mp4 |
| `recordQuality`                   | Compression quality     | %                              |
| `recordAutoStart`                 | Auto-start on play      | yes, no                        |
| `recordWaitForSound`              | Wait for sound          | yes, no                        |
| `recordPauseOnSilence`            | Pause on silence        | yes, no                        |
| `recordOverwrite`                 | Overwrite behavior      | mode                           |
| `recordAutoSplit`                 | Auto-split on crossfade | yes, no                        |
| `recordWriteCueFile`              | Write .cue file         | yes, no                        |
| `recordVideoResolution`           | Video resolution        | resolution                     |
| `recordVideoHardwareAcceleration` | Video HW acceleration   | yes, no                        |
| `recordMicrophone`                | Record microphone       | yes, no                        |
| `recordBitDepth`                  | Audio bit depth         | 16, 24                         |
| `recordVideoFps`                  | Video FPS               | fps                            |
| `recordVideoCodec`                | Video codec             | h264, h265, av1                |

## Broadcasting

| Option                        | Description          | Values                     |
| ----------------------------- | -------------------- | -------------------------- |
| `broadcastMode`               | Broadcast mode       | direct, server, podcast    |
| `broadcastVideoQuality`       | Video quality        | %                          |
| `broadcastVideoQualityCustom` | Custom video quality | res @ videokbps, audiokbps |
| `broadcastServer`             | Server address       | URL                        |
| `broadcastDirectFormat`       | Direct format        | ogg, mp3                   |
| `broadcastDirectPort`         | Direct port          | port                       |
| `broadcastDirectQuality`      | Direct quality       | %                          |
| `broadcastDirectMaxClients`   | Max direct clients   | number                     |
| `broadcastDirectName`         | Broadcast name       | name                       |
| `broadcastSongInfo`           | Show song info       | yes, no                    |
| `broadcastSongInfoFormat`     | Song info format     | format string              |
| `podcastName`                 | Podcast name         | name                       |
| `broadcastVideoProvider`      | Video provider       | provider                   |
| `broadcastVideoURL`           | Video server URL     | URL                        |
| `broadcastVideoKey`           | Stream keys          | keys                       |

## General Options

| Option                     | Description              | Values                 |
| -------------------------- | ------------------------ | ---------------------- |
| `ABtesting`                | A/B testing              | config                 |
| `language`                 | Language file            | filename               |
| `loadSecurity`             | Load confirmation        | yes, no, silent        |
| `endOfSongWarning`         | Warning time             | seconds                |
| `autoDiscMarker`           | Auto-reset disc marker   | yes, no                |
| `sandboxSplitHeadphones`   | Sandbox split headphones | yes, no                |
| `sandboxPreviewOnly`       | Sandbox preview only     | yes, no                |
| `VDJScriptGlobalVariables` | Persistent variables     | variable list          |
| `crashGuard`               | Crash guard              | yes, no                |
| `crashReportLevel`         | Crash report level       | level                  |
| `poiEditorShowAll`         | Show system POIs         | yes, no                |
| `poiEditorSnap`            | Snap to beat             | yes, no                |
| `nonColoredPoi`            | Default POI color        | color                  |
| `colorPicker`              | Color picker type        | auto, gradient, simple |
| `settingPage`              | Last settings page       | page                   |
| `dontShowAgain`            | Hidden dialogs           | dialog list            |
| `vstFxFolder`              | VST3 folder              | folder path            |
| `showTipOfTheDay`          | Show tips                | yes, no                |
| `tipOfTheDayAlreadySeen`   | Seen tips                | tip list               |
| `skinStarterTip`           | Starter tip              | tip                    |
| `startOfDayHour`           | Day start hour           | hour (default: 8)      |
| `automaticDatabaseBackup`  | Auto-backup period       | days (0 = off)         |
| `databaseBackupLocation`   | Backup folder            | folder path            |
| `watchFolders`             | Auto-scan folders        | folder list            |

## Performance

| Option                     | Description                 | Values                                         |
| -------------------------- | --------------------------- | ---------------------------------------------- |
| `stemsRealtimeSeparation`  | Stems separation mode       | always, on-demand, prepared, reduced, disabled |
| `stemsSavedStems`          | Save prepared stems         | yes, no                                        |
| `stemsGPU`                 | GPU for stems               | yes, no, auto                                  |
| `stemsFix`                 | Stems fix for old PCs       | fix options                                    |
| `stemsSavedFolder`         | Prepared stems folder       | folder path                                    |
| `skinUseLowPowerGPU`       | Use low-power GPU           | yes, no, auto                                  |
| `skinFPS`                  | Skin frame rate             | fps                                            |
| `sampleRate`               | Internal sample rate        | auto, 44100, 48000                             |
| `latency`                  | Audio latency               | samples                                        |
| `ultraLatency`             | Ultra-low latency (ASIO)    | yes, no                                        |
| `maxPreloadLength`         | Max preload length          | minutes (-1 = always)                          |
| `maxStemLength`            | Max stem preload length     | minutes (-1 = always)                          |
| `pitchQuality`             | Pitch algorithm quality     | 1-4                                            |
| `scratchFilterQuality`     | Scratch filter quality      | 1-32 (default: 12)                             |
| `songLoadPriority`         | Song load priority          | low, normal                                    |
| `experimentalBeatAnalyzer` | Experimental analyzer       | yes, no                                        |
| `experimentalSkinEngine`   | Experimental skin engine    | yes, no                                        |
| `useOpengl`                | Use OpenGL vs Metal         | yes, no                                        |
| `experimentalWaveColors`   | Experimental colors         | yes, no                                        |
| `safeVideoDecoding`        | Safe video decode           | yes, no                                        |
| `analyzeSongsOnView`       | Auto-scan on browse         | yes, no                                        |
| `keepBPMonAnalyzerUpdate`  | Keep BPM on analyzer update | yes, no                                        |

### Stems Realtime Separation Modes

- **always**: Separate every song on load
- **on-demand**: Separate only when needed (small delay)
- **prepared**: Use prepared stems or reduced quality
- **reduced**: Always use reduced quality
- **disabled**: Never separate (no modern waveforms)

### Pitch Quality

- **1**: Fastest
- **2**: Good
- **3**: Excellent
- **4**: Best (requires fast CPU)

## Usage Notes

### Setting Values

Access settings via:

```
setting "optionName" value
setting "optionName"  // query
```

### Common Value Types

- **Boolean**: `yes`, `no`, `on`, `off`, `true`, `false`
- **Percentage**: `50%`
- **Time**: `1000ms`, `2bt` (beats)
- **Path**: Absolute file/folder paths
- **List**: Comma-separated values

### Configuration Files

Settings stored in:

- **Windows**: `%LOCALAPPDATA%\VirtualDJ\`
- **Mac**: `~/Library/Application Support/VirtualDJ/`
