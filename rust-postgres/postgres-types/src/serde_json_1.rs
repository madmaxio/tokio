use bytes::{BufMut, BytesMut};
use serde_1::{Deserialize, Serialize};
use serde_json_1::Value;
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;

use crate::{FromSql, IsNull, ToSql, Type};

// https://github.com/tokio-rs/bytes/issues/170
struct B<'a>(&'a mut BytesMut);

impl<'a> BufMut for B<'a> {
    #[inline]
    fn remaining_mut(&self) -> usize {
        usize::max_value() - self.0.len()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.0.advance_mut(cnt);
    }

    #[inline]
    unsafe fn bytes_mut(&mut self) -> &mut [u8] {
        if !self.0.has_remaining_mut() {
            self.0.reserve(64);
        }

        self.0.bytes_mut()
    }
}

/// A wrapper type to allow arbitrary `Serialize`/`Deserialize` types to convert to Postgres JSON values.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<'a, T> FromSql<'a> for Json<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Json<T>, Box<dyn Error + Sync + Send>> {
        if *ty == Type::JSONB {
            let mut b = [0; 1];
            raw.read_exact(&mut b)?;
            // We only support version 1 of the jsonb binary format
            if b[0] != 1 {
                return Err("unsupported JSONB encoding version".into());
            }
        }
        serde_json_1::de::from_slice(raw)
            .map(Json)
            .map_err(Into::into)
    }

    accepts!(JSON, JSONB);
}

impl<T> ToSql for Json<T>
where
    T: Serialize + Debug,
{
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if *ty == Type::JSONB {
            B(out).put_u8(1);
        }
        serde_json_1::ser::to_writer(B(out).writer(), &self.0)?;
        Ok(IsNull::No)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}

impl<'a> FromSql<'a> for Value {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Value, Box<dyn Error + Sync + Send>> {
        Json::<Value>::from_sql(ty, raw).map(|json| json.0)
    }

    accepts!(JSON, JSONB);
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Json(self).to_sql(ty, out)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}
