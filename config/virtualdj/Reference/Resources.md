# VirtualDJ Resources

Useful source links for maintaining this repo's VirtualDJ skin, pad, effect, and scripting notes.

## Source Policy

- Prefer current VirtualDJ manual and VDJPedia pages for syntax and supported behavior.
- Treat VirtualDJ forum posts as official only when the answer is from VirtualDJ staff, Development Manager, CTO, or Support staff.
- Treat community skins, forum replies, Reddit posts, and GitHub snippets as examples to verify locally, not as authority.
- Keep local conclusions labeled as `Inference` when they combine official docs with repo testing.

## Official Documentation

- [VirtualDJ Manual](https://www.virtualdj.com/manuals/virtualdj.html) - Current user manual.
- [VDJPedia](https://www.virtualdj.com/wiki/) - Wiki-style documentation and SDK pages.
- [Developer SDK](https://virtualdj.com/wiki/developers.html) - Entry point for skin, plugin, controller, and tool development.
- [Skin SDK](https://virtualdj.com/wiki/skin%20sdk%20.html) - Official skin package and element overview.
- [Skin Browser](https://www.virtualdj.com/wiki/Skin%2BBrowser.html) - `<browser>` element reference.
- [Custom Browser](https://virtualdj.com/wiki/custombrowser.html) - Official decomposition of the default browser into custom skin elements.
- [Split Panel](https://www.virtualdj.com/wiki/Split%20Panel.html) - `<split>` layout panels.
- [Skin Button](https://www.virtualdj.com/wiki/Skin%20Button.html) - Button actions, mouse handlers, and state graphics.
- [Skin SDK Dropzone](https://www.virtualdj.com/wiki/Skin%20SDK%20Dropzone.html) - Drag/drop target element.
- [Skin Panel](https://www.virtualdj.com/wiki/Skin%20SDK%20Panel.html) - Query-driven and named panels.
- [Skin Default Colors](https://virtualdj.com/wiki/Skin%20Default%20Colors.html) - Static and dynamic color handling.
- [Skin SDK Visual](https://virtualdj.com/wiki/skinsdkvisual.html) - Dynamic visuals, including color visuals.
- [VDJScript](https://www.virtualdj.com/wiki/VDJ%20Script.html) - Language overview.
- [VDJScript Verbs](https://www.virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html) - Current verb reference.
- [VDJScript Examples](https://www.virtualdj.com/wiki/VDJScript%20Examples) - Official example scripts.
- [Options List](https://www.virtualdj.com/manuals/virtualdj/appendix/optionslist/) - Settings/options appendix.
- [Lyrics Editor](https://www.virtualdj.com/manuals/virtualdj/editors/lyricseditor.html) - AI-generated lyric editing, re-analysis, and censoring UI.
- [Stems Help](https://de.virtualdj.com/help/stems.html) - Official stems overview, stem controls, preparation behavior, and five-stem terminology.
- [VDJPedia Lists](https://de.virtualdj.com/wiki/Lists.html) - XML list and virtual folder structure for VirtualDJ 2024 and onwards.
- [Native Effects](https://www.virtualdj.com/manuals/virtualdj/appendix/nativeeffects/) - Built-in effect reference.
- [Pads Manual](https://www.virtualdj.com/manuals/virtualdj/interface/decks/decksadvanced/pads.html) - Pad pages and pad behavior.
- [Sampler Manual](https://www.virtualdj.com/manuals/virtualdj/interface/browser/sideview/sampler.html) - Sampler sideview, banks, pages, and drag/drop behavior.
- [How to Install Plugins and Addons](https://virtualdj.zendesk.com/hc/en-us/articles/360004467797-How-do-I-download-and-install-new-skins-effects-samples-etc) - Official support article for extensions.

## Official Forums And Staff Guidance

- [VirtualDJ Skins forum](https://www.virtualdj.com/forums/13/VirtualDJ_Skins.html) - Best place to search for skin engine behavior, staff clarifications, and examples.
- [VirtualDJ Technical Support forum](https://www.virtualdj.com/forums/2/VirtualDJ_Technical_Support.html) - Good for scripting, browser, sampler, and effect behavior questions.
- [VirtualDJ 2020 - Additions in Skin Engine](https://www.virtualdj.com/forums/230926/VirtualDJ_Skins/VirtualDJ_2020_-_Additions_in_Skin_Engine.html) - Staff-maintained thread for skin engine additions.
- [Border Color using placeholder](https://virtualdj.com/forums/242871/VirtualDJ_Skins/Border_Color_using_placeholder.html) - Staff clarification that dynamic button border colors are not supported.
- [Skin text action; visibility or visual?](https://www.virtualdj.com/forums/267953/VirtualDJ_Skins/Skin_text_action%3B_visibility_or_visual%3F.html) - Staff guidance for dynamic skin text color.
- [effect_colorfx & effect_stems_color ?](https://www.virtualdj.com/forums/241078/VirtualDJ_Technical_Support/effect_colorfx___effect_stems_color__.html) - Staff discussion of extra ColorFX controls.
- [Default filter and color fx filter](https://virtualdj.com/forums/252675/VirtualDJ_Technical_Support/Default_filter_and_color_fx_filter.html) - Staff guidance on filter and ColorFX behavior.
- [Aditional xml for Skins](https://virtualdj.com/forums/248589/Wishes_and_new_features/Aditional_xml_for_Skins.html) - Staff/forum context for runtime XML includes versus build-time composition.
- [Virtual DJ 2026](https://www.virtualdj.com/forums/266311/VirtualDJ_Technical_Support/Virtual_DJ_2026.html) - Staff launch-thread guidance on AI lyrics, prepared stems, re-analysis, and long-track limits.
- [How Lyrics are analyzed???](https://virtualdj.com/forums/267223/VirtualDJ_Technical_Support/How_Lyrics_are_analyzed%3F%3F%3F.html) - Staff explanation of lyric audio signatures, stems requirement, server cache, and local edits.
- [Lyrics issues with stems VDJ 2026](https://virtualdj.com/forums/266347/VirtualDJ_Technical_Support/Lyrics_issues_with_stems_VDJ_2026.html) - Early lyric/stems behavior reports and censor matching discussion.
- [Stems 2.0](https://virtualdj.com/forums/266488/VirtualDJ_Technical_Support/Stems_2.0.html) - Forum guidance around the Stems 2.0 requirement message for lyric extraction.
- [Where can I find the database.xml file on MacOS?](https://www.virtualdj.com/forums/261193/VirtualDJ_Technical_Support/Where_can_I_find_the_database_xml_file_on_MacOS_.html) - CTO reply plus user-confirmed macOS `database.xml` path under `~/Library/Application Support/VirtualDJ`.
- [Machine specific settings.xml and licence.dat?](https://www.virtualdj.com/forums/223863/VirtualDJ_Technical_Support/Machine_specific_settings_xml_and_licence_dat_.html) - Staff explanation of the VirtualDJ home folder, master database, and per-drive local databases.
- [Bug in Instant Filters > Has Lyrics](https://virtualdj.com/forums/267592/VirtualDJ_Technical_Support/Bug_in_Instant_Filters_%3E_Has_Lyrics.html) - Forum workaround for "Has Lyrics" filter state matching.
- [Undocumented scripts](https://virtualdj.com/forums/213099/VirtualDJ_Technical_Support/Undocumented_scripts.html) - Staff clarification of old `effect X active` and `var 'name' X` syntax.
- [VDJ Script Verbs update](https://www.virtualdj.com/forums/205590/VirtualDJ_Skins/VDJ_Script_Verbs_update.html) - Staff clarification that skins and controllers share verbs, with the in-app editor as a discovery source.
- [Where can I find a VDJ script reference?](https://virtualdj.com/forums/264082/VirtualDJ_Technical_Support/Where_can_I_find_a_VDJ_script_reference%253F.html) - Staff/community discussion of VDJScript's intentionally concise reference style and debugging approaches.

## Community And Unofficial Sources

- [VirtualDJ Extensions](https://www.virtualdj.com/addons/) - Officially hosted but community-contributed skins, effects, pads, samples, and mappings. Useful for studying patterns; verify syntax before copying.
- [VirtualDJ Skins forum, non-staff posts](https://www.virtualdj.com/forums/13/VirtualDJ_Skins.html) - Useful examples and troubleshooting, but source-label as `Community` unless staff confirms the behavior.
- [r/virtualdj](https://www.reddit.com/r/virtualdj/) - Broad community troubleshooting. Useful for symptoms and user workflows, low authority for SDK details.
- [GitHub code search: VirtualDJ skin XML](https://github.com/search?q=VirtualDJ+skin.xml&type=code) - Occasional public examples and tooling. Check licenses and verify against current official docs.
- [Matroska Stem Files Internet-Draft](https://www.ietf.org/archive/id/draft-swhited-mka-stems-06.html) - External container-format context for multi-track stem files.

## Local Repo References

- [VirtualDJ Reference](VirtualDJ%20Reference.md) - Source-backed overview and preferred local patterns.
- [Lyrics AI and Skins](Lyrics%20AI%20and%20Skins.md) - AI lyrics, skin styling surface, filters, and useful script quirks.
- [Application Internals](Application%20Internals.md) - Low-level macOS paths, file formats, databases, stem sidecars, and shell examples.
- [Skin SDK](Skin%20SDK.md) - Local skin SDK reference.
- [VDJScript Verbs](VDJScript%20Verbs.md) - Curated verb notes.
- [Filter Syntax](Filter%20Syntax.md) - Browser filter notes.
- [Example Skin XML Objects](Example%20Skin%20XML%20Objects.md) - Local skin XML examples.
- [GraveRaver source skin](../Skins/GraveRaver/src/skin.xml) - Active modular skin source in this repo.
- [GraveRaver build file](../Skins/GraveRaver/justfile) - Build-time XInclude workflow.
- [ModularSkeleton build skin](../Skins/ModularSkeleton/build/skin.xml) - Minimal modular skin scaffold output.
