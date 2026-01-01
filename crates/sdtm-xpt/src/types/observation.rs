//! Observation (row) type for streaming XPT operations.
//!
//! An observation represents a single row of data in an XPT dataset.
//! This type is designed for streaming read/write operations where
//! loading all data into memory is not desirable.

use super::XptValue;

/// A single observation (row) in an XPT dataset.
///
/// This type is designed for streaming operations, representing one row
/// of data that can be processed incrementally without loading the entire
/// dataset into memory.
///
/// # Example
///
/// ```
/// use sdtm_xpt::{Observation, XptValue};
///
/// let obs = Observation::new(vec![
///     XptValue::character("SUBJ001"),
///     XptValue::numeric(42.0),
/// ]);
///
/// assert_eq!(obs.len(), 2);
/// assert_eq!(obs.get(0).unwrap().as_str(), Some("SUBJ001"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    values: Vec<XptValue>,
}

impl Observation {
    /// Create a new observation from a vector of values.
    #[must_use]
    pub fn new(values: Vec<XptValue>) -> Self {
        Self { values }
    }

    /// Create an empty observation.
    #[must_use]
    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    /// Create an observation with capacity for the specified number of values.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    /// Get a value by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&XptValue> {
        self.values.get(index)
    }

    /// Get a mutable value by index.
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut XptValue> {
        self.values.get_mut(index)
    }

    /// Get the number of values in the observation.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the observation is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Add a value to the observation.
    pub fn push(&mut self, value: XptValue) {
        self.values.push(value);
    }

    /// Iterate over the values.
    pub fn iter(&self) -> impl Iterator<Item = &XptValue> {
        self.values.iter()
    }

    /// Iterate over mutable values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut XptValue> {
        self.values.iter_mut()
    }

    /// Get the values as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[XptValue] {
        &self.values
    }

    /// Consume the observation and return the values.
    #[must_use]
    pub fn into_values(self) -> Vec<XptValue> {
        self.values
    }

    /// Get the values as a mutable slice.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [XptValue] {
        &mut self.values
    }

    /// Convert from a Vec<XptValue> (consuming).
    #[must_use]
    pub fn from_vec(values: Vec<XptValue>) -> Self {
        Self::new(values)
    }
}

impl Default for Observation {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Vec<XptValue>> for Observation {
    fn from(values: Vec<XptValue>) -> Self {
        Self::new(values)
    }
}

impl FromIterator<XptValue> for Observation {
    fn from_iter<I: IntoIterator<Item = XptValue>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for Observation {
    type Item = XptValue;
    type IntoIter = std::vec::IntoIter<XptValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a Observation {
    type Item = &'a XptValue;
    type IntoIter = std::slice::Iter<'a, XptValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl std::ops::Index<usize> for Observation {
    type Output = XptValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl std::ops::IndexMut<usize> for Observation {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_new() {
        let obs = Observation::new(vec![
            XptValue::character("test"),
            XptValue::numeric(42.0),
        ]);
        assert_eq!(obs.len(), 2);
        assert!(!obs.is_empty());
    }

    #[test]
    fn test_observation_empty() {
        let obs = Observation::empty();
        assert_eq!(obs.len(), 0);
        assert!(obs.is_empty());
    }

    #[test]
    fn test_observation_get() {
        let obs = Observation::new(vec![
            XptValue::character("a"),
            XptValue::numeric(1.0),
        ]);
        assert_eq!(obs.get(0).unwrap().as_str(), Some("a"));
        assert_eq!(obs.get(1).unwrap().as_f64(), Some(1.0));
        assert!(obs.get(2).is_none());
    }

    #[test]
    fn test_observation_push() {
        let mut obs = Observation::empty();
        obs.push(XptValue::numeric(1.0));
        obs.push(XptValue::character("x"));
        assert_eq!(obs.len(), 2);
    }

    #[test]
    fn test_observation_iter() {
        let obs = Observation::new(vec![
            XptValue::numeric(1.0),
            XptValue::numeric(2.0),
        ]);
        let sum: f64 = obs.iter().filter_map(|v| v.as_f64()).sum();
        assert_eq!(sum, 3.0);
    }

    #[test]
    fn test_observation_into_values() {
        let obs = Observation::new(vec![XptValue::numeric(1.0)]);
        let values = obs.into_values();
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_observation_from_vec() {
        let values = vec![XptValue::character("test")];
        let obs: Observation = values.into();
        assert_eq!(obs.len(), 1);
    }

    #[test]
    fn test_observation_from_iter() {
        let obs: Observation = (0..3).map(|i| XptValue::numeric(i as f64)).collect();
        assert_eq!(obs.len(), 3);
    }

    #[test]
    fn test_observation_index() {
        let obs = Observation::new(vec![
            XptValue::numeric(1.0),
            XptValue::numeric(2.0),
        ]);
        assert_eq!(obs[0].as_f64(), Some(1.0));
        assert_eq!(obs[1].as_f64(), Some(2.0));
    }

    #[test]
    fn test_observation_index_mut() {
        let mut obs = Observation::new(vec![XptValue::numeric(1.0)]);
        obs[0] = XptValue::numeric(99.0);
        assert_eq!(obs[0].as_f64(), Some(99.0));
    }

    #[test]
    fn test_observation_into_iter() {
        let obs = Observation::new(vec![
            XptValue::numeric(1.0),
            XptValue::numeric(2.0),
        ]);
        let collected: Vec<_> = obs.into_iter().collect();
        assert_eq!(collected.len(), 2);
    }
}
