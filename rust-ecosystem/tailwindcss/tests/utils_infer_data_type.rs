//! Smoke tests for `utils/infer_data_type` (upstream has no `.test.ts` for
//! this module — only a `.bench.ts`). Tests cover representative behaviour
//! from upstream comments.

use farmfe_ecosystem_tailwindcss::utils::infer_data_type::{
  infer_data_type, is_angle, is_fraction, is_length, is_number, is_percentage, is_positive_integer,
  is_vector, DataType,
};

#[test]
fn rejects_var_value() {
  assert_eq!(
    infer_data_type("var(--foo)", &[DataType::Color, DataType::Length]),
    None
  );
}

#[test]
fn recognises_color() {
  assert_eq!(
    infer_data_type("#fff", &[DataType::Color]),
    Some(DataType::Color)
  );
  assert_eq!(
    infer_data_type("rgb(0, 0, 0)", &[DataType::Color]),
    Some(DataType::Color)
  );
  assert_eq!(
    infer_data_type("red", &[DataType::Color]),
    Some(DataType::Color)
  );
  assert_eq!(
    infer_data_type("Red", &[DataType::Color]),
    Some(DataType::Color)
  );
  assert_eq!(infer_data_type("notacolor", &[DataType::Color]), None);
}

#[test]
fn recognises_length() {
  assert!(is_length("16px"));
  assert!(is_length("1rem"));
  assert!(is_length("calc(1rem + 2px)"));
  assert!(!is_length("16"));
}

#[test]
fn recognises_percentage_number_fraction() {
  assert!(is_percentage("50%"));
  assert!(!is_percentage("50"));
  assert!(is_number("3.14"));
  assert!(is_number("-3.4e-2"));
  assert!(is_fraction("1/2"));
  assert!(is_fraction("16 / 9"));
}

#[test]
fn recognises_angle_and_vector() {
  assert!(is_angle("90deg"));
  assert!(is_angle("0.25turn"));
  assert!(!is_angle("90"));
  assert!(is_vector("1 2 3"));
  assert!(!is_vector("1 2"));
}

#[test]
fn recognises_positive_integer() {
  assert!(is_positive_integer("0"));
  assert!(is_positive_integer("42"));
  assert!(!is_positive_integer("-1"));
  assert!(!is_positive_integer("1.5"));
  assert!(!is_positive_integer("01"));
}

#[test]
fn type_priority_is_respected() {
  // First matching type wins.
  assert_eq!(
    infer_data_type(
      "16px",
      &[DataType::Color, DataType::Length, DataType::Number]
    ),
    Some(DataType::Length)
  );
}
