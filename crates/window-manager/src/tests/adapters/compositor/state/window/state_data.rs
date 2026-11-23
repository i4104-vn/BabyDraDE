use super::*;

#[test]
fn test_window_state_data_lifecycle() {
    let mut state = WindowStateData::default();
    assert!(!state.is_minimized);
    assert!(state.saved_location.is_none());
    assert!(state.saved_size.is_none());

    // Maximize simulation: save floating location and size
    let initial_loc = Point::<i32, Logical>::from((100, 150));
    let initial_size = Size::<i32, Logical>::from((800, 600));
    state.saved_location = Some(initial_loc);
    state.saved_size = Some(initial_size);

    assert_eq!(state.saved_location, Some(initial_loc));
    assert_eq!(state.saved_size, Some(initial_size));

    // Restore simulation: consume saved geometry
    let restored_loc = state.saved_location.take();
    let restored_size = state.saved_size.take();
    assert_eq!(restored_loc, Some(initial_loc));
    assert_eq!(restored_size, Some(initial_size));
    assert!(state.saved_location.is_none());
    assert!(state.saved_size.is_none());

    // Minimize simulation
    state.is_minimized = true;
    assert!(state.is_minimized);
    state.is_minimized = false;
    assert!(!state.is_minimized);
}
