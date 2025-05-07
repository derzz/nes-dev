// use nes::run_nestest_and_capture;

// #[test]
// fn test_trace_matches() {
//     let expected = std::fs::read_to_string("exact.txt").unwrap();
//     let expected_lines: Vec<_> = expected.lines().collect();
//     let actual_lines = run_nestest_and_capture();
//     for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
//         assert_eq!(exp, act, "Difference at line {}", i + 1);
//     }
// }
