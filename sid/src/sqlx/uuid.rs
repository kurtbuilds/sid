use crate::Sid;
use ::sqlx::encode::IsNull;
/// Encode sid as uuid in the database
use ::sqlx::postgres::{PgArgumentBuffer, PgValueRef, Postgres};
use ::sqlx::types::Uuid;

impl<T> ::sqlx::Encode<'_, Postgres> for Sid<T> {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let uuid = self.uuid();
        <Uuid as sqlx::Encode<Postgres>>::encode(uuid, buf)
    }
}

impl<T> ::sqlx::Decode<'_, Postgres> for Sid<T> {
    fn decode(value: PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let uuid = Uuid::from_slice(value.as_bytes()?)?;
        Ok(Sid::<T>::from(uuid))
    }
}

impl<T> ::sqlx::Type<Postgres> for Sid<T> {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Uuid as sqlx::Type<Postgres>>::type_info()
    }

    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <Uuid as ::sqlx::Type<Postgres>>::compatible(ty)
    }
}
