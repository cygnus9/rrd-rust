//! Support for navigating fetched data sets.

use crate::{Timestamp, TimestampExt};
use rrd_sys::rrd_double;
use std::{fmt, ops::Deref, time::Duration};

/// Provides a safe abstraction for traversing the dataset produced by `fetch()`.
///
/// Contains both the data and the metadata (e.g. start, end, step and data sources).
pub struct Data<T> {
    start: Timestamp,
    end: Timestamp,
    step: Duration,
    names: Vec<String>,
    values: T,
    row_count: usize,
}

impl<T> Data<T>
where
    T: Deref<Target = [rrd_double]>,
{
    pub(crate) fn new(
        start: Timestamp,
        end: Timestamp,
        step: Duration,
        names: Vec<String>,
        data: T,
    ) -> Self {
        assert_eq!(data.len() % names.len(), 0);
        let row_count = data.len() / names.len();
        Self {
            start,
            end,
            step,
            names,
            values: data,
            row_count,
        }
    }

    /// Timestamp for the first row of data.
    pub fn start(&self) -> Timestamp {
        self.start
    }

    /// Timestamp for the last row of data.
    pub fn end(&self) -> Timestamp {
        self.end
    }

    /// Time step between rows.
    pub fn step(&self) -> Duration {
        self.step
    }

    /// The number of rows in the dataset.
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// The data source names in the dataset.
    ///
    /// Data sources are conceptually the "columns".
    pub fn ds_names(&self) -> &[String] {
        &self.names
    }

    /// The rows of data in the dataset.
    pub fn rows(&self) -> Rows<'_, T> {
        Rows { data: self }
    }
}

/// An iterator over the [`Row`]s in [`Data`].
pub struct Rows<'data, T> {
    data: &'data Data<T>,
}

impl<'data, T> Rows<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    /// The number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.row_count()
    }

    /// True _iff_ there are 0 rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.row_count() == 0
    }

    /// Iterate over the rows.
    #[must_use]
    pub fn iter(&self) -> RowsIter<'data, T> {
        RowsIter::new(self.data)
    }
}

impl<'data, T> IntoIterator for Rows<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Item = Row<'data, T>;

    type IntoIter = RowsIter<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self.data)
    }
}

impl<'data, T> IntoIterator for &Rows<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Item = Row<'data, T>;

    type IntoIter = RowsIter<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> fmt::Debug for Rows<'_, T>
where
    T: Deref<Target = [rrd_double]> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Iterate over [`Row`]s in [`Data`].
///
/// See [`Rows::iter`].
pub struct RowsIter<'data, T> {
    data: &'data Data<T>,
    max_index: usize,
    next_index: usize,
}

impl<'data, T> RowsIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    fn new(data: &'data Data<T>) -> Self {
        Self {
            data,
            max_index: data.row_count(),
            next_index: 0,
        }
    }
}

impl<'data, T> Iterator for RowsIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Item = Row<'data, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.max_index {
            let index = self.next_index;
            self.next_index += 1;
            Some(Row::new(self.data, index))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.max_index - self.next_index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for RowsIter<'_, T> where T: Deref<Target = [rrd_double]> {}

/// A sequence of values for a particular timestamp.
pub struct Row<'data, T> {
    data: &'data Data<T>,
    data_offset: usize,
    timestamp: Timestamp,
}

impl<'data, T> Row<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    fn new(data: &'data Data<T>, row_index: usize) -> Self {
        Self {
            data,
            data_offset: data.names.len() * row_index,
            timestamp: data.start()
                + data.step() * row_index.try_into().expect("Row index exceeds u32"),
        }
    }

    /// The timestamp for this row of data.
    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// The values for this row, in the order of the data source names in the encompassing [`Data`].
    ///
    /// To access values and DS names together, see [`Self::iter_cells`].
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.data.values.as_ref()[self.data_offset..self.data_offset + self.data.names.len()]
    }

    /// Iterate over the [`Cell`]s for this row's values.
    pub fn iter_cells(&self) -> impl Iterator<Item = Cell<'_>> {
        self.data
            .names
            .iter()
            .zip(self.as_slice())
            .map(|(name, value)| Cell {
                name,
                value: *value,
            })
    }
}

impl<T> Deref for Row<'_, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Target = [rrd_double];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> fmt::Debug for Row<'_, T>
where
    T: Deref<Target = [rrd_double]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct RowDataDebug<'d> {
            row_data: &'d [rrd_double],
        }
        impl fmt::Debug for RowDataDebug<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.row_data.iter()).finish()
            }
        }

        f.debug_struct("Row")
            .field("ts", &self.timestamp)
            .field("ts_int", &self.timestamp.try_as_time_t())
            .field(
                "data",
                &RowDataDebug {
                    row_data: self.as_slice(),
                },
            )
            .finish()
    }
}

/// Contains a value in a [`Row`] along with the corresponding DS name.
#[derive(Debug)]
pub struct Cell<'data> {
    /// The data source name for this value
    pub name: &'data str,
    /// A value in a [`Row`]
    pub value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn rows_iter_size_hint_reports_remaining_rows() {
        let data = Data::new(
            UNIX_EPOCH,
            UNIX_EPOCH + Duration::from_secs(2),
            Duration::from_secs(1),
            vec!["a".to_string(), "b".to_string()],
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        );
        let mut rows = data.rows().iter();

        assert_eq!((3, Some(3)), rows.size_hint());
        assert_eq!(3, rows.len());

        assert!(rows.next().is_some());
        assert_eq!((2, Some(2)), rows.size_hint());
        assert_eq!(2, rows.len());

        assert!(rows.next().is_some());
        assert!(rows.next().is_some());
        assert_eq!((0, Some(0)), rows.size_hint());
        assert_eq!(0, rows.len());
    }
}
