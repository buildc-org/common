use super::*;

#[test]
fn the_bar_climbs_through_a_step_and_stops_short_of_the_next() {
    let seen: Vec<u16> = (0..400).map(|within| crept(2, 87, within)).collect();
    assert_eq!(seen[0], 2);
    assert!(
        seen.windows(2).all(|pair| pair[1] >= pair[0]),
        "{:?}",
        &seen[..40]
    );
    assert!(seen[10] > 10, "{}", seen[10]);
    assert!(seen[50] > 45, "{}", seen[50]);
    assert!(seen.iter().all(|at| *at < 89), "{}", seen[399]);
}

#[test]
fn the_progress_region_draws_the_step_the_messages_and_the_line_beneath() {
    let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
    let notes = ["first".to_string(), "second".to_string()];
    terminal
        .draw(|frame| {
            working(
                frame,
                frame.area(),
                50,
                0,
                "7/12 install OS",
                &notes,
                "log: /run/bootc-anywhere.log",
            )
        })
        .unwrap();
    let drawn = terminal.backend().to_string();
    assert!(drawn.contains("50%"), "{drawn}");
    assert!(drawn.contains("7/12 install OS"), "{drawn}");
    assert!(!drawn.contains("output"), "{drawn}");
    let rows: Vec<String> = drawn.lines().map(|row| row.replace('"', "")).collect();
    let name = rows
        .iter()
        .position(|row| row.contains("7/12 install OS"))
        .unwrap_or_else(|| panic!("{drawn}"));
    assert!(rows[name + 1].trim().is_empty(), "{drawn}");
    assert!(rows[name + 2].contains("first"), "{drawn}");
    assert!(drawn.contains(TURNING[0]), "{drawn}");
    assert_eq!(
        TURNING
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        10
    );
    assert!(
        drawn.contains("first") && drawn.contains("second"),
        "{drawn}"
    );
    assert!(drawn.contains("log: /run/bootc-anywhere.log"), "{drawn}");
}

#[test]
fn the_step_line_is_the_spinner_beside_the_step() {
    assert_eq!(
        step_line(0, "6/10 Installing OS"),
        format!("{} 6/10 Installing OS", TURNING[0])
    );
    assert_eq!(
        step_line(TURNING.len(), "6/10 Installing OS"),
        format!("{} 6/10 Installing OS", TURNING[0])
    );
}

#[test]
fn the_progress_console_fills_the_body_with_the_tail_of_the_log() {
    let notes: Vec<String> = (0..12).map(|n| format!("line {n}")).collect();
    let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
    terminal
        .draw(|frame| working(frame, frame.area(), 50, 0, "7/12 install OS", &notes, ""))
        .unwrap();
    let drawn = terminal.backend().to_string();
    assert!(drawn.contains("line 11"), "the last line is drawn: {drawn}");
    assert!(
        !drawn.contains("line 0"),
        "the oldest line is dropped: {drawn}"
    );
}
