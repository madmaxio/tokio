//! Frontend message serialization.
#![allow(missing_docs)]

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::marker;

use crate::{write_nullable, FromUsize, IsNull, Oid, B};

#[inline]
fn write_body<F, E>(buf: &mut BytesMut, f: F) -> Result<(), E>
where
    F: FnOnce(&mut BytesMut) -> Result<(), E>,
    E: From<io::Error>,
{
    let base = buf.len();
    buf.extend_from_slice(&[0; 4]);

    f(buf)?;

    let size = i32::from_usize(buf.len() - base)?;
    BigEndian::write_i32(&mut buf[base..], size);
    Ok(())
}

pub enum BindError {
    Conversion(Box<dyn Error + marker::Sync + Send>),
    Serialization(io::Error),
}

impl From<Box<dyn Error + marker::Sync + Send>> for BindError {
    #[inline]
    fn from(e: Box<dyn Error + marker::Sync + Send>) -> BindError {
        BindError::Conversion(e)
    }
}

impl From<io::Error> for BindError {
    #[inline]
    fn from(e: io::Error) -> BindError {
        BindError::Serialization(e)
    }
}

#[inline]
pub fn bind<I, J, F, T, K>(
    portal: &str,
    statement: &str,
    formats: I,
    values: J,
    mut serializer: F,
    result_formats: K,
    buf: &mut BytesMut,
) -> Result<(), BindError>
where
    I: IntoIterator<Item = i16>,
    J: IntoIterator<Item = T>,
    F: FnMut(T, &mut BytesMut) -> Result<IsNull, Box<dyn Error + marker::Sync + Send>>,
    K: IntoIterator<Item = i16>,
{
    B(buf).put_u8(b'B');

    write_body(buf, |buf| {
        write_cstr(portal.as_bytes(), buf)?;
        write_cstr(statement.as_bytes(), buf)?;
        write_counted(
            formats,
            |f, buf| {
                B(buf).put_i16_be(f);
                Ok::<_, io::Error>(())
            },
            buf,
        )?;
        write_counted(
            values,
            |v, buf| write_nullable(|buf| serializer(v, buf), buf),
            buf,
        )?;
        write_counted(
            result_formats,
            |f, buf| {
                B(buf).put_i16_be(f);
                Ok::<_, io::Error>(())
            },
            buf,
        )?;

        Ok(())
    })
}

#[inline]
fn write_counted<I, T, F, E>(items: I, mut serializer: F, buf: &mut BytesMut) -> Result<(), E>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T, &mut BytesMut) -> Result<(), E>,
    E: From<io::Error>,
{
    let base = buf.len();
    buf.extend_from_slice(&[0; 2]);
    let mut count = 0;
    for item in items {
        serializer(item, buf)?;
        count += 1;
    }
    let count = i16::from_usize(count)?;
    BigEndian::write_i16(&mut buf[base..], count);

    Ok(())
}

#[inline]
pub fn cancel_request(process_id: i32, secret_key: i32, buf: &mut BytesMut) {
    write_body(buf, |buf| {
        B(buf).put_i32_be(80_877_102);
        B(buf).put_i32_be(process_id);
        B(buf).put_i32_be(secret_key);
        Ok::<_, io::Error>(())
    })
    .unwrap();
}

#[inline]
pub fn close(variant: u8, name: &str, buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'C');
    write_body(buf, |buf| {
        B(buf).put_u8(variant);
        write_cstr(name.as_bytes(), buf)
    })
}

pub struct CopyData<T> {
    buf: T,
    len: i32,
}

impl<T> CopyData<T>
where
    T: Buf,
{
    pub fn new<U>(buf: U) -> io::Result<CopyData<T>>
    where
        U: IntoBuf<Buf = T>,
    {
        let buf = buf.into_buf();

        let len = buf
            .remaining()
            .checked_add(4)
            .and_then(|l| i32::try_from(l).ok())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "message length overflow")
            })?;

        Ok(CopyData { buf, len })
    }

    pub fn write(self, out: &mut BytesMut) {
        B(out).put_u8(b'd');
        B(out).put_i32_be(self.len);
        B(out).put(self.buf);
    }
}

#[inline]
pub fn copy_done(buf: &mut BytesMut) {
    B(buf).put_u8(b'c');
    write_body(buf, |_| Ok::<(), io::Error>(())).unwrap();
}

#[inline]
pub fn copy_fail(message: &str, buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'f');
    write_body(buf, |buf| write_cstr(message.as_bytes(), buf))
}

#[inline]
pub fn describe(variant: u8, name: &str, buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'D');
    write_body(buf, |buf| {
        B(buf).put_u8(variant);
        write_cstr(name.as_bytes(), buf)
    })
}

#[inline]
pub fn execute(portal: &str, max_rows: i32, buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'E');
    write_body(buf, |buf| {
        write_cstr(portal.as_bytes(), buf)?;
        B(buf).put_i32_be(max_rows);
        Ok(())
    })
}

#[inline]
pub fn parse<I>(name: &str, query: &str, param_types: I, buf: &mut BytesMut) -> io::Result<()>
where
    I: IntoIterator<Item = Oid>,
{
    B(buf).put_u8(b'P');
    write_body(buf, |buf| {
        write_cstr(name.as_bytes(), buf)?;
        write_cstr(query.as_bytes(), buf)?;
        write_counted(
            param_types,
            |t, buf| {
                B(buf).put_u32_be(t);
                Ok::<_, io::Error>(())
            },
            buf,
        )?;
        Ok(())
    })
}

#[inline]
pub fn password_message(password: &[u8], buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'p');
    write_body(buf, |buf| write_cstr(password, buf))
}

#[inline]
pub fn query(query: &str, buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'Q');
    write_body(buf, |buf| write_cstr(query.as_bytes(), buf))
}

#[inline]
pub fn sasl_initial_response(mechanism: &str, data: &[u8], buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'p');
    write_body(buf, |buf| {
        write_cstr(mechanism.as_bytes(), buf)?;
        let len = i32::from_usize(data.len())?;
        B(buf).put_i32_be(len);
        B(buf).put_slice(data);
        Ok(())
    })
}

#[inline]
pub fn sasl_response(data: &[u8], buf: &mut BytesMut) -> io::Result<()> {
    B(buf).put_u8(b'p');
    write_body(buf, |buf| {
        B(buf).put_slice(data);
        Ok(())
    })
}

#[inline]
pub fn ssl_request(buf: &mut BytesMut) {
    write_body(buf, |buf| {
        B(buf).put_i32_be(80_877_103);
        Ok::<_, io::Error>(())
    })
    .unwrap();
}

#[inline]
pub fn startup_message<'a, I>(parameters: I, buf: &mut BytesMut) -> io::Result<()>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    write_body(buf, |buf| {
        B(buf).put_i32_be(196_608);
        for (key, value) in parameters {
            write_cstr(key.as_bytes(), buf)?;
            write_cstr(value.as_bytes(), buf)?;
        }
        B(buf).put_u8(0);
        Ok(())
    })
}

#[inline]
pub fn sync(buf: &mut BytesMut) {
    B(buf).put_u8(b'S');
    write_body(buf, |_| Ok::<(), io::Error>(())).unwrap();
}

#[inline]
pub fn terminate(buf: &mut BytesMut) {
    B(buf).put_u8(b'X');
    write_body(buf, |_| Ok::<(), io::Error>(())).unwrap();
}

#[inline]
fn write_cstr(s: &[u8], buf: &mut BytesMut) -> Result<(), io::Error> {
    if s.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "string contains embedded null",
        ));
    }
    B(buf).put_slice(s);
    B(buf).put_u8(0);
    Ok(())
}
