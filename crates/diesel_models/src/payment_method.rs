use common_enums::MerchantStorageScheme;
use common_utils::{encryption::Encryption, pii};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
use masking::Secret;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
use crate::{enums as storage_enums, schema::payment_methods};
#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
use crate::{enums as storage_enums, schema_v2::payment_methods};

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
#[derive(
    Clone, Debug, Eq, PartialEq, Identifiable, Queryable, Selectable, Serialize, Deserialize,
)]
#[diesel(table_name = payment_methods, primary_key(payment_method_id), check_for_backend(diesel::pg::Pg))]
pub struct PaymentMethod {
    pub customer_id: common_utils::id_type::CustomerId,
    pub merchant_id: common_utils::id_type::MerchantId,
    pub payment_method_id: String,
    #[diesel(deserialize_as = super::OptionalDieselArray<storage_enums::Currency>)]
    pub accepted_currency: Option<Vec<storage_enums::Currency>>,
    pub scheme: Option<String>,
    pub token: Option<String>,
    pub cardholder_name: Option<Secret<String>>,
    pub issuer_name: Option<String>,
    pub issuer_country: Option<String>,
    #[diesel(deserialize_as = super::OptionalDieselArray<String>)]
    pub payer_country: Option<Vec<String>>,
    pub is_stored: Option<bool>,
    pub swift_code: Option<String>,
    pub direct_debit_token: Option<String>,
    pub created_at: PrimitiveDateTime,
    pub last_modified: PrimitiveDateTime,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub payment_method_issuer: Option<String>,
    pub payment_method_issuer_code: Option<storage_enums::PaymentMethodIssuerCode>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub payment_method_data: Option<Encryption>,
    pub locker_id: Option<String>,
    pub last_used_at: PrimitiveDateTime,
    pub connector_mandate_details: Option<serde_json::Value>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub status: storage_enums::PaymentMethodStatus,
    pub network_transaction_id: Option<String>,
    pub client_secret: Option<String>,
    pub payment_method_billing_address: Option<Encryption>,
    pub updated_by: Option<String>,
    pub version: common_enums::ApiVersion,
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
#[derive(
    Clone, Debug, Eq, PartialEq, Identifiable, Queryable, Selectable, Serialize, Deserialize,
)]
#[diesel(table_name = payment_methods, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct PaymentMethod {
    pub customer_id: common_utils::id_type::CustomerId,
    pub merchant_id: common_utils::id_type::MerchantId,
    pub created_at: PrimitiveDateTime,
    pub last_modified: PrimitiveDateTime,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub payment_method_data: Option<Encryption>,
    pub locker_id: Option<String>,
    pub last_used_at: PrimitiveDateTime,
    pub connector_mandate_details: Option<pii::SecretSerdeValue>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub status: storage_enums::PaymentMethodStatus,
    pub network_transaction_id: Option<String>,
    pub client_secret: Option<String>,
    pub payment_method_billing_address: Option<Encryption>,
    pub updated_by: Option<String>,
    pub locker_fingerprint_id: Option<String>,
    pub id: String,
    pub version: common_enums::ApiVersion,
}

impl PaymentMethod {
    #[cfg(all(
        any(feature = "v1", feature = "v2"),
        not(feature = "payment_methods_v2")
    ))]
    pub fn get_id(&self) -> &String {
        &self.payment_method_id
    }

    #[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
    pub fn get_id(&self) -> &String {
        &self.id
    }
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
#[derive(
    Clone, Debug, Eq, PartialEq, Insertable, router_derive::DebugAsDisplay, Serialize, Deserialize,
)]
#[diesel(table_name = payment_methods)]
pub struct PaymentMethodNew {
    pub customer_id: common_utils::id_type::CustomerId,
    pub merchant_id: common_utils::id_type::MerchantId,
    pub payment_method_id: String,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub payment_method_issuer: Option<String>,
    pub payment_method_issuer_code: Option<storage_enums::PaymentMethodIssuerCode>,
    pub accepted_currency: Option<Vec<storage_enums::Currency>>,
    pub scheme: Option<String>,
    pub token: Option<String>,
    pub cardholder_name: Option<Secret<String>>,
    pub issuer_name: Option<String>,
    pub issuer_country: Option<String>,
    pub payer_country: Option<Vec<String>>,
    pub is_stored: Option<bool>,
    pub swift_code: Option<String>,
    pub direct_debit_token: Option<String>,
    pub created_at: PrimitiveDateTime,
    pub last_modified: PrimitiveDateTime,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub payment_method_data: Option<Encryption>,
    pub locker_id: Option<String>,
    pub last_used_at: PrimitiveDateTime,
    pub connector_mandate_details: Option<serde_json::Value>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub status: storage_enums::PaymentMethodStatus,
    pub network_transaction_id: Option<String>,
    pub client_secret: Option<String>,
    pub payment_method_billing_address: Option<Encryption>,
    pub updated_by: Option<String>,
    pub version: common_enums::ApiVersion,
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
#[derive(
    Clone, Debug, Eq, PartialEq, Insertable, router_derive::DebugAsDisplay, Serialize, Deserialize,
)]
#[diesel(table_name = payment_methods)]
pub struct PaymentMethodNew {
    pub customer_id: common_utils::id_type::CustomerId,
    pub merchant_id: common_utils::id_type::MerchantId,
    pub payment_method: Option<storage_enums::PaymentMethod>,
    pub payment_method_type: Option<storage_enums::PaymentMethodType>,
    pub created_at: PrimitiveDateTime,
    pub last_modified: PrimitiveDateTime,
    pub metadata: Option<pii::SecretSerdeValue>,
    pub payment_method_data: Option<Encryption>,
    pub locker_id: Option<String>,
    pub last_used_at: PrimitiveDateTime,
    pub connector_mandate_details: Option<pii::SecretSerdeValue>,
    pub customer_acceptance: Option<pii::SecretSerdeValue>,
    pub status: storage_enums::PaymentMethodStatus,
    pub network_transaction_id: Option<String>,
    pub client_secret: Option<String>,
    pub payment_method_billing_address: Option<Encryption>,
    pub updated_by: Option<String>,
    pub locker_fingerprint_id: Option<String>,
    pub id: String,
    pub version: common_enums::ApiVersion,
}

impl PaymentMethodNew {
    pub fn update_storage_scheme(&mut self, storage_scheme: MerchantStorageScheme) {
        self.updated_by = Some(storage_scheme.to_string());
    }

    #[cfg(all(
        any(feature = "v1", feature = "v2"),
        not(feature = "payment_methods_v2")
    ))]
    pub fn get_id(&self) -> &String {
        &self.payment_method_id
    }

    #[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
    pub fn get_id(&self) -> &String {
        &self.id
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TokenizeCoreWorkflow {
    pub lookup_key: String,
    pub pm: storage_enums::PaymentMethod,
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentMethodUpdate {
    MetadataUpdateAndLastUsed {
        metadata: Option<serde_json::Value>,
        last_used_at: PrimitiveDateTime,
    },
    UpdatePaymentMethodDataAndLastUsed {
        payment_method_data: Option<Encryption>,
        last_used_at: PrimitiveDateTime,
    },
    PaymentMethodDataUpdate {
        payment_method_data: Option<Encryption>,
    },
    LastUsedUpdate {
        last_used_at: PrimitiveDateTime,
    },
    NetworkTransactionIdAndStatusUpdate {
        network_transaction_id: Option<String>,
        status: Option<storage_enums::PaymentMethodStatus>,
    },
    StatusUpdate {
        status: Option<storage_enums::PaymentMethodStatus>,
    },
    AdditionalDataUpdate {
        payment_method_data: Option<Encryption>,
        status: Option<storage_enums::PaymentMethodStatus>,
        locker_id: Option<String>,
        payment_method: Option<storage_enums::PaymentMethod>,
        payment_method_type: Option<storage_enums::PaymentMethodType>,
        payment_method_issuer: Option<String>,
    },
    ConnectorMandateDetailsUpdate {
        connector_mandate_details: Option<serde_json::Value>,
    },
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentMethodUpdate {
    MetadataUpdateAndLastUsed {
        metadata: Option<pii::SecretSerdeValue>,
        last_used_at: PrimitiveDateTime,
    },
    UpdatePaymentMethodDataAndLastUsed {
        payment_method_data: Option<Encryption>,
        last_used_at: PrimitiveDateTime,
    },
    PaymentMethodDataUpdate {
        payment_method_data: Option<Encryption>,
    },
    LastUsedUpdate {
        last_used_at: PrimitiveDateTime,
    },
    NetworkTransactionIdAndStatusUpdate {
        network_transaction_id: Option<String>,
        status: Option<storage_enums::PaymentMethodStatus>,
    },
    StatusUpdate {
        status: Option<storage_enums::PaymentMethodStatus>,
    },
    AdditionalDataUpdate {
        payment_method_data: Option<Encryption>,
        status: Option<storage_enums::PaymentMethodStatus>,
        locker_id: Option<String>,
        payment_method: Option<storage_enums::PaymentMethod>,
        payment_method_type: Option<storage_enums::PaymentMethodType>,
    },
    ConnectorMandateDetailsUpdate {
        connector_mandate_details: Option<pii::SecretSerdeValue>,
    },
}

impl PaymentMethodUpdate {
    pub fn convert_to_payment_method_update(
        self,
        storage_scheme: MerchantStorageScheme,
    ) -> PaymentMethodUpdateInternal {
        let mut update_internal: PaymentMethodUpdateInternal = self.into();
        update_internal.updated_by = Some(storage_scheme.to_string());
        update_internal
    }
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
#[derive(Clone, Debug, AsChangeset, router_derive::DebugAsDisplay, Serialize, Deserialize)]
#[diesel(table_name = payment_methods)]
pub struct PaymentMethodUpdateInternal {
    metadata: Option<pii::SecretSerdeValue>,
    payment_method_data: Option<Encryption>,
    last_used_at: Option<PrimitiveDateTime>,
    network_transaction_id: Option<String>,
    status: Option<storage_enums::PaymentMethodStatus>,
    locker_id: Option<String>,
    payment_method: Option<storage_enums::PaymentMethod>,
    connector_mandate_details: Option<pii::SecretSerdeValue>,
    updated_by: Option<String>,
    payment_method_type: Option<storage_enums::PaymentMethodType>,
    last_modified: PrimitiveDateTime,
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
impl PaymentMethodUpdateInternal {
    pub fn create_payment_method(self, source: PaymentMethod) -> PaymentMethod {
        let metadata = self.metadata;

        PaymentMethod { metadata, ..source }
    }

    pub fn apply_changeset(self, source: PaymentMethod) -> PaymentMethod {
        let Self {
            metadata,
            payment_method_data,
            last_used_at,
            network_transaction_id,
            status,
            connector_mandate_details,
            updated_by,
            ..
        } = self;

        PaymentMethod {
            metadata: metadata.map_or(source.metadata, Some),
            payment_method_data: payment_method_data.map_or(source.payment_method_data, Some),
            last_used_at: last_used_at.unwrap_or(source.last_used_at),
            network_transaction_id: network_transaction_id
                .map_or(source.network_transaction_id, Some),
            status: status.unwrap_or(source.status),
            connector_mandate_details: connector_mandate_details
                .map_or(source.connector_mandate_details, Some),
            updated_by: updated_by.map_or(source.updated_by, Some),
            last_modified: common_utils::date_time::now(),
            ..source
        }
    }
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
#[derive(Clone, Debug, AsChangeset, router_derive::DebugAsDisplay, Serialize, Deserialize)]
#[diesel(table_name = payment_methods)]
pub struct PaymentMethodUpdateInternal {
    metadata: Option<serde_json::Value>,
    payment_method_data: Option<Encryption>,
    last_used_at: Option<PrimitiveDateTime>,
    network_transaction_id: Option<String>,
    status: Option<storage_enums::PaymentMethodStatus>,
    locker_id: Option<String>,
    payment_method: Option<storage_enums::PaymentMethod>,
    connector_mandate_details: Option<serde_json::Value>,
    updated_by: Option<String>,
    payment_method_type: Option<storage_enums::PaymentMethodType>,
    payment_method_issuer: Option<String>,
    last_modified: PrimitiveDateTime,
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
impl PaymentMethodUpdateInternal {
    pub fn create_payment_method(self, source: PaymentMethod) -> PaymentMethod {
        let metadata = self.metadata.map(Secret::new);

        PaymentMethod { metadata, ..source }
    }

    pub fn apply_changeset(self, source: PaymentMethod) -> PaymentMethod {
        let Self {
            metadata,
            payment_method_data,
            last_used_at,
            network_transaction_id,
            status,
            connector_mandate_details,
            updated_by,
            ..
        } = self;

        PaymentMethod {
            metadata: metadata.map_or(source.metadata, |v| Some(v.into())),
            payment_method_data: payment_method_data.map_or(source.payment_method_data, Some),
            last_used_at: last_used_at.unwrap_or(source.last_used_at),
            network_transaction_id: network_transaction_id
                .map_or(source.network_transaction_id, Some),
            status: status.unwrap_or(source.status),
            connector_mandate_details: connector_mandate_details
                .map_or(source.connector_mandate_details, Some),
            updated_by: updated_by.map_or(source.updated_by, Some),
            last_modified: common_utils::date_time::now(),
            ..source
        }
    }
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
impl From<PaymentMethodUpdate> for PaymentMethodUpdateInternal {
    fn from(payment_method_update: PaymentMethodUpdate) -> Self {
        match payment_method_update {
            PaymentMethodUpdate::MetadataUpdateAndLastUsed {
                metadata,
                last_used_at,
            } => Self {
                metadata,
                payment_method_data: None,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::PaymentMethodDataUpdate {
                payment_method_data,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: None,
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::LastUsedUpdate { last_used_at } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::UpdatePaymentMethodDataAndLastUsed {
                payment_method_data,
                last_used_at,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::NetworkTransactionIdAndStatusUpdate {
                network_transaction_id,
                status,
            } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                network_transaction_id,
                status,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::StatusUpdate { status } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                network_transaction_id: None,
                status,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::AdditionalDataUpdate {
                payment_method_data,
                status,
                locker_id,
                payment_method,
                payment_method_type,
                payment_method_issuer,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: None,
                network_transaction_id: None,
                status,
                locker_id,
                payment_method,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_issuer,
                payment_method_type,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::ConnectorMandateDetailsUpdate {
                connector_mandate_details,
            } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details,
                network_transaction_id: None,
                updated_by: None,
                payment_method_issuer: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
        }
    }
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
impl From<PaymentMethodUpdate> for PaymentMethodUpdateInternal {
    fn from(payment_method_update: PaymentMethodUpdate) -> Self {
        match payment_method_update {
            PaymentMethodUpdate::MetadataUpdateAndLastUsed {
                metadata,
                last_used_at,
            } => Self {
                metadata,
                payment_method_data: None,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::PaymentMethodDataUpdate {
                payment_method_data,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: None,
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::LastUsedUpdate { last_used_at } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::UpdatePaymentMethodDataAndLastUsed {
                payment_method_data,
                last_used_at,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: Some(last_used_at),
                network_transaction_id: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::NetworkTransactionIdAndStatusUpdate {
                network_transaction_id,
                status,
            } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                network_transaction_id,
                status,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::StatusUpdate { status } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                network_transaction_id: None,
                status,
                locker_id: None,
                payment_method: None,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::AdditionalDataUpdate {
                payment_method_data,
                status,
                locker_id,
                payment_method,
                payment_method_type,
            } => Self {
                metadata: None,
                payment_method_data,
                last_used_at: None,
                network_transaction_id: None,
                status,
                locker_id,
                payment_method,
                connector_mandate_details: None,
                updated_by: None,
                payment_method_type,
                last_modified: common_utils::date_time::now(),
            },
            PaymentMethodUpdate::ConnectorMandateDetailsUpdate {
                connector_mandate_details,
            } => Self {
                metadata: None,
                payment_method_data: None,
                last_used_at: None,
                status: None,
                locker_id: None,
                payment_method: None,
                connector_mandate_details,
                network_transaction_id: None,
                updated_by: None,
                payment_method_type: None,
                last_modified: common_utils::date_time::now(),
            },
        }
    }
}

#[cfg(all(
    any(feature = "v1", feature = "v2"),
    not(feature = "payment_methods_v2")
))]
impl From<&PaymentMethodNew> for PaymentMethod {
    fn from(payment_method_new: &PaymentMethodNew) -> Self {
        Self {
            customer_id: payment_method_new.customer_id.clone(),
            merchant_id: payment_method_new.merchant_id.clone(),
            payment_method_id: payment_method_new.payment_method_id.clone(),
            locker_id: payment_method_new.locker_id.clone(),
            accepted_currency: payment_method_new.accepted_currency.clone(),
            scheme: payment_method_new.scheme.clone(),
            token: payment_method_new.token.clone(),
            cardholder_name: payment_method_new.cardholder_name.clone(),
            issuer_name: payment_method_new.issuer_name.clone(),
            issuer_country: payment_method_new.issuer_country.clone(),
            payer_country: payment_method_new.payer_country.clone(),
            is_stored: payment_method_new.is_stored,
            swift_code: payment_method_new.swift_code.clone(),
            direct_debit_token: payment_method_new.direct_debit_token.clone(),
            created_at: payment_method_new.created_at,
            last_modified: payment_method_new.last_modified,
            payment_method: payment_method_new.payment_method,
            payment_method_type: payment_method_new.payment_method_type,
            payment_method_issuer: payment_method_new.payment_method_issuer.clone(),
            payment_method_issuer_code: payment_method_new.payment_method_issuer_code,
            metadata: payment_method_new.metadata.clone(),
            payment_method_data: payment_method_new.payment_method_data.clone(),
            last_used_at: payment_method_new.last_used_at,
            connector_mandate_details: payment_method_new.connector_mandate_details.clone(),
            customer_acceptance: payment_method_new.customer_acceptance.clone(),
            status: payment_method_new.status,
            network_transaction_id: payment_method_new.network_transaction_id.clone(),
            client_secret: payment_method_new.client_secret.clone(),
            updated_by: payment_method_new.updated_by.clone(),
            payment_method_billing_address: payment_method_new
                .payment_method_billing_address
                .clone(),
            version: payment_method_new.version,
        }
    }
}

#[cfg(all(feature = "v2", feature = "payment_methods_v2"))]
impl From<&PaymentMethodNew> for PaymentMethod {
    fn from(payment_method_new: &PaymentMethodNew) -> Self {
        Self {
            customer_id: payment_method_new.customer_id.clone(),
            merchant_id: payment_method_new.merchant_id.clone(),
            locker_id: payment_method_new.locker_id.clone(),
            created_at: payment_method_new.created_at,
            last_modified: payment_method_new.last_modified,
            payment_method: payment_method_new.payment_method,
            payment_method_type: payment_method_new.payment_method_type,
            metadata: payment_method_new.metadata.clone(),
            payment_method_data: payment_method_new.payment_method_data.clone(),
            last_used_at: payment_method_new.last_used_at,
            connector_mandate_details: payment_method_new.connector_mandate_details.clone(),
            customer_acceptance: payment_method_new.customer_acceptance.clone(),
            status: payment_method_new.status,
            network_transaction_id: payment_method_new.network_transaction_id.clone(),
            client_secret: payment_method_new.client_secret.clone(),
            updated_by: payment_method_new.updated_by.clone(),
            payment_method_billing_address: payment_method_new
                .payment_method_billing_address
                .clone(),
            id: payment_method_new.id.clone(),
            locker_fingerprint_id: payment_method_new.locker_fingerprint_id.clone(),
            version: payment_method_new.version,
        }
    }
}
