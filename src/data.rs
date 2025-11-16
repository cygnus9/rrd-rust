//! Support for navigating fetched data sets.

use crate::Timestamp;
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
    data: T,
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
            data,
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
    pub fn len(&self) -> usize {
        self.data.row_count()
    }

    /// True _iff_ there are 0 rows.
    pub fn is_empty(&self) -> bool {
        self.data.row_count() == 0
    }

    /// Iterate over the rows.
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
        (self.max_index, Some(self.max_index))
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
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// The values for this row, in the order of the data source names in the encompassing [`Data`].
    ///
    /// To access values and DS names together, see [`Self::iter_cells`].
    pub fn as_slice(&self) -> &[f64] {
        &self.data.data.as_ref()[self.data_offset..self.data_offset + self.data.names.len()]
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
            .field("ts_int", &self.timestamp.timestamp())
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
