use crate::header::*;
use crate::objects::*;

pub struct Writer<'a> {
    index: usize,
    buf: &'a mut [u8],
}

#[derive(Debug, Clone)]
pub enum Error {
    BufferTooSmall(usize),
}

impl<'a> Writer<'a> {
    /// create a new DER writer, with the buffer as the user allocated write buffer
    pub fn new(buf: &'a mut [u8]) -> Self {
        Writer { index: 0, buf }
    }

    fn check_length(&self, sz: usize) -> Result<(), Error> {
        if self.index + sz > self.buf.len() {
            return Err(Error::BufferTooSmall(self.buf.len()));
        }
        Ok(())
    }

    fn identifier(&mut self, identifier: &Identifier) -> Result<(), Error> {
        let sz = identifier.size_bytes();
        self.check_length(sz)?;
        let sz2 = identifier.encode(&mut self.buf[self.index..]);
        assert_eq!(sz, sz2);
        self.index += sz;
        Ok(())
    }

    fn length(&mut self, length: Length) -> Result<(), Error> {
        let sz = length.size_bytes();
        self.check_length(sz)?;
        length.encode(&mut self.buf[self.index..]);
        self.index += sz;
        Ok(())
    }

    fn prim_identifier(&mut self, tag: u32) -> Result<(), Error> {
        let ident = Identifier {
            pc: PC::Primitive,
            class: Class::Universal,
            tag: TagEncoded::new_smallest(tag),
        };
        self.identifier(&ident)
    }

    fn constructed_identifier(&mut self, tag: u32) -> Result<(), Error> {
        let ident = Identifier {
            pc: PC::Constructed,
            class: Class::Universal,
            tag: TagEncoded::new_smallest(tag),
        };
        self.identifier(&ident)
    }

    fn copy_data(&mut self, data: &[u8]) -> Result<(), Error> {
        self.length(Length::new_smallest(data.len()))?;
        self.check_length(data.len())?;
        let end_index = self.index + data.len();
        self.buf[self.index..end_index].copy_from_slice(data);
        self.index += data.len();
        Ok(())
    }

    /// Write a boolean to the DER writer
    pub fn bool(&mut self, b: bool) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_BOOLEAN)?;
        let v = if b { [0xff] } else { [0] };
        self.copy_data(&v)
    }

    /// Write an Integer to the DER writer
    pub fn integer<'b>(&mut self, integer: &'b Integer) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_INTEGER)?;
        self.copy_data(integer.as_ref())
    }

    /// Write an Enumerated to the DER writer
    pub fn enumerated<'b>(&mut self, enumerated: &'b Enumerated) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_ENUMERATED)?;
        self.copy_data(enumerated.as_ref())
    }

    /// Write a bitstring to the DER writer
    pub fn bitstring<'b>(&mut self, obj: &'b BitString) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_BIT_STRING)?;
        self.copy_data(obj.as_ref())
    }

    /// Write a octetstring to the DER writer
    pub fn octetstring<'b>(&mut self, obj: &'b [u8]) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_OCTET_STRING)?;
        self.copy_data(obj.as_ref())
    }

    /// Write a null to the DER writer
    pub fn null(&mut self) -> Result<(), Error> {
        self.prim_identifier(constants::TAG_NULL)?;
        self.copy_data(&[])
    }

    /// Write a utf8 string to the DER writer
    pub fn utf8_string<'b>(&mut self, str: &'b str) -> Result<(), Error> {
        let bytes = str.as_bytes();
        self.prim_identifier(constants::TAG_UTF8_STRING)?;
        self.copy_data(bytes)
    }

    /// Write a sequence to the DER writer
    pub fn sequence<'b, F>(&mut self, f: F) -> Result<(), Error>
    where
        F: Fn(&mut Self) -> Result<(), Error>,
    {
        self.constructed_identifier(constants::TAG_SEQUENCE)?;
        let position_length = self.index;
        self.length(Length::Short(0))?;
        let position_data = self.index;
        f(self)?;
        let diff = self.index - position_data;
        if diff < 0x80 {
            // can reuse the same length bytes position
            let actual_length = Length::Short(diff as u8);
            actual_length.encode(&mut self.buf[position_length..]);
        } else {
            // need to move data by couple of bytes to be able
            // to write the new length
            let actual_length = Length::new_smallest(diff);
            let move_forward = actual_length.size_bytes() - 1;
            self.buf[position_data..].copy_within(0..diff, move_forward);
            self.index += move_forward;
            actual_length.encode(&mut self.buf[position_length..]);
        }

        Ok(())
    }

    /// Get the current position in the Writer, which is also the number of byte written
    pub fn current_position(&self) -> usize {
        self.index
    }

    /// Return the inner sub-slice with a valid DER stream of data
    pub fn finish<'b: 'a>(&'b self) -> &'a [u8] {
        &self.buf[0..self.index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn encode_double_sequence() {
        let mut buf = [0u8; 1024];
        let mut writer = Writer::new(&mut buf);

        let ostring = [2u8; 77];

        writer
            .sequence(|writer| {
                writer.sequence(|writer| writer.octetstring(&ostring))?;
                writer.bool(true)
            })
            .expect("outer sequence");
        let slice = writer.finish();

        let total: u32 = slice.iter().map(|x| *x as u32).sum();
        assert_eq!(slice.len(), 86, "length doesn't match");
        assert_eq!(total, 751, "byte sum doesn't match");
    }
}
