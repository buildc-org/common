use super::*;
use crate::Error;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};

#[test]
fn ctrl_c_interrupts() {
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(pressed(key), Err(Error::Interrupted));
}

#[test]
fn a_plain_c_is_a_key() {
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    assert_eq!(pressed(key), Ok(Some(KeyCode::Char('c'))));
}

#[test]
fn a_released_ctrl_c_does_not_interrupt() {
    let mut key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    key.kind = KeyEventKind::Release;
    assert_eq!(pressed(key), Ok(None));
}

#[test]
fn holding_alt_reveals_until_it_is_released() {
    let mut alt = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT);
    assert_eq!(pressed(alt), Ok(None));
    assert!(revealed());
    alt.kind = KeyEventKind::Release;
    assert_eq!(pressed(alt), Ok(None));
    assert!(!revealed());
}

#[test]
fn an_alt_modifier_key_event_reveals() {
    let modifier = KeyCode::Modifier(ModifierKeyCode::LeftAlt);
    assert_eq!(
        pressed(KeyEvent::new(modifier, KeyModifiers::NONE)),
        Ok(None)
    );
    assert!(revealed());
}

#[test]
fn a_key_without_alt_ends_the_reveal() {
    assert_eq!(
        pressed(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT)),
        Ok(None)
    );
    assert_eq!(
        pressed(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
        Ok(Some(KeyCode::Char('x')))
    );
    assert!(!revealed());
}

#[test]
fn a_repeated_ctrl_c_does_not_interrupt() {
    let mut key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    key.kind = KeyEventKind::Repeat;
    assert_eq!(pressed(key), Ok(None));
}

#[test]
fn releasing_another_key_keeps_the_reveal() {
    assert_eq!(
        pressed(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT)),
        Ok(None)
    );
    let mut other = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    other.kind = KeyEventKind::Release;
    assert_eq!(pressed(other), Ok(None));
    assert!(revealed());
}
