pub trait SectionReader {
    type Item;
    type Error;
    fn read(&mut self) -> Result<Self::Item, Self::Error>;
    fn get_count(&self) -> u32;
}

pub struct SectionItemIterator<R>
    where R: SectionReader
{
    reader: R,
    error: bool,
    remaining_items: u32,
}

impl<R> SectionItemIterator<R>
    where R: SectionReader
{
    pub fn new(reader: R) -> SectionItemIterator<R> {
        let remaining_items = reader.get_count();
        SectionItemIterator { reader, error: false, remaining_items }
    }
}

impl<R> Iterator for SectionItemIterator<R>
    where R: SectionReader
{
    type Item = Result<R::Item, R::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_items == 0 || self.error {
            None
            //TODO:Ensure that no bytes are left over
        // } else if self.error {
        //     None
        } else {
            let result = self.reader.read();
            self.error = result.is_err();
            self.remaining_items -= 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.reader.get_count() as usize;
        (count, Some(count))
    }
}