use crate::Timestamp;
use rrd_sys::rrd_double;
use std::{fmt, ops::Deref, time::Duration};

/// Adds a safe abstraction on top of the result of `rrd::fetch`.
///
/// Object of this type provide access to both the data and the
/// metadata (e.g. start, end, step and data sources).
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
    pub fn new(
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

    pub fn start(&self) -> Timestamp {
        self.start
    }

    pub fn end(&self) -> Timestamp {
        self.end
    }

    pub fn step(&self) -> Duration {
        self.step
    }

    /// Return the number of rows in the dataset.
    ///
    /// # Examples
    /// ```
    /// use std::time::{Duration};
    /// use rrd::data::Data;
    /// use rrd::Timestamp;
    ///
    /// let data = Data::new(
    ///     Timestamp::UNIX_EPOCH,
    ///     Timestamp::UNIX_EPOCH,
    ///     Duration::from_secs(1),
    ///     vec![String::from("ds1"), String::from("ds2")],
    ///     vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
    /// );
    ///
    /// assert_eq!(data.row_count(), 3)
    /// ```
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn sources(&self) -> DataSources<T> {
        DataSources { data: self }
    }

    pub fn rows(&self) -> Rows<T> {
        Rows { data: self }
    }
}

pub struct DataSources<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    data: &'data Data<T>,
}

impl<'data, T> DataSources<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    pub fn len(&self) -> usize {
        self.data.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.names.is_empty()
    }

    pub fn iter(&self) -> DataSourcesIter<'data, T> {
        DataSourcesIter::new(self.data)
    }
}

impl<'data, T> IntoIterator for DataSources<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Item = DataSource<'data, T>;

    type IntoIter = DataSourcesIter<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        DataSourcesIter::new(self.data)
    }
}

pub struct DataSourcesIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    data: &'data Data<T>,
    next_index: usize,
}

impl<'data, T> DataSourcesIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    fn new(data: &'data Data<T>) -> Self {
        Self {
            data,
            next_index: 0,
        }
    }
}

impl<'data, T> Iterator for DataSourcesIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    type Item = DataSource<'data, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.data.names.len() {
            let index = self.next_index;
            self.next_index += 1;
            Some(DataSource {
                data: self.data,
                index,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ds_count = self.data.names.len();
        let remaining = ds_count - self.next_index.min(ds_count);
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for DataSourcesIter<'_, T> where T: Deref<Target = [rrd_double]> {}

/// A data source in [`Data`].
///
/// Conceptually, this is a column defining the order of values in a [`Row`].
pub struct DataSource<'data, T> {
    data: &'data Data<T>,
    index: usize,
}

impl<'data, T> DataSource<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
    pub fn name(&self) -> &'data str {
        &self.data.names[self.index]
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
    pub fn len(&self) -> usize {
        self.data.row_count()
    }

    pub fn is_empty(&self) -> bool {
        self.data.row_count() == 0
    }

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

/// Iteratover over [`Row`]s in [`Data`]
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
    pub fn iter_cells(&self) -> impl Iterator<Item = Cell> {
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

/// Includes the corresponding DS name for each value in a row.
#[derive(Debug)]
pub struct Cell<'data> {
    pub name: &'data str,
    pub value: f64,
}
