# Cargo mutants pass 2 

❯ cargo mutants
Found 432 mutants to test
ok       Unmutated baseline in 16s build + 1s test
 INFO Auto-set test timeout to 20s
MISSED   src/app.rs:126:17: replace += with *= in expand_tabs in 0s build + 1s test
MISSED   src/decoration/mod.rs:37:44: replace && with || in add_modifier_to_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:37:30: replace > with >= in add_modifier_to_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:37:63: replace < with <= in add_modifier_to_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:54: replace && with || in emit_content_around_existing in 1s build + 2s test
MISSED   src/decoration/mod.rs:69:40: replace > with == in emit_content_around_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:69:40: replace > with < in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:40: replace > with >= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with > in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:69:70: replace < with <= in emit_content_around_existing in 2s build + 1s test
MISSED   src/decoration/mod.rs:79:16: replace < with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:79:16: replace < with <= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with == in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with < in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:82:22: replace > with >= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:86:12: replace < with <= in emit_content_around_existing in 1s build + 1s test
MISSED   src/decoration/mod.rs:127:36: replace || with && in emit_bold_italic_spans in 2s build + 1s test
MISSED   src/decoration/mod.rs:127:22: replace > with == in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:127:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:147:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:155:20: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/decoration/mod.rs:163:22: replace > with >= in emit_bold_italic_spans in 1s build + 1s test
MISSED   src/renderer/mod.rs:29:21: replace || with && in wrap_line in 1s build + 1s test
MISSED   src/renderer/mod.rs:75:41: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:75:36: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:87:33: replace + with - in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:87:33: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:90:24: replace += with -= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:90:24: replace += with *= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:96:33: replace + with - in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:96:33: replace + with * in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:103:23: replace < with <= in wrap_line in 1s build + 2s test
MISSED   src/renderer/mod.rs:132:40: replace + with - in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:132:40: replace + with * in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:135:40: replace + with * in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:136:36: replace + with * in wrap_char_ranges in 1s build + 2s test
MISSED   src/renderer/mod.rs:346:71: replace * with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:346:71: replace * with / in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:32: replace && with || in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:22: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:355:43: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with == in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with < in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:375:56: replace > with >= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:404:34: replace < with <= in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:41: replace + with - in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:407:80: replace - with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:408:65: replace - with + in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:409:46: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/renderer/mod.rs:409:37: replace + with * in apply_selection_overlay in 1s build + 2s test
MISSED   src/commands.rs:66:27: replace * with + in clamp_scroll in 0s build + 2s test
MISSED   src/commands.rs:66:27: replace * with / in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:70:19: replace < with <= in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:88:39: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:38: replace || with && in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with == in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with > in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:27: replace < with <= in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:47: replace == with != in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:89:43: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:99:22: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:100:54: replace + with - in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:100:54: replace + with * in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:103:23: replace > with < in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:105:55: replace - with + in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:105:55: replace - with / in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:106:27: replace > with == in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:106:27: replace > with < in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:106:27: replace > with >= in clamp_scroll in 0s build + 1s test
MISSED   src/commands.rs:110:21: replace -= with += in clamp_scroll in 1s build + 1s test
MISSED   src/commands.rs:110:21: replace -= with /= in clamp_scroll in 0s build + 1s test
MISSED   src/input.rs:142:43: replace | with ^ in handle_pair_wrap in 0s build + 1s test
MISSED   src/input.rs:215:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('c')) |(KeyModifiers::SUPER, KeyCode::Char('c')) in handle_key_event in 0s build + 1s test
MISSED   src/input.rs:221:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('v')) |(KeyModifiers::SUPER, KeyCode::Char('v')) in handle_key_event in 1s build + 1s test
MISSED   src/input.rs:269:24: replace && with || in handle_key_event in 1s build + 1s test
MISSED   src/input.rs:269:16: delete ! in handle_key_event in 0s build + 1s test
MISSED   src/input.rs:275:47: replace != with == in handle_key_event in 1s build + 1s test
MISSED   src/input.rs:298:5: replace event_loop -> io::Result<()> with Ok(()) in 1s build + 1s test
MISSED   src/input.rs:317:33: replace || with && in event_loop in 1s build + 1s test
MISSED   src/input.rs:317:83: replace >= with < in event_loop in 1s build + 1s test
MISSED   src/input.rs:332:70: replace && with || in event_loop in 0s build + 1s test
MISSED   src/input.rs:332:38: delete ! in event_loop in 1s build + 1s test
MISSED   src/input.rs:332:98: replace > with == in event_loop in 0s build + 1s test
MISSED   src/input.rs:332:98: replace > with < in event_loop in 1s build + 1s test
MISSED   src/input.rs:332:98: replace > with >= in event_loop in 0s build + 1s test
MISSED   src/input.rs:335:21: delete field y from struct Rect expression in event_loop in 1s build + 1s test
MISSED   src/input.rs:336:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:335:44: replace + with - in event_loop in 1s build + 1s test
MISSED   src/input.rs:335:44: replace + with * in event_loop in 1s build + 1s test
MISSED   src/input.rs:345:16: delete ! in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:66: replace && with || in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:34: delete ! in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:90: replace > with == in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:90: replace > with < in event_loop in 0s build + 1s test
MISSED   src/input.rs:370:90: replace > with >= in event_loop in 0s build + 1s test
MISSED   src/input.rs:372:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:382:21: delete field y from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:383:21: delete field height from struct Rect expression in event_loop in 0s build + 1s test
MISSED   src/input.rs:382:40: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:382:40: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:409:17: delete match arm Event::Key(k) in event_loop in 0s build + 1s test
MISSED   src/input.rs:436:17: delete match arm Event::Mouse(mouse) in event_loop in 1s build + 1s test
MISSED   src/input.rs:480:17: delete match arm Event::Resize(_, _) in event_loop in 0s build + 1s test
MISSED   src/input.rs:437:21: delete match arm MouseEventKind::ScrollDown in event_loop in 0s build + 1s test
MISSED   src/input.rs:442:21: delete match arm MouseEventKind::ScrollUp in event_loop in 0s build + 1s test
MISSED   src/input.rs:446:21: delete match arm MouseEventKind::Down(MouseButton::Left) in event_loop in 0s build + 1s test
MISSED   src/input.rs:461:21: delete match arm MouseEventKind::Drag(MouseButton::Left) in event_loop in 0s build + 1s test
MISSED   src/input.rs:439:58: replace + with - in event_loop in 0s build + 1s test
MISSED   src/input.rs:439:58: replace + with * in event_loop in 0s build + 1s test
MISSED   src/input.rs:471:32: delete ! in event_loop in 0s build + 1s test
MISSED   src/decoration/spans.rs:32:30: replace < with > in line_char_len in 2s build + 3s test
MISSED   src/decoration/spans.rs:77:31: replace == with != in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:78:29: replace == with != in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:79:32: replace + with - in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:79:32: replace + with * in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:83:39: replace + with * in add_byte_range_span in 2s build + 3s test
MISSED   src/decoration/spans.rs:88:17: delete field char_start from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:89:17: delete field char_end from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:91:17: delete field is_blockquote from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/spans.rs:92:17: delete field full_line_bg from struct StyledSpan expression in add_byte_range_span in 1s build + 3s test
MISSED   src/decoration/words.rs:18:13: replace < with <= in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:20:13: delete match arm '[' in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:20: replace match guard i + 1 < chars.len() && chars[i + 1] == '(' with true in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:20:34: replace += with *= in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:40: replace && with || in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:26: replace < with <= in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:22: replace + with - in link_split_char_idx in 1s build + 3s test
MISSED   src/decoration/words.rs:21:22: replace + with * in link_split_char_idx in 1s build + 3s test
TIMEOUT  src/decoration/words.rs:29:11: replace += with *= in link_split_char_idx in 2s build + 20s test
432 mutants tested in 19m: 129 missed, 296 caught, 6 unviable, 1 timeouts