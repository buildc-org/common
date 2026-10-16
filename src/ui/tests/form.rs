use super::*;

#[test]
fn the_form_opens_on_the_field_it_was_given_wherever_that_row_is() {
    assert_eq!(opens_at(&[0, 1, 2, 3], 2), 2);
    assert_eq!(opens_at(&[0, 1, 4, 5], 4), 2);
    assert_eq!(opens_at(&[0, 1, 4, 5], 5), 3);
    assert_eq!(opens_at(&[0, 1, 4, 5], 2), 0);
    assert_eq!(opens_at(&[0], 4), 0);
}

#[test]
fn a_pair_is_flagged_only_once_both_halves_have_been_left() {
    let fields = [
        Field::secret("password", "hunter2"),
        Field::secret("confirm", "hunter3"),
    ];
    let pair = |fields: &[Field]| match fields[0].value() == fields[1].value() {
        true => Vec::new(),
        false => vec![0, 1],
    };
    assert_eq!(alerted(&fields, &[], pair), [false, false]);
    assert_eq!(alerted(&fields, &[0], pair), [false, false]);
    assert_eq!(alerted(&fields, &[0, 1], pair), [true, true]);
    let same = [
        Field::secret("password", "x"),
        Field::secret("confirm", "x"),
    ];
    assert_eq!(alerted(&same, &[0, 1], pair), [false, false]);
}

#[test]
fn the_table_cursor_skips_what_cannot_be_taken() {
    let selectable = [true, false, true];
    assert_eq!(table_step(&selectable, 0, true), Some(2));
    assert_eq!(table_step(&selectable, 2, false), Some(0));
    assert_eq!(table_step(&selectable, 0, false), None);
    assert_eq!(table_step(&selectable, 2, true), None);
    assert_eq!(table_step(&[true], 0, true), None);
    assert_eq!(table_step(&[true], 0, false), None);
    assert_eq!(table_step(&[], 0, true), None);
}
