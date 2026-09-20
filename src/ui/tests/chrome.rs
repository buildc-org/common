use super::*;

#[test]
fn the_installer_box_reads_square_on_the_measured_consoles() {
    let draw = |width: u16, height: u16| {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                chrome(frame, Some("Buildc installer"), "", 12, "");
            })
            .unwrap();
        let drawn = terminal.backend().to_string();
        let rows: Vec<String> = drawn.lines().map(|row| row.replace('"', "")).collect();
        let top = rows
            .iter()
            .position(|row| row.contains('\u{256d}'))
            .unwrap_or_else(|| panic!("{drawn}"));
        let bottom = rows
            .iter()
            .rposition(|row| row.contains('\u{2570}'))
            .unwrap_or_else(|| panic!("{drawn}"));
        let top_row: Vec<char> = rows[top].chars().collect();
        let left = top_row.iter().position(|c| *c == '\u{256d}').unwrap();
        let right = top_row.iter().rposition(|c| *c == '\u{256e}').unwrap();
        (rows, top, bottom, right - left + 1)
    };

    // A terminal cell is twice as tall as it is wide, so a 92-column installer box is square at 46 rows.
    let (rows, top, bottom, width) = draw(160, 50);
    assert_eq!((width, bottom - top + 1), (92, 46));
    assert_eq!(top, 2, "centred on 50 rows");
    assert!(
        rows[..top].iter().all(|row| row.trim().is_empty()),
        "blank rows above a centred box"
    );

    let (rows, top, bottom, width) = draw(80, 25);
    assert_eq!((width, bottom - top + 1), (80, 25));
    assert_eq!(top, 0);
    assert!(rows[bottom + 1..].iter().all(|row| row.trim().is_empty()));
}

#[test]
fn a_read_only_screen_is_set_in_the_middle_of_the_box() {
    let items = [ListItem::new("Recovery Key:"), ListItem::new("a short row")];
    let body = Rect::new(5, 7, 68, 30);
    let placed = set_in(body, &items, 4);
    assert_eq!(
        placed,
        Rect::new(5 + (68 - 15) / 2, 7 + (30 - 4) / 2, 15, 4)
    );
    assert_eq!(set_in(body, &items, 40).height, body.height);
}

#[test]
fn an_overlay_keeps_its_size_whatever_it_holds() {
    for rows in [3u16, 12] {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut inner = Rect::default();
        terminal
            .draw(|frame| {
                inner = in_overlay(40, 10, || {
                    Ok(chrome(frame, Some("Buildc installer"), "", rows, ""))
                })
                .unwrap();
            })
            .unwrap();
        assert_eq!(inner.width, 40 - 2 - 2 * PAD_X, "{rows} rows");
        assert_eq!(inner.height, 10 - 2 - 2 * PAD_Y, "{rows} rows");
    }
}
