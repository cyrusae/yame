Found 1299 mutants to test
ok       Unmutated baseline in 21s build + 3s test
MISSED   src/search.rs:186:19: replace < with <= in SearchState::apply_replace_all in 2s build + 11s test
MISSED   src/settings.rs:395:26: replace < with <= in SettingsModal::field_value in 2s build + 10s test
MISSED   src/settings.rs:419:31: replace < with <= in SettingsModal::commit_input in 3s build + 11s test
MISSED   src/settings.rs:635:56: replace || with && in SettingsModal::hit_test in 3s build + 17s test
MISSED   src/settings.rs:635:44: replace || with && in SettingsModal::hit_test in 4s build + 18s test
MISSED   src/settings.rs:635:16: replace < with <= in SettingsModal::hit_test in 3s build + 17s test
MISSED   src/settings.rs:635:34: replace + with * in SettingsModal::hit_test in 3s build + 17s test
MISSED   src/settings.rs:635:69: replace + with * in SettingsModal::hit_test in 4s build + 19s test
MISSED   src/settings.rs:641:20: replace < with == in SettingsModal::hit_test in 3s build + 18s test
MISSED   src/settings.rs:641:20: replace < with <= in SettingsModal::hit_test in 3s build + 17s test
MISSED   src/settings.rs:641:25: replace + with - in SettingsModal::hit_test in 4s build + 16s test
MISSED   src/settings.rs:652:26: replace < with <= in SettingsModal::hit_test in 4s build + 19s test
MISSED   src/settings.rs:652:22: replace + with * in SettingsModal::hit_test in 4s build + 18s test
MISSED   src/settings.rs:653:28: replace < with <= in SettingsModal::hit_test in 4s build + 21s test
MISSED   src/settings.rs:664:41: replace < with <= in SettingsModal::hit_test in 4s build + 20s test
MISSED   src/settings.rs:664:58: replace + with * in SettingsModal::hit_test in 5s build + 22s test
MISSED   src/settings.rs:667:26: replace < with <= in SettingsModal::hit_test in 8s build + 18s test
MISSED   src/settings.rs:687:23: replace < with <= in SettingsModal::jump_to_field in 6s build + 21s test
MISSED   src/settings.rs:700:31: replace && with || in SettingsModal::activate_clicked_field in 11s build + 25s test
MISSED   src/settings.rs:700:16: delete ! in SettingsModal::activate_clicked_field in 6s build + 27s test
MISSED   src/settings.rs:700:45: replace > with == in SettingsModal::activate_clicked_field in 8s build + 27s test
MISSED   src/settings.rs:700:45: replace > with < in SettingsModal::activate_clicked_field in 9s build + 38s test
MISSED   src/settings.rs:700:45: replace > with >= in SettingsModal::activate_clicked_field in 8s build + 22s test
MISSED   src/settings.rs:775:63: replace | with & in handle_settings_key in 7s build + 31s test
MISSED   src/settings.rs:775:63: replace | with ^ in handle_settings_key in 8s build + 42s test
MISSED   src/settings.rs:775:43: replace | with ^ in handle_settings_key in 9s build + 33s test
MISSED   src/settings.rs:841:36: replace && with || in handle_settings_key in 6s build + 27s test
MISSED   src/settings.rs:841:20: delete ! in handle_settings_key in 10s build + 29s test
MISSED   src/settings.rs:841:51: replace > with == in handle_settings_key in 8s build + 25s test
MISSED   src/settings.rs:841:51: replace > with < in handle_settings_key in 7s build + 45s test
MISSED   src/settings.rs:841:51: replace > with >= in handle_settings_key in 12s build + 43s test
MISSED   src/settings.rs:1101:9: delete match arm 0..= 5 in commit_editor in 14s build + 66s test
MISSED   src/table_format.rs:252:44: replace && with || in compact_table in 16s build + 82s test
MISSED   src/table_format.rs:252:30: replace > with >= in compact_table in 17s build + 83s test
MISSED   src/commands.rs:93:50: replace > with == in clamp_scroll in 6s build + 59s test
MISSED   src/commands.rs:93:50: replace > with >= in clamp_scroll in 6s build + 39s test
MISSED   src/input.rs:294:43: replace | with & in handle_auto_close in 7s build + 39s test
MISSED   src/input.rs:294:43: replace | with ^ in handle_auto_close in 5s build + 39s test
MISSED   src/input.rs:319:9: delete match arm '[' in handle_auto_close in 6s build + 38s test
MISSED   src/input.rs:324:9: delete match arm '{' in handle_auto_close in 5s build + 39s test
MISSED   src/input.rs:338:9: delete match arm ']' in handle_auto_close in 6s build + 39s test
MISSED   src/input.rs:346:9: delete match arm '}' in handle_auto_close in 5s build + 38s test
MISSED   src/input.rs:355:9: delete match arm '"' | '\'' | '`' | '*' | '_' in handle_auto_close in 5s build + 38s test
MISSED   src/input.rs:339:26: replace == with != in handle_auto_close in 6s build + 39s test
MISSED   src/input.rs:347:26: replace == with != in handle_auto_close in 5s build + 38s test
MISSED   src/input.rs:829:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('a')) |(KeyModifiers::SUPER, KeyCode::Char('a')) in handle_key_event in 6s build + 39s test
MISSED   src/input.rs:876:67: replace | with ^ in handle_key_event in 6s build + 40s test
1299 mutants tested in 7h: 47 missed, 1019 caught, 233 unviable
