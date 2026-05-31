# Missed and timeouts 

*Cargo mutants run 2026/05/30*

Naughty updates get put in the mutant wiggler.

---

MISSED   src/main.rs:20:5: replace setup_panic_hook with () in 0s build + 4s test
MISSED   src/main.rs:30:5: replace run -> io::Result<()> with Ok(()) in 0s build + 3s test
MISSED   src/main.rs:80:8: delete ! in run in 0s build + 2s test
MISSED   src/main.rs:107:5: replace main with () in 0s build + 3s test
MISSED   src/main.rs:117:37: replace && with || in main in 2s build + 3s test
MISSED   src/main.rs:117:27: replace != with == in main in 0s build + 3s test
MISSED   src/main.rs:117:51: replace != with == in main in 0s build + 3s test
MISSED   src/app.rs:95:5: replace is_likely_binary -> bool with true in 1s build + 4s test
MISSED   src/app.rs:95:5: replace is_likely_binary -> bool with false in 2s build + 4s test
MISSED   src/app.rs:96:8: delete ! in is_likely_binary in 1s build + 4s test
MISSED   src/app.rs:351:5: replace load_file -> io::Result<TextArea<'static>> with Ok(Default::default()) in 1s build + 4s test
MISSED   src/app.rs:351:27: replace == with != in load_file in 1s build + 4s test
MISSED   src/clipboard.rs:7:5: replace handle_copy with () in 1s build + 4s test
MISSED   src/clipboard.rs:10:9: delete match arm ClipboardState::Ready(cb) in handle_copy in 1s build + 4s test
MISSED   src/clipboard.rs:22:5: replace handle_paste with () in 1s build + 4s test
MISSED   src/clipboard.rs:47:5: replace ensure_clipboard with () in 2s build + 7s test
MISSED   src/config.rs:545:5: replace load_config -> (Config, Vec<String>) with (Default::default(), vec![]) in 2s build + 4s test
MISSED   src/config.rs:545:5: replace load_config -> (Config, Vec<String>) with (Default::default(), vec![String::new()]) in 1s build + 4s test
MISSED   src/config.rs:545:5: replace load_config -> (Config, Vec<String>) with (Default::default(), vec!["xyzzy".into()]) in 1s build + 4s test
MISSED   src/config.rs:548:8: delete ! in load_config in 2s build + 4s test
MISSED   src/config.rs:626:5: replace supports_italic -> bool with true in 1s build + 4s test
MISSED   src/config.rs:626:5: replace supports_italic -> bool with false in 1s build + 5s test
MISSED   src/decoration/mod.rs:268:60: replace | with ^ in build_decoration_map in 1s build + 5s test
MISSED   src/decoration/mod.rs:268:32: replace | with ^ in build_decoration_map in 1s build + 5s test
MISSED   src/decoration/mod.rs:870:13: delete match arm Event::End(TagEnd::List(_)) in build_decoration_map in 2s build + 5s test
MISSED   src/decoration/mod.rs:327:30: replace > with >= in build_decoration_map in 1s build + 5s test
MISSED   src/decoration/mod.rs:332:29: delete field char_start from struct StyledSpan expression in build_decoration_map in 1s build + 5s test
MISSED   src/decoration/mod.rs:350:29: delete field style from struct StyledSpan expression in build_decoration_map in 2s build + 5s test
MISSED   src/decoration/mod.rs:351:29: delete field full_line_bg from struct StyledSpan expression in build_decoration_map in 2s build + 6s test
MISSED   src/decoration/mod.rs:422:70: replace + with * in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:427:48: replace + with * in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:434:61: replace - with + in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:434:61: replace - with / in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:441:48: replace + with * in build_decoration_map in 2s build + 8s test
MISSED   src/decoration/mod.rs:455:62: replace && with || in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:455:42: replace == with != in build_decoration_map in 2s build + 8s test
MISSED   src/decoration/mod.rs:455:38: replace + with * in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:455:84: replace == with != in build_decoration_map in 2s build + 9s test
MISSED   src/decoration/mod.rs:455:80: replace + with - in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:455:80: replace + with * in build_decoration_map in 6s build + 9s test
MISSED   src/decoration/mod.rs:491:70: replace + with * in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:496:48: replace + with * in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:503:61: replace - with + in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:503:61: replace - with / in build_decoration_map in 4s build + 10s test
MISSED   src/decoration/mod.rs:511:52: replace + with * in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:524:28: replace += with *= in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:541:44: replace == with != in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:554:33: replace < with <= in build_decoration_map in 2s build + 10s test
MISSED   src/decoration/mod.rs:562:36: replace < with <= in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:616:52: replace < with == in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:616:52: replace < with > in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:616:52: replace < with <= in build_decoration_map in 3s build + 9s test
MISSED   src/decoration/mod.rs:616:48: replace + with * in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:623:51: replace || with && in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:623:44: replace == with != in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:630:29: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:631:29: delete field char_end from struct StyledSpan expression in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:639:53: replace + with * in build_decoration_map in 5s build + 12s test
MISSED   src/decoration/mod.rs:640:37: replace > with >= in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:658:43: replace < with == in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:658:43: replace < with > in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:658:43: replace < with <= in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:658:39: replace + with - in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:658:39: replace + with * in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:674:43: replace match guard !hl_spans.is_empty() with true in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:685:39: replace < with <= in build_decoration_map in 4s build + 11s test
MISSED   src/decoration/mod.rs:690:45: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:691:45: delete field char_end from struct StyledSpan expression in build_decoration_map in 4s build + 11s test
MISSED   src/decoration/mod.rs:714:37: delete field char_start from struct StyledSpan expression in build_decoration_map in 4s build + 10s test
MISSED   src/decoration/mod.rs:715:37: delete field char_end from struct StyledSpan expression in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:716:37: delete field style from struct StyledSpan expression in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:726:29: replace > with >= in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:729:50: replace < with == in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:729:50: replace < with > in build_decoration_map in 2s build + 10s test
MISSED   src/decoration/mod.rs:729:50: replace < with <= in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:729:46: replace + with - in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:729:46: replace + with * in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:736:51: replace || with && in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:736:44: replace == with != in build_decoration_map in 3s build + 10s test
MISSED   src/decoration/mod.rs:736:56: replace == with != in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:743:29: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:744:29: delete field char_end from struct StyledSpan expression in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:783:29: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:785:29: delete field style from struct StyledSpan expression in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:827:38: replace > with >= in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:845:64: replace + with - in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:845:64: replace + with * in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:845:52: replace + with * in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:847:36: replace > with == in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:847:36: replace > with < in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:847:36: replace > with >= in build_decoration_map in 4s build + 11s test
MISSED   src/decoration/mod.rs:855:42: replace > with == in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:855:42: replace > with < in build_decoration_map in 4s build + 12s test
MISSED   src/decoration/mod.rs:855:42: replace > with >= in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:859:57: replace - with + in build_decoration_map in 3s build + 11s test
MISSED   src/decoration/mod.rs:859:57: replace - with / in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:886:46: replace + with * in build_decoration_map in 3s build + 15s test
MISSED   src/decoration/mod.rs:897:25: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:917:53: replace > with >= in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:926:52: replace + with * in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:933:61: replace + with - in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:933:61: replace + with * in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:936:40: replace < with <= in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:936:36: replace + with - in build_decoration_map in 4s build + 12s test
MISSED   src/decoration/mod.rs:936:36: replace + with * in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:940:51: replace + with - in build_decoration_map in 4s build + 14s test
MISSED   src/decoration/mod.rs:940:51: replace + with * in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:940:69: replace + with - in build_decoration_map in 5s build + 12s test
MISSED   src/decoration/mod.rs:940:69: replace + with * in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:944:40: replace < with == in build_decoration_map in 3s build + 14s test
MISSED   src/decoration/mod.rs:944:40: replace < with > in build_decoration_map in 4s build + 12s test
MISSED   src/decoration/mod.rs:944:40: replace < with <= in build_decoration_map in 6s build + 13s test
MISSED   src/decoration/mod.rs:944:36: replace + with - in build_decoration_map in 4s build + 12s test
MISSED   src/decoration/mod.rs:944:36: replace + with * in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:948:51: replace + with - in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:948:51: replace + with * in build_decoration_map in 4s build + 14s test
MISSED   src/decoration/mod.rs:952:36: replace < with == in build_decoration_map in 5s build + 12s test
MISSED   src/decoration/mod.rs:952:36: replace < with > in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:952:36: replace < with <= in build_decoration_map in 4s build + 12s test
MISSED   src/decoration/mod.rs:966:60: replace + with - in build_decoration_map in 3s build + 12s test
MISSED   src/decoration/mod.rs:966:60: replace + with * in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:971:47: replace + with - in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:971:47: replace + with * in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:971:64: replace + with * in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:1001:25: replace && with || in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:1007:29: delete match arm '|' in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:1014:36: replace match guard is_sep with true in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:1021:36: replace match guard is_sep with true in build_decoration_map in 3s build + 15s test
MISSED   src/decoration/mod.rs:1011:66: replace + with * in build_decoration_map in 3s build + 13s test
MISSED   src/decoration/mod.rs:1082:43: replace < with <= in build_decoration_map in 4s build + 14s test
MISSED   src/decoration/mod.rs:1082:39: replace + with * in build_decoration_map in 4s build + 13s test
MISSED   src/decoration/mod.rs:1107:25: delete field char_start from struct StyledSpan expression in build_decoration_map in 3s build + 14s test
MISSED   src/renderer/mod.rs:79:41: replace + with * in wrap_line in 4s build + 15s test
MISSED   src/renderer/mod.rs:79:36: replace + with * in wrap_line in 4s build + 15s test
TIMEOUT  src/renderer/mod.rs:81:46: replace + with * in wrap_line in 4s build + 20s test
TIMEOUT  src/renderer/mod.rs:85:22: replace == with != in wrap_line in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:94:24: replace += with *= in wrap_line in 4s build + 20s test
TIMEOUT  src/renderer/mod.rs:100:33: replace + with - in wrap_line in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:100:33: replace + with * in wrap_line in 6s build + 20s test
MISSED   src/renderer/mod.rs:344:78: replace + with - in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:344:78: replace + with * in <impl Widget for MarkdownView<'_>>::render in 6s build + 17s test
MISSED   src/renderer/mod.rs:352:29: replace + with * in <impl Widget for MarkdownView<'_>>::render in 4s build + 17s test
MISSED   src/renderer/mod.rs:352:43: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 17s test
MISSED   src/renderer/mod.rs:363:26: replace < with <= in <impl Widget for MarkdownView<'_>>::render in 4s build + 17s test
MISSED   src/renderer/mod.rs:397:65: replace && with || in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:397:52: replace > with >= in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:397:81: replace < with <= in <impl Widget for MarkdownView<'_>>::render in 4s build + 17s test
MISSED   src/renderer/mod.rs:420:60: replace > with == in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
MISSED   src/renderer/mod.rs:420:60: replace > with < in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:420:60: replace > with >= in <impl Widget for MarkdownView<'_>>::render in 5s build + 17s test
TIMEOUT  src/renderer/mod.rs:435:53: replace == with != in <impl Widget for MarkdownView<'_>>::render in 5s build + 20s test
MISSED   src/renderer/mod.rs:435:49: replace + with * in <impl Widget for MarkdownView<'_>>::render in 4s build + 18s test
MISSED   src/renderer/mod.rs:437:54: replace && with || in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
MISSED   src/renderer/mod.rs:437:40: replace >= with < in <impl Widget for MarkdownView<'_>>::render in 6s build + 19s test
MISSED   src/renderer/mod.rs:437:84: replace || with && in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
MISSED   src/renderer/mod.rs:437:73: replace < with == in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:437:73: replace < with > in <impl Widget for MarkdownView<'_>>::render in 5s build + 19s test
MISSED   src/renderer/mod.rs:437:73: replace < with <= in <impl Widget for MarkdownView<'_>>::render in 5s build + 19s test
MISSED   src/renderer/mod.rs:443:72: replace + with - in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
MISSED   src/renderer/mod.rs:443:72: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 19s test
MISSED   src/renderer/mod.rs:443:50: replace + with - in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:443:50: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:443:36: replace + with * in <impl Widget for MarkdownView<'_>>::render in 6s build + 19s test
MISSED   src/renderer/mod.rs:444:36: replace + with - in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
MISSED   src/renderer/mod.rs:444:36: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
MISSED   src/renderer/mod.rs:458:49: replace == with != in <impl Widget for MarkdownView<'_>>::render in 5s build + 18s test
TIMEOUT  src/renderer/mod.rs:458:45: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 20s test
TIMEOUT  src/renderer/mod.rs:467:37: replace + with - in <impl Widget for MarkdownView<'_>>::render in 5s build + 20s test
TIMEOUT  src/renderer/mod.rs:467:37: replace + with * in <impl Widget for MarkdownView<'_>>::render in 13s build + 20s test
TIMEOUT  src/renderer/mod.rs:470:37: replace + with - in <impl Widget for MarkdownView<'_>>::render in 9s build + 20s test
MISSED   src/renderer/mod.rs:470:37: replace + with * in <impl Widget for MarkdownView<'_>>::render in 7s build + 18s test
TIMEOUT  src/renderer/mod.rs:480:43: replace && with || in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:480:55: replace == with != in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:481:48: replace == with != in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
MISSED   src/renderer/mod.rs:491:37: replace + with - in <impl Widget for MarkdownView<'_>>::render in 7s build + 19s test
TIMEOUT  src/renderer/mod.rs:491:37: replace + with * in <impl Widget for MarkdownView<'_>>::render in 8s build + 20s test
MISSED   src/renderer/mod.rs:503:37: replace + with - in <impl Widget for MarkdownView<'_>>::render in 8s build + 19s test
TIMEOUT  src/renderer/mod.rs:503:37: replace + with * in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:503:73: replace + with - in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:503:73: replace + with * in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:503:59: replace + with - in <impl Widget for MarkdownView<'_>>::render in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:503:59: replace + with * in <impl Widget for MarkdownView<'_>>::render in 9s build + 20s test
TIMEOUT  src/renderer/mod.rs:524:54: replace + with - in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:524:54: replace + with * in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
MISSED   src/renderer/mod.rs:524:40: replace + with * in <impl Widget for MarkdownView<'_>>::render in 6s build + 18s test
TIMEOUT  src/renderer/mod.rs:530:33: replace > with == in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:530:33: replace > with >= in <impl Widget for MarkdownView<'_>>::render in 9s build + 20s test
TIMEOUT  src/renderer/mod.rs:529:82: replace + with * in <impl Widget for MarkdownView<'_>>::render in 8s build + 20s test
MISSED   src/renderer/mod.rs:529:57: replace + with * in <impl Widget for MarkdownView<'_>>::render in 5s build + 19s test
TIMEOUT  src/renderer/mod.rs:535:35: replace == with != in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:540:44: replace + with - in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:540:44: replace + with * in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:542:37: replace < with == in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:542:37: replace < with > in <impl Widget for MarkdownView<'_>>::render in 8s build + 20s test
TIMEOUT  src/renderer/mod.rs:542:37: replace < with <= in <impl Widget for MarkdownView<'_>>::render in 8s build + 20s test
TIMEOUT  src/renderer/mod.rs:541:63: replace + with - in <impl Widget for MarkdownView<'_>>::render in 8s build + 20s test
TIMEOUT  src/renderer/mod.rs:541:63: replace + with * in <impl Widget for MarkdownView<'_>>::render in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:547:31: replace += with *= in <impl Widget for MarkdownView<'_>>::render in 22s build + 20s test
TIMEOUT  src/renderer/mod.rs:561:39: replace + with - in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:561:39: replace + with * in <impl Widget for MarkdownView<'_>>::render in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:586:12: delete ! in <impl Widget for MarkdownView<'_>>::render in 6s build + 20s test
TIMEOUT  src/renderer/mod.rs:726:5: replace apply_search_overlay with () in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:728:74: replace + with - in apply_search_overlay in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:728:74: replace + with * in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:743:30: replace == with != in apply_search_overlay in 11s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:32: replace && with || in apply_search_overlay in 11s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:22: replace < with == in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:22: replace < with > in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:22: replace < with <= in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:43: replace < with == in apply_search_overlay in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:43: replace < with > in apply_search_overlay in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:749:43: replace < with <= in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:767:31: replace >= with < in apply_search_overlay in 7s build + 20s test
TIMEOUT  src/renderer/mod.rs:770:43: replace + with - in apply_search_overlay in 11s build + 20s test
TIMEOUT  src/renderer/mod.rs:770:43: replace + with * in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:771:60: replace > with == in apply_search_overlay in 10s build + 20s test
TIMEOUT  src/renderer/mod.rs:771:60: replace > with < in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:771:60: replace > with >= in apply_search_overlay in 9s build + 20s test
TIMEOUT  src/renderer/mod.rs:783:31: replace >= with < in apply_search_overlay in 9s build + 20s test
TIMEOUT  src/renderer/mod.rs:784:36: replace += with -= in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:784:36: replace += with *= in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:787:36: replace + with - in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:787:36: replace + with * in apply_search_overlay in 12s build + 20s test
TIMEOUT  src/renderer/mod.rs:788:78: replace - with + in apply_search_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:788:78: replace - with / in apply_search_overlay in 21s build + 20s test
TIMEOUT  src/renderer/mod.rs:789:76: replace - with + in apply_search_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:789:76: replace - with / in apply_search_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:68: replace + with - in apply_search_overlay in 17s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:68: replace + with * in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:46: replace + with - in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:46: replace + with * in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:32: replace + with - in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:791:32: replace + with * in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:77: replace + with - in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:77: replace + with * in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:55: replace + with - in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:55: replace + with * in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:41: replace + with - in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:792:41: replace + with * in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:793:51: replace + with - in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:793:51: replace + with * in apply_search_overlay in 19s build + 20s test
TIMEOUT  src/renderer/mod.rs:793:37: replace + with - in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:793:37: replace + with * in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:799:28: replace += with -= in apply_search_overlay in 21s build + 20s test
TIMEOUT  src/renderer/mod.rs:799:28: replace += with *= in apply_search_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:801:21: replace += with -= in apply_search_overlay in 22s build + 20s test
TIMEOUT  src/renderer/mod.rs:801:21: replace += with *= in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:815:24: replace += with -= in apply_search_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:815:24: replace += with *= in apply_search_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:816:21: replace += with -= in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:816:21: replace += with *= in apply_search_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:836:5: replace focus_paragraph_bounds -> (usize, usize) with (0, 0) in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:836:5: replace focus_paragraph_bounds -> (usize, usize) with (0, 1) in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:836:5: replace focus_paragraph_bounds -> (usize, usize) with (1, 0) in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:836:5: replace focus_paragraph_bounds -> (usize, usize) with (1, 1) in 28s build + 20s test
TIMEOUT  src/renderer/mod.rs:837:10: replace == with != in focus_paragraph_bounds in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:850:26: replace + with - in focus_paragraph_bounds in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:850:26: replace + with * in focus_paragraph_bounds in 19s build + 20s test
TIMEOUT  src/renderer/mod.rs:853:27: replace + with - in focus_paragraph_bounds in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:853:27: replace + with * in focus_paragraph_bounds in 20s build + 20s test
TIMEOUT  src/renderer/mod.rs:855:19: replace - with + in focus_paragraph_bounds in 26s build + 20s test
TIMEOUT  src/renderer/mod.rs:855:19: replace - with / in focus_paragraph_bounds in 17s build + 20s test
TIMEOUT  src/renderer/mod.rs:855:30: replace - with + in focus_paragraph_bounds in 24s build + 20s test
TIMEOUT  src/renderer/mod.rs:855:30: replace - with / in focus_paragraph_bounds in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:870:5: replace apply_focus_overlay with () in 22s build + 20s test
TIMEOUT  src/renderer/mod.rs:873:74: replace + with - in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:873:74: replace + with * in apply_focus_overlay in 29s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:32: replace && with || in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:22: replace < with == in apply_focus_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:22: replace < with > in apply_focus_overlay in 22s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:22: replace < with <= in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:43: replace < with == in apply_focus_overlay in 13s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:43: replace < with > in apply_focus_overlay in 22s build + 20s test
TIMEOUT  src/renderer/mod.rs:881:43: replace < with <= in apply_focus_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:895:27: replace >= with < in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:38: replace || with && in apply_focus_overlay in 14s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:24: replace < with == in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:24: replace < with > in apply_focus_overlay in 25s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:24: replace < with <= in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:49: replace > with == in apply_focus_overlay in 17s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:49: replace > with < in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:898:49: replace > with >= in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:899:32: replace + with - in apply_focus_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:899:32: replace + with * in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:51: replace + with - in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:51: replace + with * in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:37: replace + with - in apply_focus_overlay in 18s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:37: replace + with * in apply_focus_overlay in 19s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:86: replace + with - in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:900:86: replace + with * in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:901:34: replace + with - in apply_focus_overlay in 20s build + 20s test
TIMEOUT  src/renderer/mod.rs:901:34: replace + with * in apply_focus_overlay in 15s build + 20s test
TIMEOUT  src/renderer/mod.rs:905:24: replace += with -= in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:905:24: replace += with *= in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:907:17: replace += with -= in apply_focus_overlay in 16s build + 20s test
TIMEOUT  src/renderer/mod.rs:907:17: replace += with *= in apply_focus_overlay in 17s build + 20s test
TIMEOUT  src/search.rs:70:9: replace SearchState::update_matches with () in 16s build + 20s test
TIMEOUT  src/search.rs:102:64: replace - with + in SearchState::update_matches in 17s build + 20s test
TIMEOUT  src/search.rs:102:64: replace - with / in SearchState::update_matches in 18s build + 20s test
TIMEOUT  src/search.rs:108:9: replace SearchState::push_char with () in 20s build + 20s test
TIMEOUT  src/search.rs:119:9: replace SearchState::pop_char with () in 15s build + 20s test
TIMEOUT  src/search.rs:129:9: replace SearchState::next_match with () in 15s build + 20s test
TIMEOUT  src/search.rs:129:12: delete ! in SearchState::next_match in 19s build + 20s test
TIMEOUT  src/search.rs:130:47: replace % with / in SearchState::next_match in 16s build + 20s test
TIMEOUT  src/search.rs:130:47: replace % with + in SearchState::next_match in 15s build + 20s test
TIMEOUT  src/search.rs:130:42: replace + with - in SearchState::next_match in 15s build + 20s test
TIMEOUT  src/search.rs:130:42: replace + with * in SearchState::next_match in 22s build + 20s test
TIMEOUT  src/search.rs:136:9: replace SearchState::prev_match with () in 16s build + 20s test
TIMEOUT  src/search.rs:136:12: delete ! in SearchState::prev_match in 16s build + 20s test
TIMEOUT  src/search.rs:137:29: replace == with != in SearchState::prev_match in 16s build + 20s test
TIMEOUT  src/search.rs:138:51: replace - with + in SearchState::prev_match in 18s build + 20s test
TIMEOUT  src/search.rs:138:51: replace - with / in SearchState::prev_match in 16s build + 20s test
TIMEOUT  src/search.rs:140:30: replace -= with += in SearchState::prev_match in 16s build + 20s test
TIMEOUT  src/search.rs:140:30: replace -= with /= in SearchState::prev_match in 15s build + 20s test
TIMEOUT  src/search.rs:147:9: replace SearchState::current_match -> Option<Match> with None in 16s build + 20s test
TIMEOUT  src/search.rs:147:9: replace SearchState::current_match -> Option<Match> with Some(Default::default()) in 15s build + 20s test
TIMEOUT  src/search.rs:154:9: replace SearchState::snap_to_cursor with () in 17s build + 20s test
TIMEOUT  src/search.rs:161:34: replace || with && in SearchState::snap_to_cursor in 17s build + 20s test
TIMEOUT  src/search.rs:161:20: replace > with == in SearchState::snap_to_cursor in 17s build + 20s test
TIMEOUT  src/search.rs:161:20: replace > with < in SearchState::snap_to_cursor in 26s build + 20s test
TIMEOUT  src/search.rs:161:20: replace > with >= in SearchState::snap_to_cursor in 13s build + 20s test
TIMEOUT  src/search.rs:161:56: replace && with || in SearchState::snap_to_cursor in 17s build + 20s test
TIMEOUT  src/search.rs:161:41: replace == with != in SearchState::snap_to_cursor in 25s build + 20s test
TIMEOUT  src/search.rs:161:62: replace >= with < in SearchState::snap_to_cursor in 14s build + 20s test
TIMEOUT  src/search.rs:174:9: replace SearchState::apply_replace_to_line -> String with String::new() in 15s build + 20s test
TIMEOUT  src/search.rs:174:9: replace SearchState::apply_replace_to_line -> String with "xyzzy".into() in 21s build + 20s test
TIMEOUT  src/search.rs:185:9: replace SearchState::apply_replace_all -> Vec<String> with vec![] in 18s build + 20s test
TIMEOUT  src/search.rs:185:9: replace SearchState::apply_replace_all -> Vec<String> with vec![String::new()] in 18s build + 20s test
TIMEOUT  src/search.rs:185:9: replace SearchState::apply_replace_all -> Vec<String> with vec!["xyzzy".into()] in 27s build + 20s test
TIMEOUT  src/search.rs:185:36: replace || with && in SearchState::apply_replace_all in 16s build + 20s test
TIMEOUT  src/search.rs:190:19: replace < with == in SearchState::apply_replace_all in 17s build + 20s test
TIMEOUT  src/search.rs:190:19: replace < with > in SearchState::apply_replace_all in 16s build + 20s test
TIMEOUT  src/search.rs:190:19: replace < with <= in SearchState::apply_replace_all in 14s build + 20s test
TIMEOUT  src/search.rs:204:5: replace escape_literal -> String with String::new() in 22s build + 20s test
TIMEOUT  src/search.rs:204:5: replace escape_literal -> String with "xyzzy".into() in 16s build + 20s test
TIMEOUT  src/search.rs:219:5: replace find_all_matches -> Vec<Match> with vec![] in 17s build + 20s test
TIMEOUT  src/search.rs:219:5: replace find_all_matches -> Vec<Match> with vec![Default::default()] in 20s build + 20s test
TIMEOUT  src/search.rs:222:24: replace <= with > in find_all_matches in 17s build + 20s test
TIMEOUT  src/search.rs:225:32: replace == with != in find_all_matches in 17s build + 20s test
TIMEOUT  src/search.rs:227:46: replace + with - in find_all_matches in 19s build + 20s test
TIMEOUT  src/search.rs:227:46: replace + with * in find_all_matches in 19s build + 20s test
TIMEOUT  src/search.rs:231:47: replace + with - in find_all_matches in 15s build + 20s test
TIMEOUT  src/search.rs:231:47: replace + with * in find_all_matches in 16s build + 20s test
TIMEOUT  src/status.rs:27:9: replace StatusLine::set_timed with () in 17s build + 20s test
TIMEOUT  src/status.rs:29:40: replace + with - in StatusLine::set_timed in 17s build + 20s test
TIMEOUT  src/status.rs:34:9: replace StatusLine::set_dismissible with () in 15s build + 20s test
TIMEOUT  src/status.rs:38:9: replace StatusLine::dismiss with () in 17s build + 20s test
TIMEOUT  src/status.rs:45:9: replace StatusLine::start_goto_line with () in 16s build + 20s test
TIMEOUT  src/status.rs:54:9: replace StatusLine::goto_push with () in 16s build + 20s test
TIMEOUT  src/status.rs:55:28: replace < with == in StatusLine::goto_push in 17s build + 20s test
TIMEOUT  src/status.rs:55:28: replace < with > in StatusLine::goto_push in 17s build + 20s test
TIMEOUT  src/status.rs:55:28: replace < with <= in StatusLine::goto_push in 19s build + 20s test
TIMEOUT  src/status.rs:63:9: replace StatusLine::goto_pop with () in 16s build + 20s test
TIMEOUT  src/status.rs:70:9: replace StatusLine::goto_input -> Option<&str> with None in 19s build + 20s test
TIMEOUT  src/status.rs:70:9: replace StatusLine::goto_input -> Option<&str> with Some("") in 17s build + 20s test
TIMEOUT  src/status.rs:70:9: replace StatusLine::goto_input -> Option<&str> with Some("xyzzy") in 16s build + 20s test
TIMEOUT  src/status.rs:79:9: replace StatusLine::tick with () in 16s build + 20s test
TIMEOUT  src/status.rs:87:9: replace StatusLine::message -> Option<&str> with None in 26s build + 20s test
TIMEOUT  src/status.rs:87:9: replace StatusLine::message -> Option<&str> with Some("") in 16s build + 20s test
TIMEOUT  src/status.rs:87:9: replace StatusLine::message -> Option<&str> with Some("xyzzy") in 17s build + 20s test
TIMEOUT  src/status.rs:88:13: delete match arm StatusMode::TimedMessage{text, ..} in StatusLine::message in 17s build + 20s test
TIMEOUT  src/status.rs:89:13: delete match arm StatusMode::DismissibleMessage(text) in StatusLine::message in 16s build + 20s test
TIMEOUT  src/table_format.rs:50:5: replace find_table_bounds -> Option<(usize, usize)> with None in 22s build + 20s test
TIMEOUT  src/table_format.rs:50:5: replace find_table_bounds -> Option<(usize, usize)> with Some((0, 0)) in 17s build + 20s test
TIMEOUT  src/table_format.rs:50:5: replace find_table_bounds -> Option<(usize, usize)> with Some((0, 1)) in 17s build + 20s test
TIMEOUT  src/table_format.rs:50:5: replace find_table_bounds -> Option<(usize, usize)> with Some((1, 0)) in 16s build + 20s test
TIMEOUT  src/table_format.rs:50:5: replace find_table_bounds -> Option<(usize, usize)> with Some((1, 1)) in 16s build + 20s test
TIMEOUT  src/table_format.rs:50:19: replace >= with < in find_table_bounds in 15s build + 20s test
TIMEOUT  src/table_format.rs:53:8: delete ! in find_table_bounds in 16s build + 20s test
TIMEOUT  src/table_format.rs:59:21: replace && with || in find_table_bounds in 18s build + 20s test
TIMEOUT  src/table_format.rs:59:17: replace > with == in find_table_bounds in 17s build + 20s test
TIMEOUT  src/table_format.rs:59:17: replace > with < in find_table_bounds in 17s build + 20s test
TIMEOUT  src/table_format.rs:59:17: replace > with >= in find_table_bounds in 17s build + 20s test
TIMEOUT  src/table_format.rs:59:51: replace - with + in find_table_bounds in 16s build + 20s test
TIMEOUT  src/table_format.rs:59:51: replace - with / in find_table_bounds in 16s build + 20s test
TIMEOUT  src/table_format.rs:60:15: replace -= with += in find_table_bounds in 24s build + 20s test
TIMEOUT  src/table_format.rs:60:15: replace -= with /= in find_table_bounds in 14s build + 20s test
TIMEOUT  src/table_format.rs:65:33: replace && with || in find_table_bounds in 16s build + 20s test
TIMEOUT  src/table_format.rs:65:19: replace < with == in find_table_bounds in 20s build + 20s test
TIMEOUT  src/table_format.rs:65:19: replace < with > in find_table_bounds in 18s build + 20s test
TIMEOUT  src/table_format.rs:65:19: replace < with <= in find_table_bounds in 19s build + 20s test
TIMEOUT  src/table_format.rs:65:15: replace + with - in find_table_bounds in 17s build + 20s test
TIMEOUT  src/table_format.rs:65:15: replace + with * in find_table_bounds in 19s build + 20s test
TIMEOUT  src/table_format.rs:65:61: replace + with - in find_table_bounds in 24s build + 20s test
TIMEOUT  src/table_format.rs:65:61: replace + with * in find_table_bounds in 18s build + 20s test
TIMEOUT  src/table_format.rs:66:13: replace += with -= in find_table_bounds in 19s build + 20s test
TIMEOUT  src/table_format.rs:66:13: replace += with *= in find_table_bounds in 16s build + 20s test
TIMEOUT  src/table_format.rs:70:12: replace <= with > in find_table_bounds in 17s build + 20s test
TIMEOUT  src/table_format.rs:78:5: replace is_table_line -> bool with true in 17s build + 20s test
TIMEOUT  src/table_format.rs:78:5: replace is_table_line -> bool with false in 17s build + 20s test
TIMEOUT  src/table_format.rs:79:19: replace && with || in is_table_line in 17s build + 20s test
TIMEOUT  src/table_format.rs:79:5: delete ! in is_table_line in 18s build + 20s test
TIMEOUT  src/table_format.rs:91:5: replace parse_row -> Vec<String> with vec![] in 17s build + 20s test
TIMEOUT  src/table_format.rs:91:5: replace parse_row -> Vec<String> with vec![String::new()] in 14s build + 20s test
TIMEOUT  src/table_format.rs:91:5: replace parse_row -> Vec<String> with vec!["xyzzy".into()] in 18s build + 20s test
TIMEOUT  src/table_format.rs:100:5: replace is_separator_row -> bool with true in 23s build + 20s test
TIMEOUT  src/table_format.rs:100:5: replace is_separator_row -> bool with false in 18s build + 20s test
TIMEOUT  src/table_format.rs:101:9: replace && with || in is_separator_row in 17s build + 20s test
TIMEOUT  src/table_format.rs:100:5: delete ! in is_separator_row in 18s build + 20s test
TIMEOUT  src/table_format.rs:103:46: replace && with || in is_separator_row in 17s build + 20s test
TIMEOUT  src/table_format.rs:144:5: replace pad_cell -> String with "xyzzy".into() in 14s build + 20s test
TIMEOUT  src/table_format.rs:145:12: replace >= with < in pad_cell in 14s build + 20s test
TIMEOUT  src/table_format.rs:157:5: replace format_row -> String with String::new() in 14s build + 20s test
TIMEOUT  src/table_format.rs:157:5: replace format_row -> String with "xyzzy".into() in 17s build + 20s test
TIMEOUT  src/table_format.rs:188:5: replace format_table -> Vec<String> with vec![] in 15s build + 20s test
TIMEOUT  src/table_format.rs:197:44: replace && with || in format_table in 15s build + 20s test
TIMEOUT  src/table_format.rs:197:30: replace > with == in format_table in 15s build + 20s test
TIMEOUT  src/table_format.rs:197:30: replace > with < in format_table in 13s build + 20s test
TIMEOUT  src/table_format.rs:197:30: replace > with >= in format_table in 15s build + 20s test
TIMEOUT  src/table_format.rs:214:15: replace == with != in format_table in 14s build + 20s test
TIMEOUT  src/table_format.rs:218:19: replace < with == in format_table in 14s build + 20s test
TIMEOUT  src/table_format.rs:218:19: replace < with > in format_table in 15s build + 20s test
TIMEOUT  src/table_format.rs:218:19: replace < with <= in format_table in 14s build + 20s test
TIMEOUT  src/table_format.rs:228:44: replace && with || in format_table in 14s build + 20s test
TIMEOUT  src/table_format.rs:228:29: replace == with != in format_table in 14s build + 20s test
TIMEOUT  src/table_format.rs:243:5: replace handle_format_table with () in 14s build + 20s test
TIMEOUT  src/table_format.rs:258:25: replace + with - in handle_format_table in 20s build + 20s test
TIMEOUT  src/table_format.rs:258:25: replace + with * in handle_format_table in 13s build + 20s test
TIMEOUT  src/cli.rs:28:5: replace shell_init_str -> String with String::new() in 16s build + 20s test
TIMEOUT  src/cli.rs:28:5: replace shell_init_str -> String with "xyzzy".into() in 8s build + 20s test
TIMEOUT  src/cli.rs:68:5: replace detect_shell -> String with String::new() in 6s build + 20s test
TIMEOUT  src/cli.rs:68:5: replace detect_shell -> String with "xyzzy".into() in 7s build + 20s test
TIMEOUT  src/cli.rs:69:21: replace match guard path.contains("zsh") with true in detect_shell in 6s build + 20s test
TIMEOUT  src/cli.rs:69:21: replace match guard path.contains("zsh") with false in detect_shell in 11s build + 20s test
TIMEOUT  src/cli.rs:70:21: replace match guard path.contains("bash") with true in detect_shell in 8s build + 20s test
TIMEOUT  src/cli.rs:70:21: replace match guard path.contains("bash") with false in detect_shell in 7s build + 20s test
TIMEOUT  src/cli.rs:102:5: replace print_help with () in 6s build + 20s test
TIMEOUT  src/cli.rs:133:5: replace version_string -> String with String::new() in 6s build + 20s test
TIMEOUT  src/cli.rs:133:5: replace version_string -> String with "xyzzy".into() in 7s build + 20s test
TIMEOUT  src/cli.rs:140:38: replace || with && in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:140:30: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:140:43: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:145:38: replace || with && in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:145:30: replace == with != in parse_args in 8s build + 20s test
TIMEOUT  src/cli.rs:145:43: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:151:9: delete match arm [] in parse_args in 7s build + 20s test
TIMEOUT  src/cli.rs:160:9: delete match arm [path] in parse_args in 7s build + 20s test
TIMEOUT  src/cli.rs:155:16: replace match guard a == "init" with true in parse_args in 7s build + 20s test
TIMEOUT  src/cli.rs:155:16: replace match guard a == "init" with false in parse_args in 8s build + 20s test
TIMEOUT  src/cli.rs:156:19: replace match guard a == "init" with true in parse_args in 8s build + 20s test
TIMEOUT  src/cli.rs:156:19: replace match guard a == "init" with false in parse_args in 10s build + 20s test
TIMEOUT  src/cli.rs:159:16: replace match guard a == "write-config" with true in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:159:16: replace match guard a == "write-config" with false in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:155:18: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:156:21: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:159:18: replace == with != in parse_args in 6s build + 20s test
TIMEOUT  src/cli.rs:175:5: replace run_write_config with () in 6s build + 20s test
TIMEOUT  src/cli.rs:191:12: delete ! in run_write_config in 6s build + 20s test
TIMEOUT  src/commands.rs:12:5: replace handle_save -> io::Result<()> with Ok(()) in 6s build + 20s test
TIMEOUT  src/commands.rs:15:28: replace == with != in handle_save in 6s build + 20s test
TIMEOUT  src/commands.rs:22:12: delete ! in handle_save in 6s build + 20s test
TIMEOUT  src/commands.rs:43:5: replace handle_exit -> bool with true in 9s build + 20s test
TIMEOUT  src/commands.rs:43:5: replace handle_exit -> bool with false in 7s build + 20s test
TIMEOUT  src/commands.rs:66:5: replace clamp_scroll with () in 8s build + 20s test
TIMEOUT  src/commands.rs:75:19: replace < with == in clamp_scroll in 9s build + 20s test
TIMEOUT  src/commands.rs:75:19: replace < with > in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:93:39: replace + with - in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:93:39: replace + with * in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:94:38: replace || with && in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:94:27: replace < with == in clamp_scroll in 9s build + 20s test
TIMEOUT  src/commands.rs:94:27: replace < with > in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:94:27: replace < with <= in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:94:47: replace == with != in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:94:43: replace + with - in clamp_scroll in 8s build + 20s test
TIMEOUT  src/commands.rs:102:38: replace + with - in clamp_scroll in 11s build + 20s test
TIMEOUT  src/commands.rs:102:38: replace + with * in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:104:39: replace >= with < in clamp_scroll in 8s build + 20s test
TIMEOUT  src/commands.rs:104:22: replace + with - in clamp_scroll in 6s build + 20s test
TIMEOUT  src/commands.rs:104:22: replace + with * in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:105:70: replace + with - in clamp_scroll in 9s build + 20s test
TIMEOUT  src/commands.rs:105:70: replace + with * in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:105:54: replace + with - in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:105:54: replace + with * in clamp_scroll in 7s build + 20s test
TIMEOUT  src/commands.rs:108:23: replace > with == in clamp_scroll in 113s build + 20s test
TIMEOUT  src/commands.rs:108:23: replace > with < in clamp_scroll in 105s build + 20s test
TIMEOUT  src/commands.rs:108:23: replace > with >= in clamp_scroll in 143s build + 20s test
TIMEOUT  src/commands.rs:110:55: replace - with + in clamp_scroll in 135s build + 20s test
TIMEOUT  src/commands.rs:111:27: replace > with == in clamp_scroll in 17s build + 20s test
TIMEOUT  src/commands.rs:111:27: replace > with < in clamp_scroll in 20s build + 20s test
TIMEOUT  src/commands.rs:111:27: replace > with >= in clamp_scroll in 35s build + 20s test
TIMEOUT  src/commands.rs:114:23: replace -= with += in clamp_scroll in 35s build + 20s test
TIMEOUT  src/commands.rs:114:23: replace -= with /= in clamp_scroll in 28s build + 20s test
TIMEOUT  src/commands.rs:115:21: replace -= with += in clamp_scroll in 28s build + 20s test
TIMEOUT  src/commands.rs:115:21: replace -= with /= in clamp_scroll in 20s build + 20s test
TIMEOUT  src/commands.rs:129:5: replace center_scroll with () in 30s build + 20s test
TIMEOUT  src/commands.rs:130:46: replace / with % in center_scroll in 20s build + 20s test
TIMEOUT  src/commands.rs:130:46: replace / with * in center_scroll in 21s build + 20s test
TIMEOUT  src/input.rs:66:5: replace run_decorate -> (DecorationMap, usize) with (Default::default(), 0) in 12s build + 20s test
TIMEOUT  src/input.rs:66:5: replace run_decorate -> (DecorationMap, usize) with (Default::default(), 1) in 17s build + 20s test
TIMEOUT  src/input.rs:114:5: replace screen_to_doc -> Option<(u16, u16)> with None in 16s build + 20s test
TIMEOUT  src/input.rs:114:5: replace screen_to_doc -> Option<(u16, u16)> with Some((0, 0)) in 14s build + 20s test
TIMEOUT  src/input.rs:114:5: replace screen_to_doc -> Option<(u16, u16)> with Some((0, 1)) in 17s build + 20s test
TIMEOUT  src/input.rs:114:5: replace screen_to_doc -> Option<(u16, u16)> with Some((1, 0)) in 21s build + 20s test
TIMEOUT  src/input.rs:114:5: replace screen_to_doc -> Option<(u16, u16)> with Some((1, 1)) in 32s build + 20s test
TIMEOUT  src/input.rs:117:9: replace || with && in screen_to_doc in 27s build + 20s test
TIMEOUT  src/input.rs:116:9: replace || with && in screen_to_doc in 18s build + 20s test
TIMEOUT  src/input.rs:115:9: replace || with && in screen_to_doc in 19s build + 20s test
TIMEOUT  src/input.rs:114:19: replace < with == in screen_to_doc in 43s build + 20s test
TIMEOUT  src/input.rs:114:19: replace < with > in screen_to_doc in 42s build + 20s test
TIMEOUT  src/input.rs:114:19: replace < with <= in screen_to_doc in 33s build + 20s test
TIMEOUT  src/input.rs:115:23: replace < with == in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:115:23: replace < with > in screen_to_doc in 9s build + 20s test
TIMEOUT  src/input.rs:115:23: replace < with <= in screen_to_doc in 10s build + 20s test
TIMEOUT  src/input.rs:116:23: replace >= with < in screen_to_doc in 11s build + 20s test
TIMEOUT  src/input.rs:116:40: replace + with - in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:116:40: replace + with * in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:117:23: replace >= with < in screen_to_doc in 6s build + 20s test
TIMEOUT  src/input.rs:117:40: replace + with - in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:117:40: replace + with * in screen_to_doc in 9s build + 20s test
TIMEOUT  src/input.rs:123:46: replace + with - in screen_to_doc in 8s build + 20s test
TIMEOUT  src/input.rs:123:46: replace + with * in screen_to_doc in 8s build + 20s test
TIMEOUT  src/input.rs:125:37: replace - with + in screen_to_doc in 8s build + 20s test
TIMEOUT  src/input.rs:125:37: replace - with / in screen_to_doc in 9s build + 20s test
TIMEOUT  src/input.rs:126:61: replace + with - in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:126:61: replace + with * in screen_to_doc in 6s build + 20s test
TIMEOUT  src/input.rs:145:28: replace > with == in screen_to_doc in 9s build + 20s test
TIMEOUT  src/input.rs:145:28: replace > with < in screen_to_doc in 8s build + 20s test
TIMEOUT  src/input.rs:145:28: replace > with >= in screen_to_doc in 6s build + 20s test
TIMEOUT  src/input.rs:145:16: replace + with - in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:145:16: replace + with * in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:146:36: replace - with + in screen_to_doc in 13s build + 20s test
TIMEOUT  src/input.rs:146:36: replace - with / in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:153:36: replace > with == in screen_to_doc in 7s build + 20s test
TIMEOUT  src/input.rs:153:36: replace > with < in screen_to_doc in 28s build + 20s test
TIMEOUT  src/input.rs:153:36: replace > with >= in screen_to_doc in 58s build + 20s test
TIMEOUT  src/input.rs:159:43: replace + with - in screen_to_doc in 11s build + 20s test
TIMEOUT  src/input.rs:159:43: replace + with * in screen_to_doc in 15s build + 20s test
TIMEOUT  src/input.rs:162:13: replace += with -= in screen_to_doc in 64s build + 20s test
TIMEOUT  src/input.rs:162:13: replace += with *= in screen_to_doc in 61s build + 20s test
TIMEOUT  src/input.rs:175:5: replace is_navigation_key -> bool with true in 59s build + 20s test
TIMEOUT  src/input.rs:175:5: replace is_navigation_key -> bool with false in 50s build + 20s test
TIMEOUT  src/input.rs:191:5: replace handle_pair_wrap -> bool with true in 25s build + 20s test
TIMEOUT  src/input.rs:191:5: replace handle_pair_wrap -> bool with false in 10s build + 20s test
TIMEOUT  src/input.rs:192:43: replace | with & in handle_pair_wrap in 11s build + 20s test
TIMEOUT  src/input.rs:197:9: delete match arm KeyCode::Char('(') in handle_pair_wrap in 11s build + 20s test
TIMEOUT  src/input.rs:198:9: delete match arm KeyCode::Char('[') in handle_pair_wrap in 10s build + 20s test
TIMEOUT  src/input.rs:199:9: delete match arm KeyCode::Char('{') in handle_pair_wrap in 11s build + 20s test
TIMEOUT  src/input.rs:200:9: delete match arm KeyCode::Char('"') in handle_pair_wrap in 10s build + 20s test
TIMEOUT  src/input.rs:201:9: delete match arm KeyCode::Char('\'') in handle_pair_wrap in 10s build + 20s test
TIMEOUT  src/input.rs:202:9: delete match arm KeyCode::Char('`') in handle_pair_wrap in 9s build + 20s test
TIMEOUT  src/input.rs:203:9: delete match arm KeyCode::Char('*') in handle_pair_wrap in 10s build + 20s test
TIMEOUT  src/input.rs:204:9: delete match arm KeyCode::Char('_') in handle_pair_wrap in 9s build + 20s test
TIMEOUT  src/input.rs:222:5: replace do_replace_current with () in 11s build + 20s test
TIMEOUT  src/input.rs:230:24: replace - with + in do_replace_current in 12s build + 20s test
TIMEOUT  src/input.rs:230:24: replace - with / in do_replace_current in 9s build + 20s test
TIMEOUT  src/input.rs:252:5: replace do_replace_all with () in 13s build + 20s test
TIMEOUT  src/input.rs:262:25: replace - with + in do_replace_all in 11s build + 20s test
TIMEOUT  src/input.rs:262:25: replace - with / in do_replace_all in 19s build + 20s test
TIMEOUT  src/input.rs:296:9: delete match arm (KeyModifiers::NONE, KeyCode::Esc) in handle_search_key in 29s build + 20s test
TIMEOUT  src/input.rs:301:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('s')) |(KeyModifiers::SUPER, KeyCode::Char('s')) in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:308:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('f')) |(KeyModifiers::NONE, KeyCode::Enter) in handle_search_key in 19s build + 20s test
TIMEOUT  src/input.rs:330:9: delete match arm (KeyModifiers::SHIFT, KeyCode::Enter) in handle_search_key in 23s build + 20s test
TIMEOUT  src/input.rs:342:9: delete match arm (KeyModifiers::NONE, KeyCode::Backspace) in handle_search_key in 27s build + 20s test
TIMEOUT  src/input.rs:351:9: delete match arm (KeyModifiers::ALT, KeyCode::Char('r')) in handle_search_key in 21s build + 20s test
TIMEOUT  src/input.rs:361:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('h')) in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:369:9: delete match arm (KeyModifiers::NONE, KeyCode::Tab) in handle_search_key in 17s build + 20s test
TIMEOUT  src/input.rs:379:16: replace match guard app.search.as_ref().is_some_and(|s| s.show_replace) with true in handle_search_key in 36s build + 20s test
TIMEOUT  src/input.rs:379:16: replace match guard app.search.as_ref().is_some_and(|s| s.show_replace) with false in handle_search_key in 58s build + 20s test
TIMEOUT  src/input.rs:385:14: replace match guard !k .modifiers .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) && matches!(k.code, KeyCode::Char(_)) with true in handle_search_key in 37s build + 20s test
TIMEOUT  src/input.rs:385:14: replace match guard !k .modifiers .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) && matches!(k.code, KeyCode::Char(_)) with false in handle_search_key in 14s build + 20s test
TIMEOUT  src/input.rs:311:17: replace && with || in handle_search_key in 21s build + 20s test
TIMEOUT  src/input.rs:310:40: replace == with != in handle_search_key in 26s build + 20s test
TIMEOUT  src/input.rs:314:54: replace && with || in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:314:38: delete ! in handle_search_key in 19s build + 20s test
TIMEOUT  src/input.rs:355:32: delete ! in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:363:34: delete ! in handle_search_key in 24s build + 20s test
TIMEOUT  src/input.rs:373:34: delete ! in handle_search_key in 17s build + 20s test
TIMEOUT  src/input.rs:388:13: replace && with || in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:385:14: delete ! in handle_search_key in 33s build + 20s test
TIMEOUT  src/input.rs:387:67: replace | with & in handle_search_key in 21s build + 20s test
TIMEOUT  src/input.rs:387:67: replace | with ^ in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:387:47: replace | with & in handle_search_key in 20s build + 20s test
TIMEOUT  src/input.rs:387:47: replace | with ^ in handle_search_key in 23s build + 20s test
TIMEOUT  src/input.rs:419:9: delete match arm (KeyModifiers::NONE, KeyCode::Esc) in handle_goto_line_key in 14s build + 20s test
TIMEOUT  src/input.rs:423:9: delete match arm (KeyModifiers::NONE, KeyCode::Enter) in handle_goto_line_key in 15s build + 20s test
TIMEOUT  src/input.rs:439:9: delete match arm (KeyModifiers::NONE, KeyCode::Backspace) in handle_goto_line_key in 45s build + 20s test
TIMEOUT  src/input.rs:444:51: replace match guard c.is_ascii_digit() with true in handle_goto_line_key in 29s build + 20s test
TIMEOUT  src/input.rs:444:51: replace match guard c.is_ascii_digit() with false in handle_goto_line_key in 28s build + 20s test
TIMEOUT  src/input.rs:429:22: replace >= with < in handle_goto_line_key in 15s build + 20s test
TIMEOUT  src/input.rs:432:33: replace - with + in handle_goto_line_key in 34s build + 20s test
TIMEOUT  src/input.rs:432:33: replace - with / in handle_goto_line_key in 20s build + 20s test
TIMEOUT  src/input.rs:475:13: replace && with || in handle_key_event in 14s build + 20s test
TIMEOUT  src/input.rs:474:67: replace | with ^ in handle_key_event in 15s build + 20s test
TIMEOUT  src/input.rs:474:47: replace | with & in handle_key_event in 12s build + 20s test
TIMEOUT  src/input.rs:474:47: replace | with ^ in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:480:13: delete match arm KeyCode::Char('y') | KeyCode::Char('Y') in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:481:13: delete match arm KeyCode::Char('n') | KeyCode::Char('N') in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:506:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('s')) |(KeyModifiers::SUPER, KeyCode::Char('s')) in handle_key_event in 10s build + 20s test
TIMEOUT  src/input.rs:511:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('f')) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:523:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('h')) in handle_key_event in 10s build + 20s test
TIMEOUT  src/input.rs:534:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('x')) |(KeyModifiers::NONE, KeyCode::Esc) in handle_key_event in 10s build + 20s test
TIMEOUT  src/input.rs:553:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('z')) in handle_key_event in 9s build + 20s test
TIMEOUT  src/input.rs:563:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('y')) in handle_key_event in 15s build + 20s test
TIMEOUT  src/input.rs:573:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('g')) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:579:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('t')) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:585:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('d')) in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:590:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Char('r')) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:593:9: delete match arm (KeyModifiers::ALT, KeyCode::Char('t')) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:599:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Up) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:605:9: delete match arm (KeyModifiers::CONTROL, KeyCode::Down) in handle_key_event in 11s build + 20s test
TIMEOUT  src/input.rs:613:9: delete match arm (KeyModifiers::NONE, KeyCode::Down) in handle_key_event in 10s build + 20s test
TIMEOUT  src/input.rs:614:9: delete match arm (KeyModifiers::NONE, KeyCode::Up) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:615:9: delete match arm (KeyModifiers::SHIFT, KeyCode::Down) in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:616:9: delete match arm (KeyModifiers::SHIFT, KeyCode::Up) in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:621:9: delete match arm (KeyModifiers::NONE, KeyCode::Tab) in handle_key_event in 9s build + 20s test
TIMEOUT  src/input.rs:580:35: delete ! in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:586:30: delete ! in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:607:46: replace + with - in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:607:46: replace + with * in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:624:29: replace - with + in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:624:29: replace - with / in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:624:36: replace % with / in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:624:36: replace % with + in handle_key_event in 8s build + 20s test
TIMEOUT  src/input.rs:642:24: replace && with || in handle_key_event in 9s build + 20s test
TIMEOUT  src/input.rs:642:16: delete ! in handle_key_event in 10s build + 20s test
TIMEOUT  src/input.rs:648:47: replace != with == in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:651:20: delete ! in handle_key_event in 7s build + 20s test
TIMEOUT  src/input.rs:680:11: replace == with != in handle_visual_move in 6s build + 20s test
TIMEOUT  src/input.rs:721:27: replace < with == in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:721:27: replace < with > in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:721:27: replace < with <= in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:721:23: replace + with - in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:721:23: replace + with * in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:722:34: replace + with - in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:722:34: replace + with * in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:723:31: replace < with == in handle_visual_move in 10s build + 20s test
TIMEOUT  src/input.rs:723:31: replace < with > in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:723:31: replace < with <= in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:723:27: replace + with - in handle_visual_move in 7s build + 20s test
TIMEOUT  src/input.rs:723:27: replace + with * in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:724:22: replace + with - in handle_visual_move in 7s build + 20s test
TIMEOUT  src/input.rs:724:22: replace + with * in handle_visual_move in 10s build + 20s test
TIMEOUT  src/input.rs:729:23: replace > with == in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:729:23: replace > with < in handle_visual_move in 9s build + 20s test
TIMEOUT  src/input.rs:729:23: replace > with >= in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:730:34: replace - with + in handle_visual_move in 13s build + 20s test
TIMEOUT  src/input.rs:730:34: replace - with / in handle_visual_move in 11s build + 20s test
TIMEOUT  src/input.rs:731:27: replace > with == in handle_visual_move in 17s build + 20s test
TIMEOUT  src/input.rs:731:27: replace > with < in handle_visual_move in 9s build + 20s test
TIMEOUT  src/input.rs:731:27: replace > with >= in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:732:32: replace - with + in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:732:32: replace - with / in handle_visual_move in 8s build + 20s test
TIMEOUT  src/input.rs:801:5: replace event_loop -> io::Result<()> with Ok(()) in 7s build + 20s test
TIMEOUT  src/input.rs:849:33: replace || with && in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:849:83: replace >= with < in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:870:49: replace && with || in event_loop in 9s build + 20s test
TIMEOUT  src/input.rs:870:17: delete ! in event_loop in 11s build + 20s test
TIMEOUT  src/input.rs:870:77: replace > with == in event_loop in 7s build + 20s test
TIMEOUT  src/input.rs:870:77: replace > with < in event_loop in 7s build + 20s test
TIMEOUT  src/input.rs:870:77: replace > with >= in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:873:38: replace + with - in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:873:38: replace + with * in event_loop in 11s build + 20s test
TIMEOUT  src/input.rs:875:17: delete field y from struct Rect expression in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:876:17: delete field height from struct Rect expression in event_loop in 9s build + 20s test
TIMEOUT  src/input.rs:875:40: replace + with - in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:875:40: replace + with * in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:887:54: replace + with - in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:887:54: replace + with * in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:896:16: delete ! in event_loop in 7s build + 20s test
TIMEOUT  src/input.rs:921:65: replace && with || in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:921:33: delete ! in event_loop in 8s build + 20s test
TIMEOUT  src/input.rs:921:89: replace > with == in event_loop in 11s build + 20s test
TIMEOUT  src/input.rs:921:89: replace > with < in event_loop in 11s build + 20s test
TIMEOUT  src/input.rs:921:89: replace > with >= in event_loop in 9s build + 20s test
TIMEOUT  src/input.rs:922:40: delete field height from struct Rect expression in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:930:21: delete field y from struct Rect expression in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:931:21: delete field height from struct Rect expression in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:930:40: replace + with - in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:930:40: replace + with * in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:940:46: replace && with || in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:940:42: replace > with == in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:940:42: replace > with < in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:940:42: replace > with >= in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:940:67: replace > with == in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:940:67: replace > with < in event_loop in 11s build + 20s test
TIMEOUT  src/input.rs:940:67: replace > with >= in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:949:21: delete field y from struct Rect expression in event_loop in 16s build + 20s test
TIMEOUT  src/input.rs:950:21: delete field height from struct Rect expression in event_loop in 16s build + 20s test
TIMEOUT  src/input.rs:949:37: replace + with - in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:949:37: replace + with * in event_loop in 10s build + 20s test
TIMEOUT  src/input.rs:996:17: delete match arm Event::Key(k) in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:1038:17: delete match arm Event::Mouse(mouse) in event_loop in 9s build + 20s test
TIMEOUT  src/input.rs:1086:17: delete match arm Event::Resize(_, _) in event_loop in 9s build + 20s test
TIMEOUT  src/input.rs:1039:21: delete match arm MouseEventKind::ScrollDown in event_loop in 13s build + 20s test
TIMEOUT  src/input.rs:1044:21: delete match arm MouseEventKind::ScrollUp in event_loop in 14s build + 20s test
TIMEOUT  src/input.rs:1048:21: delete match arm MouseEventKind::Down(MouseButton::Left) in event_loop in 28s build + 20s test
TIMEOUT  src/input.rs:1065:21: delete match arm MouseEventKind::Drag(MouseButton::Left) in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:1041:58: replace + with - in event_loop in 12s build + 20s test
TIMEOUT  src/input.rs:1041:58: replace + with * in event_loop in 24s build + 20s test
TIMEOUT  src/input.rs:1077:32: delete ! in event_loop in 20s build + 20s test
TIMEOUT  src/decoration/spans.rs:7:5: replace line_start_bytes -> Vec<usize> with vec![] in 40s build + 20s test
TIMEOUT  src/decoration/spans.rs:7:5: replace line_start_bytes -> Vec<usize> with vec![0] in 44s build + 20s test
TIMEOUT  src/decoration/spans.rs:7:5: replace line_start_bytes -> Vec<usize> with vec![1] in 44s build + 20s test
TIMEOUT  src/decoration/spans.rs:9:14: replace == with != in line_start_bytes in 46s build + 20s test
TIMEOUT  src/decoration/spans.rs:10:27: replace + with - in line_start_bytes in 46s build + 20s test
TIMEOUT  src/decoration/spans.rs:10:27: replace + with * in line_start_bytes in 45s build + 20s test
TIMEOUT  src/decoration/spans.rs:25:5: replace byte_to_line_char -> (usize, usize) with (0, 0) in 45s build + 20s test
TIMEOUT  src/decoration/spans.rs:25:5: replace byte_to_line_char -> (usize, usize) with (0, 1) in 25s build + 20s test
TIMEOUT  src/decoration/spans.rs:25:5: replace byte_to_line_char -> (usize, usize) with (1, 0) in 15s build + 20s test
TIMEOUT  src/decoration/spans.rs:25:5: replace byte_to_line_char -> (usize, usize) with (1, 1) in 27s build + 20s test
TIMEOUT  src/decoration/spans.rs:27:33: replace <= with > in byte_to_line_char in 18s build + 20s test
TIMEOUT  src/decoration/spans.rs:37:5: replace line_char_len -> usize with 0 in 20s build + 20s test
TIMEOUT  src/decoration/spans.rs:37:5: replace line_char_len -> usize with 1 in 43s build + 20s test
TIMEOUT  src/decoration/spans.rs:38:30: replace < with == in line_char_len in 49s build + 20s test
TIMEOUT  src/decoration/spans.rs:38:30: replace < with > in line_char_len in 49s build + 20s test
TIMEOUT  src/decoration/spans.rs:38:30: replace < with <= in line_char_len in 29s build + 20s test
TIMEOUT  src/decoration/spans.rs:38:26: replace + with - in line_char_len in 27s build + 20s test
TIMEOUT  src/decoration/spans.rs:38:26: replace + with * in line_char_len in 16s build + 20s test
TIMEOUT  src/decoration/spans.rs:39:30: replace + with - in line_char_len in 16s build + 20s test
TIMEOUT  src/decoration/spans.rs:39:30: replace + with * in line_char_len in 16s build + 20s test
TIMEOUT  src/decoration/spans.rs:48:5: replace push_span with () in 16s build + 20s test
TIMEOUT  src/decoration/spans.rs:53:5: replace make_span -> StyledSpan with Default::default() in 15s build + 20s test
TIMEOUT  src/decoration/spans.rs:54:9: delete field char_start from struct StyledSpan expression in make_span in 20s build + 20s test
TIMEOUT  src/decoration/spans.rs:55:9: delete field char_end from struct StyledSpan expression in make_span in 17s build + 20s test
TIMEOUT  src/decoration/spans.rs:56:9: delete field style from struct StyledSpan expression in make_span in 22s build + 20s test
TIMEOUT  src/decoration/spans.rs:76:5: replace add_byte_range_span with () in 23s build + 20s test
TIMEOUT  src/decoration/spans.rs:76:19: replace >= with < in add_byte_range_span in 23s build + 20s test
TIMEOUT  src/decoration/spans.rs:85:31: replace == with != in add_byte_range_span in 19s build + 20s test
TIMEOUT  src/decoration/spans.rs:86:29: replace == with != in add_byte_range_span in 24s build + 20s test
TIMEOUT  src/decoration/spans.rs:87:32: replace + with - in add_byte_range_span in 32s build + 20s test
TIMEOUT  src/decoration/spans.rs:87:32: replace + with * in add_byte_range_span in 27s build + 20s test
TIMEOUT  src/decoration/spans.rs:91:39: replace + with - in add_byte_range_span in 24s build + 20s test
TIMEOUT  src/decoration/spans.rs:91:39: replace + with * in add_byte_range_span in 22s build + 20s test
TIMEOUT  src/decoration/spans.rs:96:17: delete field char_start from struct StyledSpan expression in add_byte_range_span in 26s build + 20s test
TIMEOUT  src/decoration/spans.rs:97:17: delete field char_end from struct StyledSpan expression in add_byte_range_span in 25s build + 20s test
TIMEOUT  src/decoration/spans.rs:98:17: delete field style from struct StyledSpan expression in add_byte_range_span in 26s build + 20s test
TIMEOUT  src/decoration/spans.rs:99:17: delete field is_blockquote from struct StyledSpan expression in add_byte_range_span in 27s build + 20s test
TIMEOUT  src/decoration/spans.rs:100:17: delete field full_line_bg from struct StyledSpan expression in add_byte_range_span in 28s build + 20s test
TIMEOUT  src/decoration/words.rs:5:5: replace count_words -> usize with 0 in 27s build + 20s test
TIMEOUT  src/decoration/words.rs:5:5: replace count_words -> usize with 1 in 27s build + 20s test
TIMEOUT  src/decoration/words.rs:16:5: replace link_split_char_idx -> Option<usize> with None in 42s build + 20s test
TIMEOUT  src/decoration/words.rs:16:5: replace link_split_char_idx -> Option<usize> with Some(0) in 32s build + 20s test
TIMEOUT  src/decoration/words.rs:16:5: replace link_split_char_idx -> Option<usize> with Some(1) in 35s build + 20s test
TIMEOUT  src/decoration/words.rs:18:13: replace < with == in link_split_char_idx in 57s build + 20s test
TIMEOUT  src/decoration/words.rs:18:13: replace < with > in link_split_char_idx in 69s build + 20s test
TIMEOUT  src/decoration/words.rs:18:13: replace < with <= in link_split_char_idx in 72s build + 20s test
TIMEOUT  src/decoration/words.rs:20:13: delete match arm '[' in link_split_char_idx in 102s build + 20s test
TIMEOUT  src/decoration/words.rs:21:20: replace match guard i + 1 < chars.len() && chars[i + 1] == '(' with true in link_split_char_idx in 81s build + 20s test
TIMEOUT  src/decoration/words.rs:21:20: replace match guard i + 1 < chars.len() && chars[i + 1] == '(' with false in link_split_char_idx in 126s build + 20s test
TIMEOUT  src/decoration/words.rs:20:34: replace += with -= in link_split_char_idx in 89s build + 20s test
TIMEOUT  src/decoration/words.rs:20:34: replace += with *= in link_split_char_idx in 33s build + 20s test
TIMEOUT  src/decoration/words.rs:21:40: replace && with || in link_split_char_idx in 29s build + 20s test
TIMEOUT  src/decoration/words.rs:21:26: replace < with == in link_split_char_idx in 45s build + 20s test
TIMEOUT  src/decoration/words.rs:21:26: replace < with > in link_split_char_idx in 25s build + 20s test
TIMEOUT  src/decoration/words.rs:21:26: replace < with <= in link_split_char_idx in 23s build + 20s test
TIMEOUT  src/decoration/words.rs:21:22: replace + with - in link_split_char_idx in 36s build + 20s test
TIMEOUT  src/decoration/words.rs:21:22: replace + with * in link_split_char_idx in 31s build + 20s test
TIMEOUT  src/decoration/words.rs:21:56: replace == with != in link_split_char_idx in 25s build + 20s test
TIMEOUT  src/decoration/words.rs:21:51: replace + with - in link_split_char_idx in 15s build + 20s test
TIMEOUT  src/decoration/words.rs:21:51: replace + with * in link_split_char_idx in 28s build + 20s test
TIMEOUT  src/decoration/words.rs:22:34: replace <= with > in link_split_char_idx in 18s build + 20s test
TIMEOUT  src/decoration/words.rs:29:11: replace += with -= in link_split_char_idx in 17s build + 20s test
TIMEOUT  src/decoration/words.rs:36:5: replace count_chars_in -> usize with 0 in 16s build + 20s test
TIMEOUT  src/decoration/words.rs:36:5: replace count_chars_in -> usize with 1 in 19s build + 20s test
TIMEOUT  src/renderer/status.rs:27:5: replace pill1_parts -> (Span<'static>, Color) with (Default::default(), Default::default()) in 16s build + 20s test
TIMEOUT  src/renderer/status.rs:49:5: replace render_status_bar with () in 19s build + 20s test
TIMEOUT  src/renderer/status.rs:98:5: replace build_timed_message_bar -> Line<'static> with Default::default() in 17s build + 20s test
TIMEOUT  src/renderer/status.rs:122:5: replace build_normal_status_bar -> Line<'static> with Default::default() in 25s build + 20s test
TIMEOUT  src/renderer/status.rs:151:5: replace build_goto_line_bar -> Line<'static> with Default::default() in 20s build + 20s test
TIMEOUT  src/renderer/status.rs:177:5: replace render_info_line with () in 17s build + 20s test
TIMEOUT  src/renderer/status.rs:193:9: delete field width from struct Rect expression in render_info_line in 17s build + 20s test
TIMEOUT  src/renderer/status.rs:221:8: delete ! in render_info_line in 15s build + 20s test
TIMEOUT  src/renderer/status.rs:223:15: replace <= with > in render_info_line in 19s build + 20s test
TIMEOUT  src/renderer/status.rs:225:17: delete field x from struct Rect expression in render_info_line in 15s build + 20s test
TIMEOUT  src/renderer/status.rs:226:17: delete field width from struct Rect expression in render_info_line in 15s build + 20s test
TIMEOUT  src/renderer/status.rs:225:40: replace - with + in render_info_line in 14s build + 20s test
TIMEOUT  src/renderer/status.rs:225:40: replace - with / in render_info_line in 15s build + 20s test
TIMEOUT  src/renderer/status.rs:225:27: replace + with - in render_info_line in 17s build + 20s test
TIMEOUT  src/renderer/status.rs:225:27: replace + with * in render_info_line in 14s build + 20s test
TIMEOUT  src/renderer/utils.rs:8:5: replace shorten_path -> String with String::new() in 13s build + 20s test
TIMEOUT  src/renderer/utils.rs:8:5: replace shorten_path -> String with "xyzzy".into() in 22s build + 20s test
TIMEOUT  src/renderer/utils.rs:19:5: replace format_thousands -> String with String::new() in 21s build + 20s test
TIMEOUT  src/renderer/utils.rs:19:5: replace format_thousands -> String with "xyzzy".into() in 20s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:18: replace && with || in format_thousands in 21s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:14: replace > with == in format_thousands in 19s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:14: replace > with < in format_thousands in 20s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:14: replace > with >= in format_thousands in 26s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:26: replace - with + in format_thousands in 17s build + 20s test
TIMEOUT  src/renderer/utils.rs:24:26: replace - with / in format_thousands in 31s build + 20s test
TIMEOUT  src/renderer/utils.rs:43:5: replace split_into_spans -> Vec<Span<'static>> with vec![] in 37s build + 20s test
TIMEOUT  src/renderer/utils.rs:43:5: replace split_into_spans -> Vec<Span<'static>> with vec![Default::default()] in 29s build + 20s test
TIMEOUT  src/renderer/utils.rs:61:20: replace >= with < in split_into_spans in 17s build + 20s test
TIMEOUT  src/renderer/utils.rs:66:21: replace < with == in split_into_spans in 35s build + 20s test
TIMEOUT  src/renderer/utils.rs:66:21: replace < with > in split_into_spans in 17s build + 20s test
TIMEOUT  src/renderer/utils.rs:66:21: replace < with <= in split_into_spans in 28s build + 20s test
TIMEOUT  src/renderer/utils.rs:77:33: replace < with == in split_into_spans in 20s build + 20s test
TIMEOUT  src/renderer/utils.rs:77:33: replace < with > in split_into_spans in 18s build + 20s test
TIMEOUT  src/renderer/utils.rs:77:33: replace < with <= in split_into_spans in 18s build + 20s test
TIMEOUT  src/renderer/utils.rs:91:17: replace < with == in split_into_spans in 17s build + 20s test
TIMEOUT  src/renderer/utils.rs:91:17: replace < with > in split_into_spans in 23s build + 20s test
TIMEOUT  src/renderer/utils.rs:91:17: replace < with <= in split_into_spans in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:11:5: replace search_bar_height -> u16 with 0 in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:11:5: replace search_bar_height -> u16 with 1 in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:22:5: replace render_search_bar with () in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:32:29: delete field height from struct Rect expression in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:32:40: delete field y from struct Rect expression in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:37:21: replace + with - in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:37:21: replace + with * in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:38:32: replace && with || in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:38:47: replace >= with < in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:39:25: replace + with - in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:39:25: replace + with * in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:39:45: replace + with - in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:39:45: replace + with * in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:65:14: replace >= with < in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:65:24: replace + with - in render_search_bar in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:65:24: replace + with * in render_search_bar in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:67:11: replace += with -= in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:67:11: replace += with *= in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:73:14: replace >= with < in render_search_bar in 38s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:73:24: replace + with - in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:73:24: replace + with * in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:75:11: replace += with -= in render_search_bar in 15s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:75:11: replace += with *= in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:79:8: delete ! in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:82:30: replace + with - in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:82:30: replace + with * in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:85:30: replace + with - in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:85:30: replace + with * in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:86:19: replace >= with < in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:86:29: replace + with - in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:86:29: replace + with * in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:92:28: replace && with || in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:92:43: replace >= with < in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:93:38: replace + with - in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:93:38: replace + with * in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:95:34: delete ! in render_search_bar in 34s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:105:29: replace + with - in render_search_bar in 15s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:105:29: replace + with * in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:109:19: replace >= with < in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:109:29: replace + with - in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:109:29: replace + with * in render_search_bar in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:111:16: replace += with -= in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:111:16: replace += with *= in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:114:42: replace || with && in render_search_bar in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:114:19: replace >= with < in render_search_bar in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:114:29: replace + with - in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:114:29: replace + with * in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:114:48: replace >= with < in render_search_bar in 16s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:116:16: replace += with -= in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:116:16: replace += with *= in render_search_bar in 18s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:119:29: replace + with - in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:119:29: replace + with * in render_search_bar in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:120:19: replace >= with < in render_search_bar in 17s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:120:29: replace + with - in render_search_bar in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:120:29: replace + with * in render_search_bar in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:56: replace + with - in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:56: replace + with * in 27s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:51: replace + with - in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:51: replace + with * in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:47: replace + with - in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:47: replace + with * in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:42: replace + with - in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:42: replace + with * in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:38: replace + with - in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:38: replace + with * in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:33: replace + with - in 27s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:33: replace + with * in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:29: replace + with - in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:29: replace + with * in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:136:24: replace + with * in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:138:28: replace + with - in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:138:28: replace + with * in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:149:38: replace + with - in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:149:38: replace + with * in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:157:5: replace render_search_help_modal with () in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:158:8: delete ! in render_search_help_modal in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:34: replace || with && in render_search_help_modal in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:26: replace < with == in render_search_help_modal in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:26: replace < with > in render_search_help_modal in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:26: replace < with <= in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:56: replace < with == in render_search_help_modal in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:56: replace < with > in render_search_help_modal in 32s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:161:56: replace < with <= in render_search_help_modal in 32s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:172:28: replace + with - in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:172:28: replace + with * in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:172:72: replace / with % in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:172:72: replace / with * in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:181:28: replace - with + in render_search_help_modal in 52s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:181:28: replace - with / in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:185:25: replace + with - in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:185:25: replace + with * in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:185:21: replace + with - in render_search_help_modal in 25s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:185:21: replace + with * in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:19: replace && with || in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:14: replace >= with < in render_search_help_modal in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:24: replace < with == in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:24: replace < with > in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:24: replace < with <= in render_search_help_modal in 24s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:28: replace + with - in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:187:28: replace + with * in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:188:37: replace - with + in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:188:37: replace - with / in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:189:28: replace == with != in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:195:21: replace - with + in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:195:21: replace - with / in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:195:13: replace + with - in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:195:13: replace + with * in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:199:25: replace + with - in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:199:25: replace + with * in render_search_help_modal in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:199:21: replace + with - in render_search_help_modal in 23s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:199:21: replace + with * in render_search_help_modal in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:203:12: replace += with -= in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:203:12: replace += with *= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:206:52: replace += with -= in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:206:52: replace += with *= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:207:52: replace += with -= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:207:52: replace += with *= in render_search_help_modal in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:210:52: replace += with -= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:210:52: replace += with *= in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:212:52: replace += with -= in render_search_help_modal in 19s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:212:52: replace += with *= in render_search_help_modal in 21s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:214:52: replace += with -= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:214:52: replace += with *= in render_search_help_modal in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:218:52: replace += with -= in render_search_help_modal in 22s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:218:52: replace += with *= in render_search_help_modal in 20s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:223:31: replace - with + in render_search_help_modal in 26s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:223:31: replace - with / in render_search_help_modal in 67s build + 20s test
TIMEOUT  src/renderer/search_bar.rs:223:23: replace + with - in render_search_help_modal in 59s build + 20s test
