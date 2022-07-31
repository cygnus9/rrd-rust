use std::{
    ffi::CStr,
    ops::Deref,
    ptr::null,
    slice,
    time::{Duration, SystemTime},
};

use crate::{sys, util};

pub struct Data {
    pub(crate) start: sys::c_time_t,
    pub(crate) end: sys::c_time_t,
    pub(crate) step: sys::c_ulong,
    pub(crate) ds_count: sys::c_ulong,
    pub(crate) ds_names: *const *const sys::c_char,
    pub(crate) data: *const sys::c_double,
}

impl Data {
    pub fn start(&self) -> SystemTime {
        util::from_unix_time(self.start)
    }

    pub fn end(&self) -> SystemTime {
        util::from_unix_time(self.end)
    }

    pub fn step(&self) -> Duration {
        Duration::from_secs(self.step as u64)
    }

    pub fn row_count(&self) -> usize {
        assert!(self.end >= self.start);
        1 + (self.end - self.start) as usize / self.step as usize
    }

    pub fn sources(&self) -> DataSources {
        DataSources { data: self }
    }

    pub fn rows(&self) -> Rows {
        Rows { data: self }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            step: 0,
            ds_count: 0,
            ds_names: null(),
            data: null(),
        }
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.ds_count as usize {
                sys::rrd_freemem(*self.ds_names.add(i) as *mut sys::c_void);
            }
            sys::rrd_freemem(self.ds_names as *mut sys::c_void);
            sys::rrd_freemem(self.data as *mut sys::c_void);
        }
    }
}

pub struct DataSources<'data> {
    data: &'data Data,
}

impl<'data> DataSources<'data> {
    pub fn len(&self) -> usize {
        self.data.ds_count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> DataSourcesIter<'data> {
        DataSourcesIter::new(self.data)
    }
}

impl<'data> IntoIterator for DataSources<'data> {
    type Item = DataSource<'data>;

    type IntoIter = DataSourcesIter<'data>;

    fn into_iter(self) -> Self::IntoIter {
        DataSourcesIter::new(self.data)
    }
}

pub struct DataSourcesIter<'data> {
    data: &'data Data,
    next_index: usize,
}

impl<'data> DataSourcesIter<'data> {
    fn new(data: &'data Data) -> Self {
        Self {
            data,
            next_index: 0,
        }
    }
}

impl<'data> Iterator for DataSourcesIter<'data> {
    type Item = DataSource<'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.data.ds_count as usize {
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
        let ds_count = self.data.ds_count as usize;
        let remaining = ds_count - self.next_index.min(ds_count);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for DataSourcesIter<'_> {}

pub struct DataSource<'data> {
    data: &'data Data,
    index: usize,
}

impl<'data> DataSource<'data> {
    pub fn name(&self) -> &'data str {
        unsafe {
            let p = self.data.ds_names.add(self.index);
            let s = CStr::from_ptr(*p);
            s.to_str().unwrap()
        }
    }
}

pub struct Rows<'data> {
    data: &'data Data,
}

impl<'data> Rows<'data> {
    pub fn len(&self) -> usize {
        self.data.row_count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> RowsIter<'data> {
        RowsIter::new(self.data)
    }
}

impl<'data> IntoIterator for Rows<'data> {
    type Item = Row<'data>;

    type IntoIter = RowsIter<'data>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self.data)
    }
}

pub struct RowsIter<'data> {
    data: &'data Data,
    names: Vec<&'data str>,
    max_index: usize,
    next_index: usize,
}

impl<'data> RowsIter<'data> {
    fn new(data: &'data Data) -> Self {
        let names: Vec<_> = data.sources().iter().map(|s| s.name()).collect();
        Self {
            data,
            names,
            max_index: data.row_count(),
            next_index: 0,
        }
    }
}

impl<'data, 'iterator> Iterator for RowsIter<'data> {
    type Item = Row<'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.max_index {
            let index = self.next_index;
            self.next_index += 1;
            // Some(Row{data: self.data, names: &self.names, index})
            Some(Row::new(self.data, &self.names, index))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.max_index, Some(self.max_index))
    }
}

impl ExactSizeIterator for RowsIter<'_> {}

pub struct Row<'data> {
    timestamp: SystemTime,
    cells: Vec<Cell<'data>>,
}

impl<'data> Row<'data> {
    fn new(data: &'data Data, names: &[&'data str], index: usize) -> Self {
        let timestamp = data.start() + data.step() * index as u32;
        let offset = data.ds_count as usize * index;
        let values =
            unsafe { slice::from_raw_parts(data.data.add(offset), data.ds_count as usize) };
        assert!(values.len() == names.len());
        let cells = names
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
