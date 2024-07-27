use std::{
    ops::Deref,
    time::{Duration, SystemTime},
};

use rrd_sys::rrd_double;

/// Adds a safe abstraction on top of the result of `rrd::fetch`.
///
/// Object of this type provide access to both the data and the
/// metadata (e.g. start, end, step and data sources).
pub struct Data<T>
where
    T: Deref<Target = [rrd_double]>,
{
    start: SystemTime,
    end: SystemTime,
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
        start: SystemTime,
        end: SystemTime,
        step: Duration,
        names: Vec<String>,
        data: T,
    ) -> Self {
        assert!(data.len() % names.len() == 0);
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

    pub fn start(&self) -> SystemTime {
        self.start
    }

    pub fn end(&self) -> SystemTime {
        self.end
    }

    pub fn step(&self) -> Duration {
        self.step
    }

    /// Return the number of rows in the dataset.
    ///
    /// # Examples
    /// ```
    /// use std::time::{Duration, SystemTime};
    /// use rrd::data::Data;
    ///
    /// let data = Data::new(
    ///     SystemTime::UNIX_EPOCH,
    ///     SystemTime::UNIX_EPOCH,
    ///     Duration::from_secs(1),
    ///     vec![String::from("ds1"), String::from("ds2")],
    ///     vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
    /// );
    ///
    /// assert!(data.row_count() == 3)
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

pub struct DataSource<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
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

pub struct Rows<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
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
    type Item = Row<'data>;

    type IntoIter = RowsIter<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self.data)
    }
}

pub struct RowsIter<'data, T>
where
    T: Deref<Target = [rrd_double]>,
{
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
    type Item = Row<'data>;

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

pub struct Row<'data> {
    timestamp: SystemTime,
    cells: Vec<Cell<'data>>,
}

impl<'data> Row<'data> {
    fn new<T>(data: &'data Data<T>, index: usize) -> Self
    where
        T: Deref<Target = [rrd_double]>,
    {
        let timestamp = data.start() + data.step() * index as u32;
        let offset = data.names.len() * index;
        let values = &data.data.as_ref()[offset..offset + data.names.len()];
        assert!(values.len() == data.names.len());
        let cells = data
            .names
            .iter()
            .zip(values)
            .map(|(name, value)| Cell {
                name,
                value: *value,
            })
            .collect();
        Self { timestamp, cells }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl<'data> Deref for Row<'data> {
    type Target = [Cell<'data>];

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

pub struct Cell<'data> {
    pub name: &'data str,
    pub value: f64,
}
