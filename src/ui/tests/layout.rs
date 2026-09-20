use super::*;

#[test]
fn a_command_that_owns_the_screen_draws_one_box_around_the_form() {
    let fields = [
        Field::pick(
            "disk",
            vec![Choice::new("/dev/vda", "64G  QEMU HARDDISK")],
            None,
        ),
        Field::text("computer name", "deb2"),
        Field::secret("password", ""),
    ];
    let visible: Vec<usize> = (0..fields.len()).collect();
    let (shown, _, _) = laid_out(
        &fields,
        &visible,
        0,
        0,
        &Mode::Rows,
        &[copy::INSTALL, copy::SHUT_DOWN],
        Some("still needs a disk, a password"),
        true,
        &[],
        &[],
    );
    let mut terminal = Terminal::new(TestBackend::new(64, 16)).unwrap();
    terminal
        .draw(|frame| {
            let area = chrome(
                frame,
                Some("Buildc installer"),
                "",
                shown.len() as u16,
                copy::INSTALL_KEYS,
            );
            sheet_of(frame, area, &shown, "", 0, 0)
        })
        .unwrap();
    let drawn = terminal.backend().to_string();
    let rows: Vec<&str> = drawn.lines().collect();
    assert!(rows[0].contains('\u{256d}'), "{drawn}");
    let top = rows
        .iter()
        .position(|row| row.contains("Buildc installer"))
        .unwrap_or_else(|| panic!("{drawn}"));
    assert!(rows[top].contains('\u{256d}'), "{drawn}");
    assert!(drawn.contains("?  disk"), "{drawn}");
    assert!(drawn.contains(copy::INSTALL), "{drawn}");
    assert!(drawn.contains(copy::SHUT_DOWN), "{drawn}");
    assert!(drawn.contains("still needs a disk, a password"), "{drawn}");
    assert_eq!(drawn.matches(copy::INSTALL_KEYS).count(), 1, "{drawn}");
    let last = rows
        .iter()
        .rposition(|row| row.contains('\u{2570}'))
        .unwrap();
    assert!(rows[last].contains(copy::INSTALL_KEYS), "{drawn}");
}

#[test]
fn sections_are_drawn_above_the_rows() {
    let fields = [
        Field::text("computer name", "deb2"),
        Field::text("disk", ""),
    ];
    let header = vec![
        HeaderLine::Section("OS Image".to_string()),
        HeaderLine::Row("name: next44".to_string()),
    ];
    let sections = [(0usize, "OS setup"), (1usize, "Disk setup")];
    let (lines, focus, _) = laid_out(
        &fields,
        &[0, 1],
        1,
        0,
        &Mode::Rows,
        &[copy::INSTALL],
        None,
        false,
        &header,
        &sections,
    );
    let text = |line: &Line<'static>| {
        line.spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>()
    };
    assert_eq!(text(&lines[0]), "  OS Image");
    assert_eq!(text(&lines[1]), "     name: next44");
    assert_eq!(text(&lines[2]), "", "one blank under the panel");
    assert_eq!(text(&lines[3]), "  OS setup");
    assert!(
        text(&lines[4]).contains("computer name"),
        "{:?}",
        text(&lines[4])
    );
    assert_eq!(text(&lines[5]), "", "a blank separates the sections");
    assert_eq!(text(&lines[6]), "  Disk setup");
    assert!(text(&lines[7]).contains("disk"), "{:?}", text(&lines[7]));
    assert_eq!(text(&lines[8]), "");
    assert!(text(&lines[9]).contains(copy::INSTALL));
    assert_eq!(focus, 7);
    let title = lines[0].spans[0].style;
    assert_eq!(title.fg, Some(Color::White), "{title:?}");
    assert!(title.add_modifier.contains(Modifier::BOLD), "{title:?}");
    let row = lines[1].spans[0].style;
    assert!(row.add_modifier.contains(Modifier::DIM), "{row:?}");
}

#[test]
fn the_required_statements_wrap_and_keep_the_blank_between_them() {
    let statement = copy::required_uki_db();
    let header: Vec<HeaderLine> = statement
        .iter()
        .map(|row| HeaderLine::Row(row.to_string()))
        .collect();
    let fields = [Field::text("computer name", "deb2")];
    let (lines, _, _) = laid_out(
        &fields,
        &[0],
        0,
        0,
        &Mode::Rows,
        &[],
        None,
        false,
        &header,
        &[],
    );
    let text = |line: &Line<'static>| {
        line.spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>()
    };
    let drawn: Vec<String> = lines.iter().map(text).collect();
    let blank = drawn
        .iter()
        .position(|line| line.trim().is_empty())
        .expect("the blank between the statements");
    assert!(blank > 1, "the first statement wrapped: {drawn:?}");
    let joined = |part: &[String]| {
        part.iter()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join(" ")
    };
    assert_eq!(joined(&drawn[..blank]), statement[0]);
    let after = blank + 1;
    let end = drawn[after..]
        .iter()
        .position(|line| line.trim().is_empty())
        .expect("the blank under the panel");
    assert_eq!(joined(&drawn[after..after + end]), statement[2]);
    assert!(
        drawn.iter().all(|line| line.chars().count() <= PANEL_ROOM),
        "{drawn:?}"
    );
    let dim = lines[0].spans[0].style;
    assert!(dim.add_modifier.contains(Modifier::DIM), "{dim:?}");
}

#[test]
fn the_longest_panel_fits_the_fallback_console() {
    let mut header = vec![HeaderLine::Section("OS Image".to_string())];
    header.extend((0..3).map(|n| HeaderLine::Row(format!("image row {n}"))));
    header.push(HeaderLine::Section("Detected System Firmware".to_string()));
    header.extend((0..2).map(|n| HeaderLine::Row(format!("firmware row {n}"))));
    header.push(HeaderLine::Section("Required".to_string()));
    header.extend((0..8).map(|n| HeaderLine::Row(format!("required row {n}"))));
    let fields = [
        Field::text("computer name", "next44"),
        Field::text("username", ""),
        Field::secret("password", ""),
        Field::secret("confirm", ""),
        Field::pick("disk", vec![Choice::new("/dev/vda", "")], None),
    ];
    let visible = [0, 1, 2, 3, 4];
    let (lines, focus, _) = laid_out(
        &fields,
        &visible,
        0,
        0,
        &Mode::Rows,
        &["Install"],
        None,
        false,
        &header,
        &[(0, "OS setup")],
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 25)).unwrap();
    terminal
        .draw(|frame| {
            let area = chrome(
                frame,
                Some("Buildc installer"),
                "",
                lines.len() as u16,
                "",
            );
            sheet_of(frame, area, &lines, "", focus, 0);
        })
        .unwrap();
    let drawn = terminal.backend().to_string();
    assert!(drawn.contains("OS Image"), "{drawn}");
    assert!(drawn.contains("required row 7"), "{drawn}");
    assert!(drawn.contains("computer name"), "{drawn}");
}

#[test]
fn the_chosen_action_is_the_highlight_colour() {
    let fields = [Field::text("disk", "")];
    let (chosen, _, _) = laid_out(
        &fields,
        &[0],
        1,
        0,
        &Mode::Rows,
        &["Install", "Shut down"],
        None,
        false,
        &[],
        &[],
    );
    let span = |line: &Line<'static>| {
        line.spans
            .iter()
            .find(|span| span.content.contains("Install"))
            .unwrap()
            .style
    };
    let style = span(chosen.last().expect("the actions row"));
    assert_eq!(style.fg, Some(HIGHLIGHT), "{style:?}");
    assert!(style.add_modifier.contains(Modifier::REVERSED), "{style:?}");

    let (blocked, _, _) = laid_out(
        &fields,
        &[0],
        1,
        0,
        &Mode::Rows,
        &["Install"],
        Some("still needs a disk"),
        true,
        &[],
        &[],
    );
    let buttons = blocked
        .iter()
        .find(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("Install"))
        })
        .expect("the actions row");
    let style = span(buttons);
    assert!(style.add_modifier.contains(Modifier::DIM), "{style:?}");
    assert_eq!(
        blocked.iter().rev().nth(1).map(|line| line.width()),
        Some(0),
        "{blocked:?}"
    );
    assert!(
        blocked.last().is_some_and(|line| line
            .spans
            .iter()
            .any(|span| span.content.contains("still needs a disk"))),
        "{blocked:?}"
    );
}

#[test]
fn the_blocked_reason_is_drawn_only_once_the_action_is_tried() {
    let fields = [Field::text("computer name", "")];
    let quiet = laid_out(
        &fields,
        &[0],
        1,
        0,
        &Mode::Rows,
        &["Install"],
        Some("still needs a computer name"),
        false,
        &[],
        &[],
    )
    .0;
    assert!(
        !quiet.iter().any(|line| line
            .spans
            .iter()
            .any(|span| span.content.contains("still needs"))),
        "{quiet:?}"
    );
    assert!(
        quiet.last().is_some_and(|line| line
            .spans
            .iter()
            .any(|span| span.content.contains("Install"))),
        "{quiet:?}"
    );
}

#[test]
fn an_actionless_window_draws_the_reason_it_refused_with() {
    const WHY: &str = "the two answers do not agree";
    let fields = [
        Field::secret("passphrase", "a"),
        Field::secret("confirm", "b"),
    ];
    let text = |lines: &[Line<'static>]| -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect()
            })
            .collect()
    };
    let quiet = laid_out(
        &fields,
        &[0, 1],
        0,
        0,
        &Mode::Rows,
        &[],
        Some(WHY),
        false,
        &[],
        &[],
    );
    assert!(
        !text(&quiet.0).iter().any(|line| line.contains(WHY)),
        "{:?}",
        quiet.0
    );
    assert_eq!(
        quiet.2,
        quiet.0.len(),
        "a form without actions pads under its reason"
    );
    let said = laid_out(
        &fields,
        &[0, 1],
        1,
        0,
        &Mode::Rows,
        &[],
        Some(WHY),
        true,
        &[],
        &[],
    );
    let drawn = text(&said.0);
    assert_eq!(
        drawn.len(),
        4,
        "two fields, the blank, and the reason: {:?}",
        drawn
    );
    assert!(drawn[2].is_empty(), "one blank under the field: {drawn:?}");
    assert!(drawn[3].contains(WHY), "{drawn:?}");
    assert_eq!(said.1, 3, "the reason is what stays in view");
    assert_eq!(
        said.2,
        said.0.len(),
        "the padding goes below the reason, not between it and the field"
    );
    assert!(drawn[3].starts_with("     "), "{:?}", drawn[3]);
}

#[test]
fn a_measure_takes_only_digits_and_reads_with_one_decimal() {
    let mut fields = [Field::measure("size", "20", "GB")];
    fields[0].push('x');
    fields[0].push('.');
    fields[0].push('5');
    assert_eq!(fields[0].value(), "205", "a letter is not a size");
    fields[0].pop();
    assert_eq!(fields[0].shown(), "20.0");
    let (lines, _, _) = laid_out(&fields, &[0], 0, 0, &Mode::Rows, &[], None, false, &[], &[]);
    let text: String = lines[0]
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    assert!(text.contains("20.0 GB"), "{text}");
    assert!(
        lines[0].spans[4].style.add_modifier.contains(Modifier::DIM),
        "the unit is drawn apart from the value: {:?}",
        lines[0].spans[4].style
    );
}

#[test]
fn a_radio_group_is_drawn_and_its_dot_is_the_answer() {
    let fields = [
        Field::radio(
            "type",
            vec![Choice::new("none", ""), Choice::new("passphrase", "")],
            Some(1),
        ),
        Field::text("name", ""),
    ];
    let text = |lines: &[Line<'static>]| -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect()
            })
            .collect()
    };
    let away = laid_out(
        &fields,
        &[0, 1],
        1,
        0,
        &Mode::Rows,
        &[],
        None,
        false,
        &[],
        &[],
    );
    let away = text(&away.0);
    let answer = away
        .iter()
        .find(|line| line.contains("\u{25c9} passphrase"))
        .expect("the dotted answer");
    assert!(!answer.contains("> "), "{answer}");
    assert!(away.iter().any(|line| line.contains("\u{25cb} none")));
    let drawn = laid_out(
        &fields,
        &[0, 1],
        0,
        0,
        &Mode::Open(0),
        &[],
        None,
        false,
        &[],
        &[],
    );
    let on_it = text(&drawn.0);
    assert!(!on_it[0].contains("passphrase"), "{}", on_it[0]);
    assert!(!on_it[0].contains("none"), "{}", on_it[0]);
    let label = drawn.0[0]
        .spans
        .iter()
        .find(|span| span.content.contains("type"))
        .expect("the row's label");
    assert_ne!(label.style.fg, Some(HIGHLIGHT), "{:?}", label.style);
    assert!(!on_it[0].contains("> "), "{}", on_it[0]);
    let answer = on_it
        .iter()
        .find(|line| line.contains("\u{25c9} passphrase"))
        .expect("the dotted answer");
    assert!(!answer.contains("> "), "{answer}");
    let cursor = on_it
        .iter()
        .find(|line| line.contains("\u{25cb} none"))
        .expect("the cursor's option");
    assert!(cursor.contains("> "), "{cursor}");
    let elsewhere = [
        Field::fixed("shown", "value"),
        Field::radio(
            "type",
            vec![Choice::new("none", ""), Choice::new("passphrase", "")],
            Some(1),
        ),
    ];
    let open = laid_out(
        &elsewhere,
        &[0, 1],
        1,
        0,
        &Mode::Open(0),
        &[],
        None,
        false,
        &[],
        &[],
    );
    let dim = open.0[0]
        .spans
        .iter()
        .find(|span| span.content.contains("shown"))
        .expect("the fixed row's label");
    assert!(dim.style.add_modifier.contains(Modifier::DIM), "{dim:?}");
}

#[test]
fn a_hidden_option_is_not_drawn_or_landed_on() {
    let options = vec![
        Choice::new("no TPM", "No TPM available")
            .unavailable()
            .hidden(),
        Choice::new("passphrase", ""),
        Choice::new("no initramfs", "No LUKS support")
            .unavailable()
            .hidden(),
    ];
    let fields = [Field::radio("encryption", options, Some(0))];
    assert_eq!(
        fields[0].value(),
        "no TPM",
        "the held answer keeps its index"
    );
    assert!(matches!(opened(&fields[0]), Mode::Open(1)));
    let (lines, _, _) = laid_out(
        &fields,
        &[0],
        0,
        0,
        &Mode::Open(1),
        &[],
        None,
        false,
        &[],
        &[],
    );
    let drawn: String = lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .map(|span| span.content.as_ref())
        .collect();
    assert!(drawn.contains("passphrase"), "{drawn}");
    assert!(!drawn.contains("no TPM"), "{drawn}");
    assert!(!drawn.contains("no initramfs"), "{drawn}");
    assert_eq!(
        option_step(
            match &fields[0] {
                Field::Pick { options, .. } => options,
                _ => unreachable!(),
            },
            1,
            false
        ),
        None
    );
    assert_eq!(
        option_step(
            match &fields[0] {
                Field::Pick { options, .. } => options,
                _ => unreachable!(),
            },
            1,
            true
        ),
        None
    );

    let none = [Field::radio(
        "encryption",
        vec![Choice::new("no TPM", "No TPM available")
            .unavailable()
            .hidden()],
        Some(0),
    )];
    let (lines, _, _) = laid_out(&none, &[0], 0, 0, &Mode::Rows, &[], None, false, &[], &[]);
    assert!(lines.iter().any(|line| line
        .spans
        .iter()
        .any(|span| span.content.contains(NO_CHOICES))));
}

#[test]
fn a_flagged_pair_is_drawn_red_on_its_values() {
    let fields = [
        Field::Secret {
            label: "password".to_string(),
            value: "hunter2".to_string(),
            alert: true,
        },
        Field::Secret {
            label: "confirm".to_string(),
            value: "hunter3".to_string(),
            alert: true,
        },
        Field::text("computer name", "deb2"),
    ];
    let (lines, _, _) = laid_out(
        &fields,
        &[0, 1, 2],
        2,
        0,
        &Mode::Rows,
        &[],
        None,
        false,
        &[],
        &[],
    );
    for at in 0..2 {
        let value = lines[at].spans.last().expect("a value span");
        assert_eq!(value.style.fg, Some(ALERT), "{:?}", lines[at]);
        let label = lines[at]
            .spans
            .iter()
            .find(|span| span.content.contains("password") || span.content.contains("confirm"))
            .expect("the row's label");
        assert_ne!(label.style.fg, Some(ALERT), "{:?}", label.style);
    }
    let plain = lines[2].spans.last().expect("a value span");
    assert_ne!(plain.style.fg, Some(ALERT), "a row nobody flagged");
}

#[test]
fn a_long_option_detail_wraps_to_its_own_line() {
    let reason = copy::OPENED_KEYFILE_PLAIN;
    let fields = [Field::pick(
        copy::ROW_ENCRYPTION,
        vec![Choice::new(copy::OPENED_ADD_KEY, reason).unavailable()],
        Some(0),
    )];
    let (lines, _, _) = laid_out(
        &fields,
        &[0],
        0,
        0,
        &Mode::Open(0),
        &[],
        None,
        false,
        &[],
        &[],
    );
    let text: Vec<String> = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect()
        })
        .collect();
    assert!(
        text.iter().any(|line| line.contains(reason)),
        "the reason is not on a line of its own: {text:?}"
    );
}

#[test]
fn a_step_lands_on_the_next_question_without_opening_it() {
    let fields = [
        Field::pick(
            "layout",
            vec![Choice::new("whole", ""), Choice::new("manual", "")],
            Some(0),
        ),
        Field::gap(),
        Field::text("name", ""),
    ];
    let visible = [0, 1, 2];
    let mut cursor = 0;
    assert!(matches!(onward(&fields, &visible, &mut cursor), Mode::Rows));
    assert_eq!(cursor, 2, "the gap is not a stop");
    let mut cursor = 2;
    assert!(matches!(
        backward(&fields, &visible, &mut cursor),
        Mode::Rows
    ));
    assert_eq!(cursor, 0);
    let (lines, _, _) = laid_out(
        &fields,
        &visible,
        0,
        0,
        &Mode::Rows,
        &[],
        None,
        false,
        &[],
        &[],
    );
    let text = |line: &Line<'static>| {
        line.spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>()
    };
    assert_eq!(text(&lines[1]), "", "{:?}", lines);
    assert!(text(&lines[2]).contains("name"), "{:?}", lines);
}

#[test]
fn a_table_field_draws_its_headings_its_rows_and_the_hot_one() {
    let rows = vec![
        vec![
            Cell::set("/dev/vda"),
            Cell::new("68 GB"),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
        ],
        vec![
            Cell::new("\u{2514}\u{2500} root"),
            Cell::new("the rest"),
            Cell::new("ext4"),
            Cell::new(""),
            Cell::new(""),
            Cell::new("/"),
        ],
    ];
    let fields = [
        Field::text("computer name", "deb2"),
        Field::table(
            "disk",
            "disk",
            &["size", "filesystem", "format", "type", "mount"],
            rows.clone(),
            vec![true, false],
            vec![Vec::new(), Vec::new()],
            0,
            false,
        ),
    ];
    let resting = laid_out(
        &fields,
        &[0, 1],
        1,
        0,
        &Mode::Rows,
        &[copy::INSTALL],
        None,
        false,
        &[],
        &[],
    )
    .0;
    let cold = resting[3]
        .spans
        .iter()
        .find(|span| span.content.contains("/dev/vda"))
        .expect("the disk row");
    assert_ne!(cold.style.fg, Some(HIGHLIGHT), "{:?}", cold.style);
    let away = laid_out(
        &fields,
        &[0, 1],
        0,
        0,
        &Mode::Rows,
        &[copy::INSTALL],
        None,
        false,
        &[],
        &[],
    )
    .0;
    let cold_label = away[1]
        .spans
        .iter()
        .find(|span| span.content.contains("disk"))
        .expect("the table's label");
    assert_ne!(
        cold_label.style.fg,
        Some(HIGHLIGHT),
        "{:?}",
        cold_label.style
    );
    let hot_label = away[0]
        .spans
        .iter()
        .find(|span| span.content.contains("computer name"))
        .expect("the field above's label");
    assert_eq!(hot_label.style.fg, Some(HIGHLIGHT), "{:?}", hot_label.style);
    let (lines, focus, _) = laid_out(
        &fields,
        &[0, 1],
        1,
        0,
        &Mode::Table,
        &[copy::INSTALL],
        None,
        false,
        &[],
        &[],
    );
    let text = |line: &Line<'static>| {
        line.spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>()
    };
    assert!(text(&lines[1]).contains("disk"), "{:?}", text(&lines[1]));
    assert!(text(&lines[2]).contains("size"), "{:?}", text(&lines[2]));
    assert!(
        text(&lines[3]).contains("/dev/vda"),
        "{:?}",
        text(&lines[3])
    );
    assert!(text(&lines[4]).contains("root"), "{:?}", text(&lines[4]));
    assert_eq!(focus, 3);
    let preview = lines[4]
        .spans
        .iter()
        .find(|span| span.content.contains("root"))
        .expect("the preview row");
    assert!(
        preview.style.add_modifier.contains(Modifier::DIM),
        "{:?}",
        preview.style
    );
    let hot = lines[3]
        .spans
        .iter()
        .find(|span| span.content.contains("/dev/vda"))
        .expect("the hot row");
    assert_eq!(hot.style.fg, Some(HIGHLIGHT), "{:?}", hot.style);
}

#[test]
fn a_table_with_long_node_names_still_draws_its_last_columns() {
    let names = [
        "\u{25c9} QEMU HARDDISK (/dev/vda)",
        "\u{251c}\u{2500} esp (/boot/efi)",
        "\u{2514}\u{2500} home (/var/home)",
    ];
    let rows: Vec<Vec<Cell>> = names
        .iter()
        .map(|name| {
            vec![
                Cell::new(name),
                Cell::new("68.7 GB"),
                Cell::new("ext4"),
                Cell::new(""),
                Cell::new(""),
                Cell::new("/var/home"),
            ]
        })
        .collect();
    let fields = [Field::table(
        "disk selection",
        "disk",
        &copy::layout_headings(),
        rows,
        vec![true; names.len()],
        vec![Vec::new(); names.len()],
        0,
        true,
    )];
    let (lines, _, _) = laid_out(&fields, &[0], 0, 0, &Mode::Rows, &[], None, false, &[], &[]);
    let text = |line: &Line<'static>| -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    };
    let head = text(&lines[1]);
    assert!(head.contains("type"), "{head:?}");
    assert!(head.contains("mount"), "{head:?}");
    assert!(head.chars().count() <= PANEL_ROOM, "{head:?}");
    for line in &lines[2..] {
        let row = text(line);
        assert!(row.chars().count() <= PANEL_ROOM, "{row:?}");
    }
}

#[test]
fn a_form_shows_every_field_and_opens_a_list_where_it_stands() {
    let disks = vec![
        Choice::new("/dev/sda", "512G  Samsung SSD"),
        Choice::new("/dev/sdb", "1T  WD  removable"),
    ];
    let fields = [
        Field::pick("disk", disks, None),
        Field::text("computer name", "debian-bootc"),
        Field::text("username", ""),
        Field::secret("password", "hunter2"),
        Field::secret("confirm", "hunter2"),
        Field::pick(
            "encryption",
            vec![Choice::new("none", "not encrypted")],
            Some(0),
        ),
    ];
    let visible: Vec<usize> = (0..fields.len()).collect();
    let (shown, _, _) = laid_out(
        &fields,
        &visible,
        0,
        0,
        &Mode::Open(1),
        &["Install", "Shut down"],
        Some("still needs a disk, a username"),
        false,
        &[],
        &[],
    );
    let mut terminal = Terminal::new(TestBackend::new(64, shown.len() as u16 + 1)).unwrap();
    terminal
        .draw(|frame| sheet_of(frame, frame.area(), &shown, copy::INSTALL_KEYS, 0, 0))
        .unwrap();
    let drawn = terminal.backend().to_string();
    eprintln!("SHOT\n{drawn}SHOT");
    let rows: Vec<&str> = drawn.lines().collect();
    assert!(rows[0].contains("?  disk"), "{drawn}");
    assert!(!rows[0].contains("> ?  disk"), "{drawn}");
    assert!(rows[1].contains("/dev/sda"), "{drawn}");
    assert!(rows[2].contains("> "), "{drawn}");
    assert!(rows[3].contains("computer name"), "{drawn}");
    assert!(
        drawn.contains("*******") && !drawn.contains("hunter2"),
        "{drawn}"
    );
    assert!(drawn.contains('?'), "{drawn}");
    assert!(
        drawn.contains("Install") && drawn.contains("Shut down"),
        "{drawn}"
    );
    assert!(!drawn.contains("still needs a disk, a username"), "{drawn}");
}
