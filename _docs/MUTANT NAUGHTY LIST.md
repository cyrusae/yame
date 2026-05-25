# Mutant naughty list

`cargo mutants` findings May 25 2026 first run.

❯ cargo mutants
Found 434 mutants to test
ok       Unmutated baseline in 20s build + 1s test
 INFO Auto-set test timeout to 20s
MISSED   src/app.rs:126:17: replace += with *= in expand_tabs in 0s build + 1s test
MISSED   src/config.rs:173:5: replace blend_colors -> Color with Default::default() in 2s build + 1s test
MISSED   src/decoration/mod.rs:37:44: replace && with || in add_modifier_to_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:37:30: replace > with >= in add_modifier_to_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:37:63: replace < with <= in add_modifier_to_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:54: replace && with || in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:40: replace > with == in emit_content_around_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:69:40: replace > with < in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:40: replace > with >= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with > in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with <= in emit_content_around_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:79:16: replace < with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:79:16: replace < with <= in emit_content_around_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with < in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with >= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:86:12: replace < with <= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:127:36: replace || with && in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:127:22: replace > with == in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:127:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:147:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:155:20: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:163:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/renderer/mod.rs:29:21: replace || with && in wrap_line in 1s build + 1s test
MISSED   src/renderer/mod.rs:75:41: replace + with * in wrap_line in 1s build + 1s test
MISSED   src/renderer/mod.rs:75:36: replace + with * in wrap_line in 1s build + 1s test
MISSED   src/renderer/mod.rs:87:33: replace + with - in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:87:33: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:90:24: replace += with -= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:90:24: replace += with *= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:96:33: replace + with - in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:96:33: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:103:23: replace < with <= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:124:5: replace wrap_char_ranges -> Vec<(usize, usize)> with vec![(0, 1)] in 1s build + 2s test
MISSED   src/renderer/mod.rs:124:5: replace wrap_char_ranges -> Vec<(usize, usize)> with vec![(1, 1)] in 1s build + 2s test
MISSED   src/renderer/mod.rs:130:56: replace - with / in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:132:40: replace + with - in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:132:40: replace + with * in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:135:40: replace + with * in wrap_char_ranges in 3s build + 2s test
MISSED   src/renderer/mod.rs:136:36: replace + with * in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:345:5: replace apply_selection_overlay with () in 1s build + 2s test
MISSED   src/renderer/mod.rs:346:71: replace * with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:346:71: replace * with / in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:32: replace && with || in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:22: replace < with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:22: replace < with > in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:22: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:43: replace < with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:43: replace < with > in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:43: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:356:20: replace > with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:356:20: replace > with < in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:356:20: replace > with >= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:367:27: replace >= with < in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:371:39: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:371:39: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with < in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with >= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:389:41: replace && with || in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:389:24: replace >= with < in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:389:52: replace <= with > in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:390:49: replace == with != in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:395:47: replace == with != in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:404:34: replace < with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:404:34: replace < with > in apply_selection_overlay in 1s build + 3s test
MISSED   src/renderer/mod.rs:404:34: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:405:36: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:405:36: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:63: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:63: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:41: replace + with - in apply_selection_overlay in 1s build + 3s test
MISSED   src/renderer/mod.rs:407:41: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:32: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:32: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:80: replace - with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:80: replace - with / in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:50: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:50: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:41: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:41: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:65: replace - with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:65: replace - with / in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:409:46: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:409:46: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:409:37: replace + with - in apply_selection_overlay in 2s build + 2s test
MISSED   src/renderer/mod.rs:409:37: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:416:24: replace += with -= in apply_selection_overlay in 2s build + 2s test
MISSED   src/renderer/mod.rs:416:24: replace += with *= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:419:17: replace += with -= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:419:17: replace += with *= in apply_selection_overlay in 1s build + 2s test
MISSED   src/status.rs:31:9: replace StatusLine::set_dismissible with () in 1s build + 2s test
MISSED   src/status.rs:52:13: delete match arm StatusMode::DismissibleMessage(text) in StatusLine::message in 1s build + 2s test
MISSED   src/commands.rs:66:27: replace * with + in clamp_scroll in 0s build + 2s test
MISSED   src/commands.rs:66:27: replace * with / in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:70:19: replace < with <= in clamp_scroll in 1s build + 3s test
MISSED   src/commands.rs:88:39: replace + with * in clamp_scroll in 1s build + 1s test
MISSED   src/commands.rs:89:38: replace || with && in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with == in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with > in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with <= in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:47: replace == with != in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:43: replace + with - in clamp_scroll in 2s build + 2s test
MISSED   src/commands.rs:89:43: replace + with * in clamp_scroll in 3s build + 3s test
MISSED   src/commands.rs:97:38: replace + with - in clamp_scroll in 6s build + 2s test
MISSED   src/commands.rs:99:22: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:100:54: replace + with - in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:100:54: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:103:23: replace > with < in clamp_scroll in 1s build + 1s test
MISSED   src/commands.rs:105:55: replace - with + in clamp_scroll in 0s build + 2s test
MISSED   src/commands.rs:105:55: replace - with / in clamp_scroll in 3s build + 2s test
MISSED   src/commands.rs:106:27: replace > with == in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:106:27: replace > with < in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:106:27: replace > with >= in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:110:21: replace -= with += in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:110:21: replace -= with /= in clamp_scroll in 0s build + 1s test
MISSED   src/input.rs:64:5: replace is_navigation_key -> bool with true in 0s build + 1s test
MISSED   src/input.rs:67:46: replace && with || in is_navigation_key in 0s build + 1s test
MISSED   src/input.rs:67:21: replace == with != in is_navigation_key in 0s build + 1s test
MISSED   src/input.rs:97:32: replace == with != in get_selection_text in 1s build + 1s test
MISSED   src/input.rs:98:30: replace == with != in get_selection_text in 0s build + 1s test
MISSED   src/input.rs:104:20: replace < with == in get_selection_text in 0s build + 1s test
MISSED   src/input.rs:104:20: replace < with <= in get_selection_text in 1s build + 1s test
MISSED   src/input.rs:116:43: replace | with ^ in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:121:9: delete match arm KeyCode::Char('(') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:123:9: delete match arm KeyCode::Char('{') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:124:9: delete match arm KeyCode::Char('"') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:125:9: delete match arm KeyCode::Char('\'') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:126:9: delete match arm KeyCode::Char('`') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:128:9: delete match arm KeyCode::Char('_') in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:145:5: replace event_loop -> io::Result<()> with Ok(()) in 0s build + 1s test
MISSED   src/input.rs:164:33: replace || with && in event_loop in 0s build + 1s test
MISSED   src/input.rs:164:83: replace >= with < in event_loop in 0s build + 1s test
MISSED   src/input.rs:179:70: replace && with || in event_loop in 0s build + 1s test
MISSED   src/input.rs:179:38: delete ! in event_loop in 0s build + 1s test
MISSED   src/input.rs:179:98: replace > with == in event_loop in 0s build + 1s test
MISSED   src/input.rs:179:98: replace > with < in event_loop in 0s build + 1s test
MISSED   src/input.rs:179:98: replace > with >= in event_loop in 0s build + 1s test
MISSED   src/input.rs:182:21: delete field y from struct Rect expression in event_loop in 1s build + 1s test
MISSED   src/input.rs:183:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:182:44: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:182:44: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:192:16: delete ! in event_loop in 0s build + 1s test
MISSED   src/input.rs:217:66: replace && with || in event_loop in 0s build + 1s test
MISSED   src/input.rs:217:34: delete ! in event_loop in 1s build + 1s test
MISSED   src/input.rs:217:90: replace > with == in event_loop in 2s build + 1s test
MISSED   src/input.rs:217:90: replace > with < in event_loop in 0s build + 1s test
MISSED   src/input.rs:217:90: replace > with >= in event_loop in 0s build + 1s test
MISSED   src/input.rs:219:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:229:21: delete field y from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:230:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:229:40: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:229:40: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:256:17: delete match arm Event::Key(k) in event_loop in 0s build + 1s test
MISSED   src/input.rs:364:17: delete match arm Event::Mouse(mouse) in event_loop in 1s build + 1s test
MISSED   src/input.rs:408:17: delete match arm Event::Resize(_, _) in event_loop in 1s build + 1s test
MISSED   src/input.rs:263:29: delete match arm KeyCode::Char('y') | KeyCode::Char('Y') in event_loop in 0s build + 1s test
MISSED   src/input.rs:267:29: delete match arm KeyCode::Char('n') | KeyCode::Char('N') in event_loop in 0s build + 1s test
MISSED   src/input.rs:270:29: delete match arm KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('x') | KeyCode::Char('X') in event_loop in 0s build + 1s test
MISSED   src/input.rs:281:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('s')) |(KeyModifiers::SUPER, KeyCode::Char('s')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:285:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('x')) |(KeyModifiers::NONE, KeyCode::Esc) in event_loop in 0s build + 1s test
MISSED   src/input.rs:291:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('c')) |(KeyModifiers::SUPER, KeyCode::Char('c')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:295:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('v')) |(KeyModifiers::SUPER, KeyCode::Char('v')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:300:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('z')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:308:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('y')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:316:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('r')) in event_loop in 0s build + 1s test
MISSED   src/input.rs:331:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Up) in event_loop in 0s build + 1s test
MISSED   src/input.rs:335:29: delete match arm (KeyModifiers::CONTROL, KeyCode::Down) in event_loop in 0s build + 1s test
MISSED   src/input.rs:337:66: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:337:66: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:345:44: replace && with || in event_loop in 0s build + 1s test
MISSED   src/input.rs:345:36: delete ! in event_loop in 0s build + 1s test
MISSED   src/input.rs:351:67: replace != with == in event_loop in 0s build + 1s test
MISSED   src/input.rs:365:21: delete match arm MouseEventKind::ScrollDown in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:21: delete match arm MouseEventKind::ScrollUp in event_loop in 0s build + 1s test
MISSED   src/input.rs:374:21: delete match arm MouseEventKind::Down(MouseButton::Left) in event_loop in 0s build + 1s test
MISSED   src/input.rs:389:21: delete match arm MouseEventKind::Drag(MouseButton::Left) in event_loop in 0s build + 1s test
MISSED   src/input.rs:367:58: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:367:58: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:399:32: delete ! in event_loop in 0s build + 1s test
MISSED   src/decoration/spans.rs:32:30: replace < with > in line_char_len in 1s build + 3s test
MISSED   src/decoration/spans.rs:77:31: replace == with != in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:78:29: replace == with != in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:79:32: replace + with - in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:79:32: replace + with * in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:83:39: replace + with * in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:88:17: delete field char_start from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:89:17: delete field char_end from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:91:17: delete field is_blockquote from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:92:17: delete field full_line_bg from struct StyledSpan expression in add_byte_range_span in 1s build + 4s test
MISSED   src/decoration/words.rs:18:13: replace < with <= in link_split_char_idx in 2s build + 3s test
MISSED   src/decoration/words.rs:20:13: delete match arm '[' in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:20: replace match guard i + 1 < chars.len() && chars[i + 1] == '(' with true in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:20:34: replace += with *= in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:40: replace && with || in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:26: replace < with <= in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:22: replace + with - in link_split_char_idx in 1s
