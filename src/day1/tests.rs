use super::*;

#[test]
fn fcu() {
    assert_eq!(fuel_counter_upper(12), 2);
    assert_eq!(fuel_counter_upper(14), 2);
    assert_eq!(fuel_counter_upper(1969), 654);
    assert_eq!(fuel_counter_upper(100756), 33583);
}

#[test]
fn test_tyranny() {
    assert_eq!(tyrannical_fcu(14), 2);
    assert_eq!(tyrannical_fcu(1969), 966);
    assert_eq!(tyrannical_fcu(100756), 50346);
}
