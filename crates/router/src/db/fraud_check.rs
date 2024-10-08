use diesel_models::fraud_check::{self as storage, FraudCheck, FraudCheckUpdate};
use error_stack::report;
use router_env::{instrument, tracing};

use super::MockDb;
use crate::{
    connection,
    core::errors::{self, CustomResult},
    services::Store,
};

#[async_trait::async_trait]
pub trait FraudCheckInterface {
    async fn insert_fraud_check_response(
        &self,
        new: storage::FraudCheckNew,
    ) -> CustomResult<FraudCheck, errors::StorageError>;

    async fn update_fraud_check_response_with_attempt_id(
        &self,
        this: FraudCheck,
        fraud_check: FraudCheckUpdate,
    ) -> CustomResult<FraudCheck, errors::StorageError>;

    async fn find_fraud_check_by_payment_id(
        &self,
        payment_id: common_utils::id_type::PaymentId,
        merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<FraudCheck, errors::StorageError>;

    async fn find_fraud_check_by_payment_id_if_present(
        &self,
        payment_id: common_utils::id_type::PaymentId,
        merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<Option<FraudCheck>, errors::StorageError>;
}

#[async_trait::async_trait]
impl FraudCheckInterface for Store {
    #[instrument(skip_all)]
    async fn insert_fraud_check_response(
        &self,
        new: storage::FraudCheckNew,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        let conn = connection::pg_connection_write(self).await?;
        new.insert(&conn)
            .await
            .map_err(|error| report!(errors::StorageError::from(error)))
    }

    #[instrument(skip_all)]
    async fn update_fraud_check_response_with_attempt_id(
        &self,
        this: FraudCheck,
        fraud_check: FraudCheckUpdate,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        let conn = connection::pg_connection_write(self).await?;
        this.update_with_attempt_id(&conn, fraud_check)
            .await
            .map_err(|error| report!(errors::StorageError::from(error)))
    }

    #[instrument(skip_all)]
    async fn find_fraud_check_by_payment_id(
        &self,
        payment_id: common_utils::id_type::PaymentId,
        merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        let conn = connection::pg_connection_write(self).await?;
        FraudCheck::get_with_payment_id(&conn, payment_id, merchant_id)
            .await
            .map_err(|error| report!(errors::StorageError::from(error)))
    }

    #[instrument(skip_all)]
    async fn find_fraud_check_by_payment_id_if_present(
        &self,
        payment_id: common_utils::id_type::PaymentId,
        merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<Option<FraudCheck>, errors::StorageError> {
        let conn = connection::pg_connection_write(self).await?;
        FraudCheck::get_with_payment_id_if_present(&conn, payment_id, merchant_id)
            .await
            .map_err(|error| report!(errors::StorageError::from(error)))
    }
}

#[async_trait::async_trait]
impl FraudCheckInterface for MockDb {
    async fn insert_fraud_check_response(
        &self,
        _new: storage::FraudCheckNew,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        Err(errors::StorageError::MockDbError)?
    }
    async fn update_fraud_check_response_with_attempt_id(
        &self,
        _this: FraudCheck,
        _fraud_check: FraudCheckUpdate,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        Err(errors::StorageError::MockDbError)?
    }
    async fn find_fraud_check_by_payment_id(
        &self,
        _payment_id: common_utils::id_type::PaymentId,
        _merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<FraudCheck, errors::StorageError> {
        Err(errors::StorageError::MockDbError)?
    }

    async fn find_fraud_check_by_payment_id_if_present(
        &self,
        _payment_id: common_utils::id_type::PaymentId,
        _merchant_id: common_utils::id_type::MerchantId,
    ) -> CustomResult<Option<FraudCheck>, errors::StorageError> {
        Err(errors::StorageError::MockDbError)?
    }
}
