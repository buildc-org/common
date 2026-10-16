use super::*;

#[test]
fn a_terminal_that_reports_no_size_is_given_one() {
    assert!(unsized_tty(None));
    assert!(unsized_tty(Some((0, 0))));
    assert!(unsized_tty(Some((80, 0))));
    assert!(!unsized_tty(Some((80, 24))));
}

#[test]
fn a_form_scrolls_to_keep_its_focused_line_visible() {
    let lines: Vec<Line<'static>> = (0..12)
        .map(|row| Line::from(format!("row {row}")))
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
    terminal
        .draw(|frame| sheet_of(frame, frame.area(), &lines, "", 11, 0))
        .unwrap();
    let drawn = terminal.backend().to_string();
    assert!(drawn.contains("row 11"), "{drawn}");
    assert!(!drawn.contains("row 0"), "{drawn}");
}

#[test]
fn the_completion_screen_draws_the_recovery_key() {
    const KEY: &str = "6f1b4c0d2a9e83f57b6c1d40e2578a93bb0e4f21c7d68a5039e1b74c2f8d605a";
    let options: Vec<Choice> = vec![
        Choice::new(copy::RECOVERY_HEADING, "").heading(),
        Choice::new(copy::write_down(), "").content(),
        Choice::new("", ""),
        Choice::new(KEY, "").content().tinted(),
        Choice::new(copy::NEXT_STEPS_HEADING, "").heading(),
        Choice::new("set secure boot to setup mode", "").content(),
        Choice::new(
            copy::logging(Some(std::path::Path::new("/var/log/bootc-anywhere.log"))),
            "",
        )
        .content(),
        Choice::new("the automatic action explains itself here", "").content(),
        Choice::new(copy::AUTO_WINDOW, "").warning().content(),
        Choice::new("", ""),
        Choice::new(copy::RESTART_AUTO, ""),
        Choice::new(copy::RESTART_MANUAL, ""),
    ];
    for (at, choice) in options.iter().enumerate().take(options.len() - 3) {
        if choice.label.is_empty() {
            continue;
        }
        assert!(!choice.available, "row {at} is landable");
        assert!(!choice.dim, "row {at} is dim");
    }
    assert!(available(&options, options.len() - 2));
    assert!(available(&options, options.len() - 1));

    let at = options.len() - 2;
    let mut state = ListState::default().with_selected(Some(at));
    let mut terminal = Terminal::new(TestBackend::new(
        72,
        u16::try_from(options.len() + 2).unwrap(),
    ))
    .unwrap();
    terminal
        .draw(|frame| {
            draw(
                frame,
                frame.area(),
                copy::INSTALL_DONE,
                &options,
                None,
                copy::DONE_KEYS,
                &mut state,
            )
        })
        .unwrap();
    let drawn = terminal.backend().to_string();
    assert!(drawn.contains(KEY), "the key is not on the screen: {drawn}");
    assert!(drawn.contains("/var/log/bootc-anywhere.log"), "{drawn}");
    assert!(drawn.contains(copy::RESTART_AUTO), "{drawn}");
    assert!(drawn.contains(copy::RESTART_MANUAL), "{drawn}");
    let buffer = terminal.backend().buffer().clone();
    let warn_row = (1..buffer.area.height)
        .find(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, *row)].symbol().to_string())
                .collect::<String>()
                .contains(copy::AUTO_WINDOW)
        })
        .expect("the window is not drawn");
    let lit = (0..buffer.area.width)
        .map(|column| &buffer[(column, warn_row)])
        .find(|cell| !cell.symbol().trim().is_empty())
        .expect("the window has no ink");
    assert_eq!(lit.fg, AMBER, "the window is not the warning colour");

    let buffer = terminal.backend().buffer();
    for heading in [copy::RECOVERY_HEADING, copy::NEXT_STEPS_HEADING] {
        let row = drawn
            .lines()
            .position(|line| line.contains(heading))
            .unwrap_or_else(|| panic!("{heading} is not on the screen: {drawn}"));
        let cell = (0..buffer.area.width)
            .map(|column| &buffer[(column, u16::try_from(row).unwrap())])
            .find(|cell| !cell.symbol().trim().is_empty())
            .unwrap_or_else(|| panic!("{heading} has no ink: {drawn}"));
        assert_eq!(cell.fg, Color::White, "{heading} is not white: {drawn}");
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "{heading} is not bold: {drawn}"
        );
    }

    let row = u16::try_from(
        drawn
            .lines()
            .position(|line| line.contains(KEY))
            .expect("the key's row"),
    )
    .unwrap();
    let lit = (0..buffer.area.width)
        .map(|column| &buffer[(column, row)])
        .filter(|cell| !cell.symbol().trim().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        lit.len(),
        KEY.len(),
        "the key's row is not the key: {drawn}"
    );
    for cell in &lit {
        assert!(
            !cell.modifier.contains(Modifier::DIM),
            "the recovery key is drawn dim: {drawn}"
        );
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "the recovery key is not bold: {drawn}"
        );
    }

    let tint = |class: fn(&char) -> bool| {
        KEY.chars()
            .zip(&lit)
            .filter(|(letter, _)| class(letter))
            .map(|(_, cell)| cell.fg)
            .collect::<std::collections::HashSet<_>>()
    };
    let digits = tint(|letter| letter.is_ascii_digit());
    let letters = tint(|letter| letter.is_alphabetic());
    assert_eq!(digits, std::collections::HashSet::from([Color::White]));
    assert_eq!(letters, std::collections::HashSet::from([HIGHLIGHT]));

    let row = 1 + u16::try_from(at).unwrap();
    let lit = (0..buffer.area.width)
        .map(|column| &buffer[(column, row)])
        .filter(|cell| !cell.symbol().trim().is_empty())
        .collect::<Vec<_>>();
    assert!(!lit.is_empty(), "{drawn}");
    for cell in &lit {
        assert_eq!(cell.fg, HIGHLIGHT, "{drawn}");
    }
}

#[test]
fn the_confirmation_shows_the_answers_and_lights_the_one_enter_takes() {
    let rows = [
        ("disk".to_string(), "/dev/vda".to_string()),
        ("computer name".to_string(), "deb2".to_string()),
        ("password".to_string(), copy::PASSWORD_SET.to_string()),
    ];
    let note = copy::erasing("/dev/vda");
    let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();

    for (button, lit) in [(0, copy::START_INSTALLATION), (1, copy::GO_BACK)] {
        terminal
            .draw(|frame| {
                decide_draw(
                    frame,
                    frame.area(),
                    copy::INSTALLATION_SUMMARY,
                    &note,
                    &[],
                    &rows,
                    copy::READY,
                    copy::START_INSTALLATION,
                    copy::GO_BACK,
                    button,
                )
            })
            .unwrap();
        let drawn = terminal.backend().to_string();
        assert!(drawn.contains(copy::INSTALLATION_SUMMARY), "{drawn}");
        assert!(drawn.contains(copy::READY), "{drawn}");
        assert!(drawn.contains("will be erased. Are you sure?"), "{drawn}");
        assert!(
            drawn.contains("/dev/vda") && drawn.contains("deb2"),
            "{drawn}"
        );
        assert!(drawn.contains(copy::START_INSTALLATION), "{drawn}");
        assert!(drawn.contains(copy::GO_BACK), "{drawn}");

        let buffer = terminal.backend().buffer().clone();
        let row = (0..buffer.area.height)
            .find(|row| {
                let text: String = (0..buffer.area.width)
                    .map(|column| buffer[(column, *row)].symbol().to_string())
                    .collect();
                text.contains(copy::START_INSTALLATION)
            })
            .expect("the answers row is not drawn");
        let taken: String = (0..buffer.area.width)
            .filter(|column| buffer[(*column, row)].modifier.contains(Modifier::REVERSED))
            .map(|column| buffer[(column, row)].symbol().to_string())
            .collect();
        assert_eq!(taken.trim(), lit, "{drawn}");
    }
}

#[test]
fn the_decide_box_sizes_its_note_and_warning_blocks_independently() {
    let rows = [("computer name".to_string(), "deb2".to_string())];
    let note = copy::erasing("/dev/vda");
    let warning = [
        copy::ERASE_WARNING_SECURE_BOOT,
        copy::ERASE_WARNING_UEFI_SETUP,
    ];
    let heading = copy::INSTALLATION_SUMMARY;
    let bare = decide_content_height(heading, "", &[], &rows);
    let note_extra = decide_content_height(heading, &note, &[], &rows) - bare;
    let warning_extra = decide_content_height(heading, "", &warning, &rows) - bare;
    assert_eq!(
        decide_content_height(heading, &note, &warning, &rows),
        bare + note_extra + warning_extra,
        "note and warning must add up rather than interact"
    );
    assert!(note_extra > 0);
    assert!(warning_extra > 0);
}

#[test]
fn a_confirmation_taller_than_the_console_has_no_action() {
    assert!(decide_fits(23, 25));
    assert!(!decide_fits(24, 25));
}

#[test]
fn the_confirmation_shows_a_warning_ahead_of_the_dim_summary() {
    let rows = [("computer name".to_string(), "deb2".to_string())];
    let warning = [copy::ERASE_WARNING_SECURE_BOOT];
    let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
    terminal
        .draw(|frame| {
            decide_draw(
                frame,
                frame.area(),
                copy::INSTALLATION_SUMMARY,
                &copy::erasing("/dev/vda"),
                &warning,
                &rows,
                copy::READY,
                copy::START_INSTALLATION,
                copy::GO_BACK,
                0,
            )
        })
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let warn_row = (0..buffer.area.height)
        .find(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, *row)].symbol().to_string())
                .collect::<String>()
                .contains(copy::ERASE_WARNING_SECURE_BOOT)
        })
        .expect("the warning is not drawn");
    let summary_row = (0..buffer.area.height)
        .find(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, *row)].symbol().to_string())
                .collect::<String>()
                .contains("deb2")
        })
        .expect("the summary row is not drawn");
    assert!(
        warn_row < summary_row,
        "warning must read before the summary"
    );
    assert_eq!(buffer[(0, warn_row)].fg, AMBER);

    let mut plain = Terminal::new(TestBackend::new(70, 12)).unwrap();
    plain
        .draw(|frame| {
            decide_draw(
                frame,
                frame.area(),
                copy::INSTALLATION_SUMMARY,
                &copy::erasing("/dev/vda"),
                &[],
                &rows,
                copy::READY,
                copy::START_INSTALLATION,
                copy::GO_BACK,
                0,
            )
        })
        .unwrap();
    let drawn = plain.backend().to_string();
    assert!(!drawn.contains(copy::ERASE_WARNING_SECURE_BOOT), "{drawn}");
}

#[test]
fn a_confirmation_that_does_not_fit_still_wraps_its_note_and_keeps_its_answers() {
    let one = [("disk".to_string(), "/dev/nvme0n1".to_string())];
    let mut narrow = Terminal::new(TestBackend::new(70, 8)).unwrap();
    narrow
        .draw(|frame| {
            decide_draw(
                frame,
                frame.area(),
                "",
                &copy::changing_partitions("/dev/nvme0n1"),
                &[],
                &one,
                copy::READY,
                copy::START_INSTALLATION,
                copy::GO_BACK,
                0,
            )
        })
        .unwrap();
    let drawn = narrow.backend().to_string();
    assert!(drawn.contains("sure?"), "{drawn}");

    let many: Vec<(String, String)> = (0..20)
        .map(|at| (format!("row {at}"), "value".to_string()))
        .collect();
    let mut short = Terminal::new(TestBackend::new(40, 10)).unwrap();
    short
        .draw(|frame| {
            decide_draw(
                frame,
                frame.area(),
                "",
                "",
                &[],
                &many,
                copy::READY,
                copy::START_INSTALLATION,
                copy::GO_BACK,
                0,
            )
        })
        .unwrap();
    let drawn = short.backend().to_string();
    assert!(drawn.contains(copy::START_INSTALLATION), "{drawn}");
    assert!(drawn.contains(copy::GO_BACK), "{drawn}");
}

fn typed_line(prefix: &str, typed: &str, default: Option<&str>) -> String {
    let mut terminal = Terminal::new(TestBackend::new(60, 3)).unwrap();
    terminal
        .draw(|frame| written(frame, frame.area(), "who owns it", prefix, typed, default))
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn a_default_is_shown_until_something_is_typed_over_it() {
    let empty = typed_line("github.com/", "", Some("someone"));
    assert!(empty.contains("github.com/someone"), "{empty}");
    assert!(empty.contains(LINE_KEYS), "{empty}");

    let over = typed_line("github.com/", "else", Some("someone"));
    assert!(over.contains("github.com/else"), "{over}");
    assert!(!over.contains("someone"), "{over}");

    let bare = typed_line("", "", None);
    assert!(bare.contains("who owns it"), "{bare}");
    let answer = bare.lines().nth(1).unwrap().replace('"', "");
    assert_eq!(answer.trim(), "", "{bare}");
}
