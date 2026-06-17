Found 1528 mutants to test
ok       Unmutated baseline in 61s build + 9s test
MISSED   src/settings.rs:219:26: replace < with <= in SettingsModal::field_value in 3s build + 13s test
MISSED   src/settings.rs:239:31: replace < with <= in SettingsModal::commit_input in 3s build + 11s test
MISSED   src/settings.rs:294:13: delete match arm 2 in SettingsModal::toggle_current in 2s build + 14s test
MISSED   src/settings.rs:283:21: delete match arm 1 in SettingsModal::toggle_current in 4s build + 12s test
MISSED   src/settings.rs:289:21: delete match arm 3 in SettingsModal::toggle_current in 3s build + 14s test
MISSED   src/settings.rs:285:68: delete ! in SettingsModal::toggle_current in 3s build + 12s test
MISSED   src/settings.rs:289:46: delete ! in SettingsModal::toggle_current in 3s build + 15s test
MISSED   src/settings.rs:296:21: delete match arm 0 in SettingsModal::toggle_current in 3s build + 15s test
MISSED   src/settings.rs:297:21: delete match arm 1 in SettingsModal::toggle_current in 3s build + 15s test
MISSED   src/settings.rs:296:63: delete ! in SettingsModal::toggle_current in 3s build + 13s test
MISSED   src/settings.rs:299:29: delete ! in SettingsModal::toggle_current in 4s build + 13s test
MISSED   src/settings.rs:311:9: replace SettingsModal::start_editing with () in 6s build + 13s test
MISSED   src/settings.rs:317:9: replace SettingsModal::cancel_editing with () in 4s build + 12s test
MISSED   src/settings.rs:359:46: replace - with + in SettingsModal::move_down in 3s build + 17s test
MISSED   src/settings.rs:359:46: replace - with / in SettingsModal::move_down in 3s build + 14s test
MISSED   src/settings.rs:359:42: replace + with - in SettingsModal::move_down in 3s build + 16s test
MISSED   src/settings.rs:359:42: replace + with * in SettingsModal::move_down in 4s build + 15s test
MISSED   src/settings.rs:368:27: replace < with == in SettingsModal::move_up in 3s build + 15s test
MISSED   src/settings.rs:368:27: replace < with > in SettingsModal::move_up in 4s build + 14s test
MISSED   src/settings.rs:368:27: replace < with <= in SettingsModal::move_up in 4s build + 13s test
MISSED   src/settings.rs:385:9: replace SettingsModal::prev_tab with () in 3s build + 15s test
MISSED   src/settings.rs:385:53: replace % with / in SettingsModal::prev_tab in 4s build + 17s test
MISSED   src/settings.rs:385:53: replace % with + in SettingsModal::prev_tab in 4s build + 16s test
MISSED   src/settings.rs:385:48: replace - with + in SettingsModal::prev_tab in 4s build + 14s test
MISSED   src/settings.rs:385:48: replace - with / in SettingsModal::prev_tab in 5s build + 14s test
MISSED   src/settings.rs:385:30: replace + with - in SettingsModal::prev_tab in 4s build + 15s test
MISSED   src/settings.rs:385:30: replace + with * in SettingsModal::prev_tab in 5s build + 16s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with None in 3s build + 16s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((0, 0, 0)) in 5s build + 15s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((0, 0, 1)) in 4s build + 16s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((0, 1, 0)) in 4s build + 15s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((0, 1, 1)) in 4s build + 15s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((1, 0, 0)) in 4s build + 16s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((1, 0, 1)) in 4s build + 17s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((1, 1, 0)) in 4s build + 17s test
MISSED   src/settings.rs:409:9: replace SettingsModal::input_rgb -> Option<(u8, u8, u8)> with Some((1, 1, 1)) in 4s build + 18s test
MISSED   src/settings.rs:439:13: delete match arm (KeyModifiers::NONE, KeyCode::Esc) in handle_settings_key in 3s build + 16s test
MISSED   src/settings.rs:442:13: delete match arm (KeyModifiers::NONE, KeyCode::Enter) in handle_settings_key in 5s build + 17s test
MISSED   src/settings.rs:450:13: delete match arm (KeyModifiers::NONE, KeyCode::Backspace) in handle_settings_key in 4s build + 16s test
MISSED   src/settings.rs:454:20: replace match guard !mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) with true in handle_settings_key in 5s build + 16s test
MISSED   src/settings.rs:454:20: replace match guard !mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) with false in handle_settings_key in 5s build + 14s test
MISSED   src/settings.rs:454:20: delete ! in handle_settings_key in 5s build + 17s test
MISSED   src/settings.rs:454:79: replace | with & in handle_settings_key in 5s build + 15s test
MISSED   src/settings.rs:454:79: replace | with ^ in handle_settings_key in 4s build + 17s test
MISSED   src/settings.rs:454:59: replace | with & in handle_settings_key in 5s build + 16s test
MISSED   src/settings.rs:454:59: replace | with ^ in handle_settings_key in 4s build + 18s test
MISSED   src/settings.rs:466:9: delete match arm (KeyModifiers::NONE, KeyCode::Esc) |(KeyModifiers::NONE, KeyCode::F(2)) in handle_settings_key in 5s build + 18s test
MISSED   src/settings.rs:470:9: delete match arm (KeyModifiers::NONE, KeyCode::Tab) in handle_settings_key in 4s build + 19s test
MISSED   src/settings.rs:474:9: delete match arm (KeyModifiers::SHIFT, KeyCode::Tab) in handle_settings_key in 4s build + 19s test
MISSED   src/settings.rs:478:9: delete match arm (KeyModifiers::NONE, KeyCode::Right) in handle_settings_key in 4s build + 17s test
MISSED   src/settings.rs:486:9: delete match arm (KeyModifiers::NONE, KeyCode::Left) in handle_settings_key in 5s build + 17s test
MISSED   src/settings.rs:496:9: delete match arm (KeyModifiers::NONE, KeyCode::Down) in handle_settings_key in 4s build + 17s test
MISSED   src/settings.rs:500:9: delete match arm (KeyModifiers::NONE, KeyCode::Up) in handle_settings_key in 4s build + 21s test
MISSED   src/settings.rs:506:9: delete match arm (KeyModifiers::NONE, KeyCode::Enter) |(KeyModifiers::NONE, KeyCode::Char(' ')) in handle_settings_key in 5s build + 16s test
MISSED   src/settings.rs:509:31: replace && with || in handle_settings_key in 4s build + 18s test
MISSED   src/settings.rs:509:26: replace == with != in handle_settings_key in 4s build + 17s test
MISSED   src/settings.rs:509:46: replace == with != in handle_settings_key in 4s build + 19s test
MISSED   src/settings.rs:510:34: delete ! in handle_settings_key in 4s build + 20s test
MISSED   src/settings.rs:511:36: replace && with || in handle_settings_key in 4s build + 20s test
MISSED   src/settings.rs:511:20: delete ! in handle_settings_key in 4s build + 20s test
MISSED   src/settings.rs:511:51: replace > with == in handle_settings_key in 6s build + 21s test
MISSED   src/settings.rs:511:51: replace > with < in handle_settings_key in 4s build + 20s test
MISSED   src/settings.rs:511:51: replace > with >= in handle_settings_key in 4s build + 17s test
MISSED   src/settings.rs:534:21: delete match arm FieldKind::Toggle | FieldKind::Cycler{..} in handle_settings_key in 4s build + 18s test
MISSED   src/settings.rs:575:9: delete match arm 3 in appearance_core_value in 4s build + 19s test
MISSED   src/settings.rs:576:9: delete match arm 4 in appearance_core_value in 4s build + 17s test
MISSED   src/settings.rs:577:9: delete match arm 5 in appearance_core_value in 4s build + 18s test
MISSED   src/settings.rs:578:9: delete match arm 6 in appearance_core_value in 4s build + 18s test
MISSED   src/settings.rs:579:9: delete match arm 7 in appearance_core_value in 4s build + 20s test
MISSED   src/settings.rs:586:9: delete match arm 0 in appearance_extended_value in 7s build + 24s test
MISSED   src/settings.rs:587:9: delete match arm 1 in appearance_extended_value in 4s build + 20s test
MISSED   src/settings.rs:588:9: delete match arm 2 in appearance_extended_value in 4s build + 20s test
MISSED   src/settings.rs:589:9: delete match arm 3 in appearance_extended_value in 4s build + 19s test
MISSED   src/settings.rs:590:9: delete match arm 4 in appearance_extended_value in 6s build + 21s test
MISSED   src/settings.rs:591:9: delete match arm 5 in appearance_extended_value in 4s build + 24s test
MISSED   src/settings.rs:592:9: delete match arm 6 in appearance_extended_value in 5s build + 17s test
MISSED   src/settings.rs:593:9: delete match arm 7 in appearance_extended_value in 5s build + 24s test
MISSED   src/settings.rs:594:9: delete match arm 8 in appearance_extended_value in 5s build + 23s test
MISSED   src/settings.rs:595:9: delete match arm 9 in appearance_extended_value in 6s build + 23s test
MISSED   src/settings.rs:596:9: delete match arm 10 in appearance_extended_value in 5s build + 22s test
MISSED   src/settings.rs:597:9: delete match arm 11 in appearance_extended_value in 5s build + 22s test
MISSED   src/settings.rs:598:9: delete match arm 12 in appearance_extended_value in 4s build + 22s test
MISSED   src/settings.rs:599:9: delete match arm 13 in appearance_extended_value in 9s build + 23s test
MISSED   src/settings.rs:600:9: delete match arm 14 in appearance_extended_value in 5s build + 28s test
MISSED   src/settings.rs:601:9: delete match arm 15 in appearance_extended_value in 5s build + 21s test
MISSED   src/settings.rs:603:9: delete match arm 17 in appearance_extended_value in 5s build + 26s test
MISSED   src/settings.rs:604:9: delete match arm 18 in appearance_extended_value in 6s build + 25s test
MISSED   src/settings.rs:605:9: delete match arm 19 in appearance_extended_value in 6s build + 19s test
MISSED   src/settings.rs:606:9: delete match arm 20 in appearance_extended_value in 6s build + 25s test
MISSED   src/settings.rs:607:9: delete match arm 21 in appearance_extended_value in 5s build + 28s test
MISSED   src/settings.rs:608:9: delete match arm 22 in appearance_extended_value in 9s build + 40s test
MISSED   src/settings.rs:609:9: delete match arm 23 in appearance_extended_value in 9s build + 30s test
MISSED   src/settings.rs:610:9: delete match arm 24 in appearance_extended_value in 5s build + 24s test
MISSED   src/settings.rs:611:9: delete match arm 25 in appearance_extended_value in 5s build + 24s test
MISSED   src/settings.rs:612:9: delete match arm 26 in appearance_extended_value in 5s build + 26s test
MISSED   src/settings.rs:620:9: delete match arm 0 in editor_value in 12s build + 27s test
MISSED   src/settings.rs:621:9: delete match arm 1 in editor_value in 7s build + 30s test
MISSED   src/settings.rs:622:9: delete match arm 2 in editor_value in 6s build + 25s test
MISSED   src/settings.rs:623:9: delete match arm 3 in editor_value in 8s build + 39s test
MISSED   src/settings.rs:624:9: delete match arm 4 in editor_value in 9s build + 26s test
MISSED   src/settings.rs:628:9: delete match arm 6 in editor_value in 11s build + 36s test
MISSED   src/settings.rs:631:9: delete match arm 7 in editor_value in 11s build + 28s test
MISSED   src/settings.rs:640:9: delete match arm 0 in file_value in 6s build + 21s test
MISSED   src/settings.rs:641:9: delete match arm 1 in file_value in 7s build + 23s test
MISSED   src/settings.rs:642:9: delete match arm 2 in file_value in 5s build + 33s test
MISSED   src/settings.rs:644:9: delete match arm 4 in file_value in 9s build + 33s test
MISSED   src/settings.rs:686:9: delete match arm 1 in commit_appearance_core in 10s build + 29s test
MISSED   src/settings.rs:688:9: delete match arm 3 in commit_appearance_core in 10s build + 49s test
MISSED   src/settings.rs:689:9: delete match arm 4 in commit_appearance_core in 10s build + 42s test
MISSED   src/settings.rs:690:9: delete match arm 5 in commit_appearance_core in 10s build + 55s test
MISSED   src/settings.rs:691:9: delete match arm 6 in commit_appearance_core in 9s build + 49s test
MISSED   src/settings.rs:692:9: delete match arm 7 in commit_appearance_core in 9s build + 29s test
MISSED   src/settings.rs:699:9: delete match arm 0 in commit_appearance_extended in 11s build + 51s test
MISSED   src/settings.rs:700:9: delete match arm 1 in commit_appearance_extended in 10s build + 31s test
MISSED   src/settings.rs:701:9: delete match arm 2 in commit_appearance_extended in 10s build + 47s test
MISSED   src/settings.rs:702:9: delete match arm 3 in commit_appearance_extended in 9s build + 49s test
MISSED   src/settings.rs:703:9: delete match arm 4 in commit_appearance_extended in 9s build + 49s test
MISSED   src/settings.rs:704:9: delete match arm 5 in commit_appearance_extended in 9s build + 52s test
MISSED   src/settings.rs:705:9: delete match arm 6 in commit_appearance_extended in 10s build + 52s test
MISSED   src/settings.rs:706:9: delete match arm 7 in commit_appearance_extended in 10s build + 45s test
MISSED   src/settings.rs:707:9: delete match arm 8 in commit_appearance_extended in 10s build + 45s test
MISSED   src/settings.rs:708:9: delete match arm 9 in commit_appearance_extended in 10s build + 41s test
MISSED   src/settings.rs:709:9: delete match arm 10 in commit_appearance_extended in 9s build + 39s test
MISSED   src/settings.rs:710:9: delete match arm 11 in commit_appearance_extended in 10s build + 42s test
MISSED   src/settings.rs:711:9: delete match arm 12 in commit_appearance_extended in 9s build + 33s test
MISSED   src/settings.rs:712:9: delete match arm 13 in commit_appearance_extended in 9s build + 54s test
MISSED   src/settings.rs:713:9: delete match arm 14 in commit_appearance_extended in 10s build + 56s test
MISSED   src/settings.rs:714:9: delete match arm 15 in commit_appearance_extended in 10s build + 55s test
MISSED   src/settings.rs:722:9: delete match arm 17 in commit_appearance_extended in 13s build + 53s test
MISSED   src/settings.rs:723:9: delete match arm 18 in commit_appearance_extended in 16s build + 57s test
MISSED   src/settings.rs:724:9: delete match arm 19 in commit_appearance_extended in 10s build + 56s test
MISSED   src/settings.rs:725:9: delete match arm 20 in commit_appearance_extended in 10s build + 57s test
TIMEOUT  src/settings.rs:726:9: delete match arm 21 in commit_appearance_extended in 10s build + 60s test
MISSED   src/settings.rs:727:9: delete match arm 22 in commit_appearance_extended in 15s build + 49s test
MISSED   src/settings.rs:728:9: delete match arm 23 in commit_appearance_extended in 10s build + 53s test
MISSED   src/settings.rs:729:9: delete match arm 24 in commit_appearance_extended in 9s build + 51s test
TIMEOUT  src/settings.rs:730:9: delete match arm 25 in commit_appearance_extended in 10s build + 60s test
TIMEOUT  src/settings.rs:731:9: delete match arm 26 in commit_appearance_extended in 11s build + 60s test
TIMEOUT  src/settings.rs:739:9: delete match arm 0..= 4 in commit_editor in 13s build + 60s test
TIMEOUT  src/settings.rs:740:9: delete match arm 5 in commit_editor in 16s build + 60s test
TIMEOUT  src/settings.rs:768:9: delete match arm 0 | 1 in commit_file in 14s build + 60s test
TIMEOUT  src/settings.rs:769:9: delete match arm 2 in commit_file in 13s build + 60s test
TIMEOUT  src/table_format.rs:252:44: replace && with || in compact_table in 14s build + 60s test
TIMEOUT  src/table_format.rs:252:30: replace > with >= in compact_table in 11s build + 60s test
TIMEOUT  src/input.rs:708:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('q')) |(KeyModifiers::NONE, KeyCode::Esc) in handle_key_event in 5s build + 60s test
MISSED   src/input.rs:721:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('x')) |(KeyModifiers::SUPER, KeyCode::Char('x')) in handle_key_event in 7s build + 41s test
MISSED   src/input.rs:802:9: delete match arm (KeyModifiers::NONE, KeyCode::F(2)) in handle_key_event in 5s build + 38s test
MISSED   src/input.rs:756:67: replace | with ^ in handle_key_event in 5s build + 37s test
MISSED   src/input.rs:889:47: replace != with == in handle_key_event in 6s build + 56s test
MISSED   src/input.rs:941:18: replace > with >= in detect_list_prefix in 5s build + 39s test
MISSED   src/input.rs:947:48: replace && with || in detect_list_prefix in 5s build + 33s test
MISSED   src/picker.rs:117:5: replace command_exists -> bool with false in 5s build + 38s test
MISSED   src/preview.rs:229:61: replace && with || in render_lines in 5s build + 39s test
MISSED   src/preview.rs:229:48: replace > with >= in render_lines in 5s build + 39s test
MISSED   src/preview.rs:229:77: replace < with <= in render_lines in 5s build + 39s test
MISSED   src/preview.rs:231:29: delete field char_start from struct StyledSpan expression in render_lines in 5s build + 36s test
MISSED   src/preview.rs:247:23: replace > with >= in render_lines in 6s build + 55s test
MISSED   src/preview.rs:297:5: replace preview_width -> usize with 1 in 5s build + 41s test
MISSED   src/preview.rs:299:14: replace > with == in preview_width in 5s build + 40s test
MISSED   src/preview.rs:299:14: replace > with < in preview_width in 5s build + 41s test
MISSED   src/preview.rs:299:14: replace > with >= in preview_width in 5s build + 40s test
TIMEOUT  src/decoration/builder/mod.rs:101:13: delete match arm Event::End(TagEnd::Heading(_)) in build_decoration_map in 18s build + 60s test
TIMEOUT  src/decoration/emit.rs:118:36: replace || with && in emit_bold_italic_spans in 21s build + 60s test
TIMEOUT  src/decoration/emit.rs:118:22: replace > with == in emit_bold_italic_spans in 22s build + 60s test
TIMEOUT  src/decoration/emit.rs:118:22: replace > with >= in emit_bold_italic_spans in 17s build + 60s test
TIMEOUT  src/decoration/emit.rs:138:22: replace > with >= in emit_bold_italic_spans in 20s build + 60s test
TIMEOUT  src/decoration/emit.rs:146:20: replace > with >= in emit_bold_italic_spans in 21s build + 60s test
TIMEOUT  src/decoration/emit.rs:154:22: replace > with >= in emit_bold_italic_spans in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:19:48: replace + with - in 22s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:19:48: replace + with * in 25s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:24:37: replace + with - in 35s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:24:37: replace + with * in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:199:5: replace draw_top_border with () in 36s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:203:25: replace + with - in draw_top_border in 35s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:203:25: replace + with * in draw_top_border in 29s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:203:21: replace + with - in draw_top_border in 24s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:203:21: replace + with * in draw_top_border in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:19: replace && with || in draw_top_border in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:14: replace >= with < in draw_top_border in 18s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:24: replace < with == in draw_top_border in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:24: replace < with > in draw_top_border in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:24: replace < with <= in draw_top_border in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:28: replace + with - in draw_top_border in 39s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:204:28: replace + with * in draw_top_border in 13s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:205:37: replace - with + in draw_top_border in 16s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:205:37: replace - with / in draw_top_border in 35s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:206:28: replace == with != in draw_top_border in 18s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:212:23: replace + with - in draw_top_border in 16s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:212:23: replace + with * in draw_top_border in 15s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:212:13: replace + with - in draw_top_border in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:212:13: replace + with * in draw_top_border in 38s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:216:5: replace draw_bottom_border with () in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:218:21: replace + with - in draw_bottom_border in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:218:21: replace + with * in draw_bottom_border in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:218:17: replace + with - in draw_bottom_border in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:218:17: replace + with * in draw_bottom_border in 39s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:220:23: replace + with - in draw_bottom_border in 35s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:220:23: replace + with * in draw_bottom_border in 18s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:220:13: replace + with - in draw_bottom_border in 27s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:220:13: replace + with * in draw_bottom_border in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:224:5: replace draw_hline with () in 25s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:225:25: replace - with + in draw_hline in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:225:25: replace - with / in draw_hline in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:226:17: replace + with - in draw_hline in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:226:17: replace + with * in draw_hline in 15s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:228:23: replace - with + in draw_hline in 15s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:228:23: replace - with / in draw_hline in 16s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:228:13: replace + with - in draw_hline in 16s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:228:13: replace + with * in draw_hline in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:242:5: replace draw_tab_bar with () in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:243:20: replace + with - in draw_tab_bar in 18s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:243:20: replace + with * in draw_tab_bar in 22s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:245:23: replace == with != in draw_tab_bar in 22s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:246:22: replace == with != in draw_tab_bar in 24s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:248:18: replace >= with < in draw_tab_bar in 27s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:248:24: replace + with - in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:248:24: replace + with * in draw_tab_bar in 24s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:255:15: replace += with -= in draw_tab_bar in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:255:15: replace += with *= in draw_tab_bar in 27s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:257:18: replace < with == in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:257:18: replace < with > in draw_tab_bar in 26s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:257:18: replace < with <= in draw_tab_bar in 25s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:257:14: replace + with - in draw_tab_bar in 31s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:257:14: replace + with * in draw_tab_bar in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:258:18: replace < with == in draw_tab_bar in 26s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:258:18: replace < with > in draw_tab_bar in 18s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:258:18: replace < with <= in draw_tab_bar in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:258:23: replace + with - in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:258:23: replace + with * in draw_tab_bar in 31s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:260:19: replace += with -= in draw_tab_bar in 25s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:260:19: replace += with *= in draw_tab_bar in 31s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:262:18: replace < with == in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:262:18: replace < with > in draw_tab_bar in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:262:18: replace < with <= in draw_tab_bar in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:262:23: replace + with - in draw_tab_bar in 29s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:262:23: replace + with * in draw_tab_bar in 26s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:264:19: replace += with -= in draw_tab_bar in 25s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:264:19: replace += with *= in draw_tab_bar in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:266:18: replace < with == in draw_tab_bar in 40s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:266:18: replace < with > in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:266:18: replace < with <= in draw_tab_bar in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:266:23: replace + with - in draw_tab_bar in 32s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:266:23: replace + with * in draw_tab_bar in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:268:19: replace += with -= in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:268:19: replace += with *= in draw_tab_bar in 17s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:272:23: replace + with - in draw_tab_bar in 23s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:272:23: replace + with * in draw_tab_bar in 19s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:272:13: replace + with - in draw_tab_bar in 36s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:272:13: replace + with * in draw_tab_bar in 47s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:277:5: replace put_padded with () in 44s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:280:16: replace + with - in put_padded in 27s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:280:16: replace + with * in put_padded in 30s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:281:21: replace + with - in put_padded in 22s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:281:21: replace + with * in put_padded in 46s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:284:16: replace + with - in put_padded in 48s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:284:16: replace + with * in put_padded in 24s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:290:5: replace put_str_clipped with () in 15s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:291:16: replace + with - in put_str_clipped in 20s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:291:16: replace + with * in put_str_clipped in 26s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:296:5: replace put_str with () in 41s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:297:16: replace + with - in put_str in 46s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:297:16: replace + with * in put_str in 31s build + 60s test
TIMEOUT  src/renderer/settings_modal.rs:303:5: replace put_swatch with () in 21s build + 60s test
TIMEOUT  src/renderer/status.rs:190:5: replace build_save_as_bar -> Line<'static> with Default::default() in 31s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:56: replace + with - in 32s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:56: replace + with * in 29s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:51: replace + with - in 31s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:51: replace + with * in 30s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:47: replace + with - in 24s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:47: replace + with * in 24s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:42: replace + with - in 36s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:42: replace + with * in 29s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:38: replace + with - in 31s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:38: replace + with * in 39s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:33: replace + with - in 29s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:33: replace + with * in 33s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:29: replace + with - in 21s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:29: replace + with * in 24s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:193:24: replace + with * in 19s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:195:28: replace + with - in 34s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:195:28: replace + with * in 35s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:206:38: replace + with - in 32s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:206:38: replace + with * in 22s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:397:46: replace + with - in 17s build + 60s test
TIMEOUT  src/renderer/search_bar.rs:397:46: replace + with * in 19s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:42:17: delete field char_start from struct StyledSpan expression in on_blockquote_start in 28s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:78:34: replace + with * in on_item_start in 29s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:112:41: replace > with >= in on_task_marker in 27s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:128:49: replace + with - in on_task_marker in 29s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:131:28: replace < with <= in on_task_marker in 26s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:131:24: replace + with - in on_task_marker in 26s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:131:24: replace + with * in on_task_marker in 28s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:139:28: replace < with > in on_task_marker in 39s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:139:28: replace < with <= in on_task_marker in 26s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:139:24: replace + with - in on_task_marker in 19s build + 60s test
TIMEOUT  src/decoration/builder/blocks.rs:147:24: replace < with <= in on_task_marker in 38s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:113:17: delete field char_start from struct StyledSpan expression in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:116:17: delete field full_line_bg from struct StyledSpan expression in on_start in 31s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:122:41: replace + with * in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:123:25: replace > with >= in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:138:46: replace + with * in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:150:43: replace + with * in on_start in 31s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:154:31: replace match guard !hl_spans.is_empty() with true in on_start in 33s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:154:31: replace match guard !hl_spans.is_empty() with false in on_start in 31s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:154:31: delete ! in on_start in 29s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:165:27: replace < with == in on_start in 31s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:165:27: replace < with > in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:165:27: replace < with <= in on_start in 32s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:170:33: delete field char_start from struct StyledSpan expression in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:171:33: delete field char_end from struct StyledSpan expression in on_start in 25s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:172:33: delete field style from struct StyledSpan expression in on_start in 28s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:175:33: delete field full_line_bg from struct StyledSpan expression in on_start in 32s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:175:52: replace == with != in on_start in 34s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:192:25: delete field char_start from struct StyledSpan expression in on_start in 29s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:204:17: replace > with >= in on_start in 30s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:217:17: delete field char_start from struct StyledSpan expression in on_start in 31s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:219:17: delete field style from struct StyledSpan expression in on_start in 63s build + 60s test
TIMEOUT  src/decoration/builder/fenced.rs:220:17: delete field full_line_bg from struct StyledSpan expression in on_start in 63s build + 60s test
TIMEOUT  src/decoration/builder/headings.rs:77:5: replace on_end with () in 32s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:49:50: replace && with || in on_strong_end in 31s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:106:36: replace + with * in on_strong_end in 31s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:162:36: replace + with * in on_emphasis_end in 28s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:177:40: replace + with * in on_emphasis_end in 34s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:224:21: replace < with <= in on_code in 33s build + 60s test
TIMEOUT  src/decoration/builder/inline.rs:232:24: replace < with <= in on_code in 33s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:45:26: replace > with >= in on_link in 60s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:49:42: replace + with - in on_link in 93s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:49:58: replace + with - in on_link in 61s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:49:58: replace + with * in on_link in 130s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:57:32: replace + with - in on_link in 56s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:57:32: replace + with * in on_link in 73s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:58:44: replace + with - in on_link in 80s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:58:44: replace + with * in on_link in 44s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:58:32: replace + with - in on_link in 53s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:63:52: replace + with - in on_link in 39s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:65:24: replace > with == in on_link in 42s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:65:24: replace > with >= in on_link in 52s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:73:30: replace > with >= in on_link in 39s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:92:5: replace on_image with () in 34s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:117:50: replace + with - in on_image in 36s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:120:26: replace > with >= in on_image in 35s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:140:25: replace > with >= in on_image in 35s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:148:30: replace > with >= in on_image in 34s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:184:31: replace < with <= in on_strikethrough in 34s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:184:27: replace + with * in on_strikethrough in 33s build + 60s test
TIMEOUT  src/decoration/builder/misc.rs:212:13: delete field char_start from struct StyledSpan expression in on_rule in 35s build + 60s test
TIMEOUT  src/decoration/builder/tables.rs:46:54: replace + with * in on_table_start in 38s build + 60s test
TIMEOUT  src/decoration/builder/tables.rs:53:54: replace + with - in on_table_start in 59s build + 60s test
1528 mutants tested in 17h: 151 missed, 1130 caught, 50 unviable, 197 timeouts
