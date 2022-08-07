use std::{
    ops::Deref,
    time::{Duration, SystemTime},
};

use crate::sys;

/// Adds a safe abstraction on top of the result of `rrd::fetch`.
/// 
/// Object of this type provide access to both the data and the
/// metadata (e.g. start, end, step and data sources).
pub struct Data<DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    start: SystemTime,
    end: SystemTime,
    step: Duration,
    names: Vec<String>,
    data: DataType,
    row_count: usize,
}

impl<DataType> Data<DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    pub fn new(
        start: SystemTime,
        end: SystemTime,
        step: Duration,
        names: Vec<String>,
        data: DataType,
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

    pub fn sources(&self) -> DataSources<DataType> {
        DataSources { data: self }
    }

    pub fn rows(&self) -> Rows<DataType> {
        Rows { data: self }
    }
}

pub struct DataSources<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    data: &'data Data<DataType>,
}

impl<'data, DataType> DataSources<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    pub fn len(&self) -> usize {
        self.data.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.names.is_empty()
    }

    pub fn iter(&self) -> DataSourcesIter<'data, DataType> {
        DataSourcesIter::new(self.data)
    }
}

impl<'data, DataType> IntoIterator for DataSources<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    type Item = DataSource<'data, DataType>;

    type IntoIter = DataSourcesIter<'data, DataType>;

    fn into_iter(self) -> Self::IntoIter {
        DataSourcesIter::new(self.data)
    }
}

pub struct DataSourcesIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    data: &'data Data<DataType>,
    next_index: usize,
}

impl<'data, DataType> DataSourcesIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    fn new(data: &'data Data<DataType>) -> Self {
        Self {
            data,
            next_index: 0,
        }
    }
}

impl<'data, DataType> Iterator for DataSourcesIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    type Item = DataSource<'data, DataType>;

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

impl<DataType> ExactSizeIterator for DataSourcesIter<'_, DataType> where
    DataType: Deref<Target = [sys::c_double]>
{
}

pub struct DataSource<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    data: &'data Data<DataType>,
    index: usize,
}

impl<'data, DataType> DataSource<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    pub fn name(&self) -> &'data str {
        &self.data.names[self.index]
    }
}

pub struct Rows<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    data: &'data Data<DataType>,
}

impl<'data, DataType> Rows<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    pub fn len(&self) -> usize {
        self.data.row_count()
    }

    pub fn is_empty(&self) -> bool {
        self.data.row_count() == 0
    }

    pub fn iter(&self) -> RowsIter<'data, DataType> {
        RowsIter::new(self.data)
    }
}

impl<'data, DataType> IntoIterator for Rows<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    type Item = Row<'data>;

    type IntoIter = RowsIter<'data, DataType>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self.data)
    }
}

pub struct RowsIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    data: &'data Data<DataType>,
    max_index: usize,
    next_index: usize,
}

impl<'data, DataType> RowsIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
{
    fn new(data: &'data Data<DataType>) -> Self {
        Self {
            data,
            max_index: data.row_count(),
            next_index: 0,
        }
    }
}

impl<'data, 'iterator, DataType> Iterator for RowsIter<'data, DataType>
where
    DataType: Deref<Target = [sys::c_double]>,
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

impl<DataType> ExactSizeIterator for RowsIter<'_, DataType> where
    DataType: Deref<Target = [sys::c_double]>
{
}

pub struct Row<'data> {
    timestamp: SystemTime,
    cells: Vec<Cell<'data>>,
}

impl<'data> Row<'data> {
    fn new<DataType>(data: &'data Data<DataType>, index: usize) -> Self
    where
        DataType: Deref<Target = [sys::c_double]>,
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
