use diesel::{associations::HasTable, BoolExpressionMethods, ExpressionMethods};

use super::generics;
#[cfg(all(any(feature = "v1", feature = "v2"), not(feature = "payment_v2")))]
use crate::schema::payment_intent::dsl;
#[cfg(all(feature = "v2", feature = "payment_v2"))]
use crate::schema_v2::payment_intent::dsl;
use crate::{
    errors,
    payment_intent::{
        PaymentIntent, PaymentIntentNew, PaymentIntentUpdate, PaymentIntentUpdateInternal,
    },
    PgPooledConn, StorageResult,
};

impl PaymentIntentNew {
    pub async fn insert(self, conn: &PgPooledConn) -> StorageResult<PaymentIntent> {
        generics::generic_insert(conn, self).await
    }
}

impl PaymentIntent {
    pub async fn update(
        self,
        conn: &PgPooledConn,
        payment_intent: PaymentIntentUpdate,
    ) -> StorageResult<Self> {
        match generics::generic_update_with_results::<<Self as HasTable>::Table, _, _, _>(
            conn,
            dsl::payment_id
                .eq(self.payment_id.to_owned())
                .and(dsl::merchant_id.eq(self.merchant_id.to_owned())),
            PaymentIntentUpdateInternal::from(payment_intent),
        )
        .await
        {
            Err(error) => match error.current_context() {
                errors::DatabaseError::NoFieldsToUpdate => Ok(self),
                _ => Err(error),
            },
            Ok(mut payment_intents) => payment_intents
                .pop()
                .ok_or(error_stack::report!(errors::DatabaseError::NotFound)),
        }
    }

    pub async fn find_by_payment_id_merchant_id(
        conn: &PgPooledConn,
        payment_id: &common_utils::id_type::PaymentId,
        merchant_id: &common_utils::id_type::MerchantId,
    ) -> StorageResult<Self> {
        generics::generic_find_one::<<Self as HasTable>::Table, _, _>(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::payment_id.eq(payment_id.to_owned())),
        )
        .await
    }

    pub async fn find_optional_by_payment_id_merchant_id(
        conn: &PgPooledConn,
        payment_id: &common_utils::id_type::PaymentId,
        merchant_id: &common_utils::id_type::MerchantId,
    ) -> StorageResult<Option<Self>> {
        generics::generic_find_one_optional::<<Self as HasTable>::Table, _, _>(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::payment_id.eq(payment_id.to_owned())),
        )
        .await
    }
}
