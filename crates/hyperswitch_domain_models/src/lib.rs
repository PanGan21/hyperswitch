pub mod api;
pub mod behaviour;
pub mod business_profile;
pub mod consts;
pub mod customer;
pub mod errors;
pub mod mandates;
pub mod merchant_account;
pub mod merchant_connector_account;
pub mod merchant_key_store;
pub mod payment_address;
pub mod payment_method_data;
pub mod payment_methods;
pub mod payments;
#[cfg(feature = "payouts")]
pub mod payouts;
pub mod router_data;
pub mod router_data_v2;
pub mod router_flow_types;
pub mod router_request_types;
pub mod router_response_types;
pub mod type_encryption;
pub mod types;

#[cfg(not(feature = "payouts"))]
pub trait PayoutAttemptInterface {}

#[cfg(not(feature = "payouts"))]
pub trait PayoutsInterface {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum RemoteStorageObject<T: ForeignIDRef> {
    ForeignID(String),
    Object(T),
}

impl<T: ForeignIDRef> From<T> for RemoteStorageObject<T> {
    fn from(value: T) -> Self {
        Self::Object(value)
    }
}

pub trait ForeignIDRef {
    fn foreign_id(&self) -> String;
}

impl<T: ForeignIDRef> RemoteStorageObject<T> {
    pub fn get_id(&self) -> String {
        match self {
            Self::ForeignID(id) => id.clone(),
            Self::Object(i) => i.foreign_id(),
        }
    }
}
