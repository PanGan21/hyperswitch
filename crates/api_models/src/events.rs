pub mod apple_pay_certificates_migration;
pub mod connector_onboarding;
pub mod customer;
pub mod dispute;
pub mod gsm;
mod locker_migration;
pub mod payment;
#[cfg(feature = "payouts")]
pub mod payouts;
#[cfg(feature = "recon")]
pub mod recon;
pub mod refund;
pub mod routing;
pub mod user;
pub mod user_role;

use common_utils::{
    events::{ApiEventMetric, ApiEventsType},
    impl_api_event_type,
};

use crate::customers::CustomerListRequest;
#[allow(unused_imports)]
use crate::{
    admin::*,
    analytics::{
        api_event::*, auth_events::*, connector_events::ConnectorEventsRequest,
        outgoing_webhook_event::OutgoingWebhookLogsRequest, sdk_events::*, search::*, *,
    },
    api_keys::*,
    cards_info::*,
    disputes::*,
    files::*,
    mandates::*,
    organization::{OrganizationId, OrganizationRequest, OrganizationResponse},
    payment_methods::*,
    payments::*,
    user::{UserKeyTransferRequest, UserTransferKeyResponse},
    verifications::*,
};

impl ApiEventMetric for TimeRange {}

impl ApiEventMetric for GetPaymentIntentFiltersRequest {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Analytics)
    }
}

impl ApiEventMetric for GetPaymentIntentMetricRequest {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Analytics)
    }
}

impl ApiEventMetric for PaymentIntentFiltersResponse {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Analytics)
    }
}

impl_api_event_type!(
    Miscellaneous,
    (
        PaymentMethodId,
        PaymentMethodCreate,
        PaymentLinkInitiateRequest,
        RetrievePaymentLinkResponse,
        MandateListConstraints,
        CreateFileResponse,
        MerchantConnectorResponse,
        MerchantConnectorId,
        MandateResponse,
        MandateRevokedResponse,
        RetrievePaymentLinkRequest,
        PaymentLinkListConstraints,
        MandateId,
        DisputeListConstraints,
        RetrieveApiKeyResponse,
        BusinessProfileResponse,
        BusinessProfileUpdate,
        BusinessProfileCreate,
        RevokeApiKeyResponse,
        ToggleKVResponse,
        ToggleKVRequest,
        ToggleAllKVRequest,
        ToggleAllKVResponse,
        MerchantAccountDeleteResponse,
        MerchantAccountUpdate,
        CardInfoResponse,
        CreateApiKeyResponse,
        CreateApiKeyRequest,
        MerchantConnectorDeleteResponse,
        MerchantConnectorUpdate,
        MerchantConnectorCreate,
        MerchantId,
        CardsInfoRequest,
        MerchantAccountResponse,
        MerchantAccountListRequest,
        MerchantAccountCreate,
        PaymentsSessionRequest,
        ApplepayMerchantVerificationRequest,
        ApplepayMerchantResponse,
        ApplepayVerifiedDomainsResponse,
        UpdateApiKeyRequest,
        GetApiEventFiltersRequest,
        ApiEventFiltersResponse,
        GetInfoResponse,
        GetPaymentMetricRequest,
        GetRefundMetricRequest,
        GetActivePaymentsMetricRequest,
        GetSdkEventMetricRequest,
        GetAuthEventMetricRequest,
        GetPaymentFiltersRequest,
        PaymentFiltersResponse,
        GetRefundFilterRequest,
        RefundFiltersResponse,
        GetSdkEventFiltersRequest,
        SdkEventFiltersResponse,
        ApiLogsRequest,
        GetApiEventMetricRequest,
        SdkEventsRequest,
        ReportRequest,
        ConnectorEventsRequest,
        OutgoingWebhookLogsRequest,
        GetGlobalSearchRequest,
        GetSearchRequest,
        GetSearchResponse,
        GetSearchRequestWithIndex,
        GetDisputeFilterRequest,
        DisputeFiltersResponse,
        GetDisputeMetricRequest,
        OrganizationResponse,
        OrganizationRequest,
        OrganizationId,
        CustomerListRequest
    )
);

impl_api_event_type!(
    Keymanager,
    (
        TransferKeyResponse,
        MerchantKeyTransferRequest,
        UserKeyTransferRequest,
        UserTransferKeyResponse
    )
);

impl<T> ApiEventMetric for MetricsResponse<T> {
    fn get_api_event_type(&self) -> Option<ApiEventsType> {
        Some(ApiEventsType::Miscellaneous)
    }
}
