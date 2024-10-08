use std::{str::FromStr, vec::IntoIter};

use common_utils::{ext_traits::Encode, types::MinorUnit};
use diesel_models::enums as storage_enums;
use error_stack::{report, ResultExt};
use router_env::{
    logger,
    tracing::{self, instrument},
};

use crate::{
    core::{
        errors::{self, RouterResult, StorageErrorExt},
        payments::{
            self,
            flows::{ConstructFlowSpecificData, Feature},
            operations,
        },
    },
    db::StorageInterface,
    routes::{
        self,
        app::{self, ReqState},
        metrics,
    },
    services,
    types::{self, api, domain, storage},
};

#[instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
pub async fn do_gsm_actions<F, ApiRequest, FData>(
    state: &app::SessionState,
    req_state: ReqState,
    payment_data: &mut payments::PaymentData<F>,
    mut connectors: IntoIter<api::ConnectorData>,
    original_connector_data: api::ConnectorData,
    mut router_data: types::RouterData<F, FData, types::PaymentsResponseData>,
    merchant_account: &domain::MerchantAccount,
    key_store: &domain::MerchantKeyStore,
    operation: &operations::BoxedOperation<'_, F, ApiRequest>,
    customer: &Option<domain::Customer>,
    validate_result: &operations::ValidateResult,
    schedule_time: Option<time::PrimitiveDateTime>,
    frm_suggestion: Option<storage_enums::FrmSuggestion>,
    business_profile: &domain::BusinessProfile,
) -> RouterResult<types::RouterData<F, FData, types::PaymentsResponseData>>
where
    F: Clone + Send + Sync,
    FData: Send + Sync,
    payments::PaymentResponse: operations::Operation<F, FData>,

    payments::PaymentData<F>: ConstructFlowSpecificData<F, FData, types::PaymentsResponseData>,
    types::RouterData<F, FData, types::PaymentsResponseData>: Feature<F, FData>,
    dyn api::Connector: services::api::ConnectorIntegration<F, FData, types::PaymentsResponseData>,
{
    let mut retries = None;

    metrics::AUTO_RETRY_ELIGIBLE_REQUEST_COUNT.add(&metrics::CONTEXT, 1, &[]);

    let mut initial_gsm = get_gsm(state, &router_data).await?;

    //Check if step-up to threeDS is possible and merchant has enabled
    let step_up_possible = initial_gsm
        .clone()
        .map(|gsm| gsm.step_up_possible)
        .unwrap_or(false);
    let is_no_three_ds_payment = matches!(
        payment_data.payment_attempt.authentication_type,
        Some(storage_enums::AuthenticationType::NoThreeDs)
    );
    let should_step_up = if step_up_possible && is_no_three_ds_payment {
        is_step_up_enabled_for_merchant_connector(
            state,
            merchant_account.get_id(),
            original_connector_data.connector_name,
        )
        .await
    } else {
        false
    };

    if should_step_up {
        router_data = do_retry(
            &state.clone(),
            req_state.clone(),
            original_connector_data,
            operation,
            customer,
            merchant_account,
            key_store,
            payment_data,
            router_data,
            validate_result,
            schedule_time,
            true,
            frm_suggestion,
            business_profile,
        )
        .await?;
    }
    // Step up is not applicable so proceed with auto retries flow
    else {
        loop {
            // Use initial_gsm for first time alone
            let gsm = match initial_gsm.as_ref() {
                Some(gsm) => Some(gsm.clone()),
                None => get_gsm(state, &router_data).await?,
            };

            match get_gsm_decision(gsm) {
                api_models::gsm::GsmDecision::Retry => {
                    retries = get_retries(state, retries, merchant_account.get_id()).await;

                    if retries.is_none() || retries == Some(0) {
                        metrics::AUTO_RETRY_EXHAUSTED_COUNT.add(&metrics::CONTEXT, 1, &[]);
                        logger::info!("retries exhausted for auto_retry payment");
                        break;
                    }

                    if connectors.len() == 0 {
                        logger::info!("connectors exhausted for auto_retry payment");
                        metrics::AUTO_RETRY_EXHAUSTED_COUNT.add(&metrics::CONTEXT, 1, &[]);
                        break;
                    }

                    let connector = super::get_connector_data(&mut connectors)?;

                    router_data = do_retry(
                        &state.clone(),
                        req_state.clone(),
                        connector,
                        operation,
                        customer,
                        merchant_account,
                        key_store,
                        payment_data,
                        router_data,
                        validate_result,
                        schedule_time,
                        //this is an auto retry payment, but not step-up
                        false,
                        frm_suggestion,
                        business_profile,
                    )
                    .await?;

                    retries = retries.map(|i| i - 1);
                }
                api_models::gsm::GsmDecision::Requeue => {
                    Err(report!(errors::ApiErrorResponse::NotImplemented {
                        message: errors::NotImplementedMessage::Reason(
                            "Requeue not implemented".to_string(),
                        ),
                    }))?
                }
                api_models::gsm::GsmDecision::DoDefault => break,
            }
            initial_gsm = None;
        }
    }
    Ok(router_data)
}

#[instrument(skip_all)]
pub async fn is_step_up_enabled_for_merchant_connector(
    state: &app::SessionState,
    merchant_id: &common_utils::id_type::MerchantId,
    connector_name: types::Connector,
) -> bool {
    let key = merchant_id.get_step_up_enabled_key();
    let db = &*state.store;
    db.find_config_by_key_unwrap_or(key.as_str(), Some("[]".to_string()))
        .await
        .change_context(errors::ApiErrorResponse::InternalServerError)
        .and_then(|step_up_config| {
            serde_json::from_str::<Vec<types::Connector>>(&step_up_config.config)
                .change_context(errors::ApiErrorResponse::InternalServerError)
                .attach_printable("Step-up config parsing failed")
        })
        .map_err(|err| {
            logger::error!(step_up_config_error=?err);
        })
        .ok()
        .map(|connectors_enabled| connectors_enabled.contains(&connector_name))
        .unwrap_or(false)
}

#[instrument(skip_all)]
pub async fn get_retries(
    state: &app::SessionState,
    retries: Option<i32>,
    merchant_id: &common_utils::id_type::MerchantId,
) -> Option<i32> {
    match retries {
        Some(retries) => Some(retries),
        None => {
            let key = merchant_id.get_max_auto_retries_enabled();

            let db = &*state.store;
            db.find_config_by_key(key.as_str())
                .await
                .change_context(errors::ApiErrorResponse::InternalServerError)
                .and_then(|retries_config| {
                    retries_config
                        .config
                        .parse::<i32>()
                        .change_context(errors::ApiErrorResponse::InternalServerError)
                        .attach_printable("Retries config parsing failed")
                })
                .map_err(|err| {
                    logger::error!(retries_error=?err);
                    None::<i32>
                })
                .ok()
        }
    }
}

#[instrument(skip_all)]
pub async fn get_gsm<F, FData>(
    state: &app::SessionState,
    router_data: &types::RouterData<F, FData, types::PaymentsResponseData>,
) -> RouterResult<Option<storage::gsm::GatewayStatusMap>> {
    let error_response = router_data.response.as_ref().err();
    let error_code = error_response.map(|err| err.code.to_owned());
    let error_message = error_response.map(|err| err.message.to_owned());
    let connector_name = router_data.connector.to_string();
    let flow = get_flow_name::<F>()?;
    Ok(
        payments::helpers::get_gsm_record(state, error_code, error_message, connector_name, flow)
            .await,
    )
}

#[instrument(skip_all)]
pub fn get_gsm_decision(
    option_gsm: Option<storage::gsm::GatewayStatusMap>,
) -> api_models::gsm::GsmDecision {
    let option_gsm_decision = option_gsm
            .and_then(|gsm| {
                api_models::gsm::GsmDecision::from_str(gsm.decision.as_str())
                    .map_err(|err| {
                        let api_error = report!(err).change_context(errors::ApiErrorResponse::InternalServerError)
                            .attach_printable("gsm decision parsing failed");
                        logger::warn!(get_gsm_decision_parse_error=?api_error, "error fetching gsm decision");
                        api_error
                    })
                    .ok()
            });

    if option_gsm_decision.is_some() {
        metrics::AUTO_RETRY_GSM_MATCH_COUNT.add(&metrics::CONTEXT, 1, &[]);
    }
    option_gsm_decision.unwrap_or_default()
}

#[inline]
fn get_flow_name<F>() -> RouterResult<String> {
    Ok(std::any::type_name::<F>()
        .to_string()
        .rsplit("::")
        .next()
        .ok_or(errors::ApiErrorResponse::InternalServerError)
        .attach_printable("Flow stringify failed")?
        .to_string())
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all)]
pub async fn do_retry<F, ApiRequest, FData>(
    state: &routes::SessionState,
    req_state: ReqState,
    connector: api::ConnectorData,
    operation: &operations::BoxedOperation<'_, F, ApiRequest>,
    customer: &Option<domain::Customer>,
    merchant_account: &domain::MerchantAccount,
    key_store: &domain::MerchantKeyStore,
    payment_data: &mut payments::PaymentData<F>,
    router_data: types::RouterData<F, FData, types::PaymentsResponseData>,
    validate_result: &operations::ValidateResult,
    schedule_time: Option<time::PrimitiveDateTime>,
    is_step_up: bool,
    frm_suggestion: Option<storage_enums::FrmSuggestion>,
    business_profile: &domain::BusinessProfile,
) -> RouterResult<types::RouterData<F, FData, types::PaymentsResponseData>>
where
    F: Clone + Send + Sync,
    FData: Send + Sync,
    payments::PaymentResponse: operations::Operation<F, FData>,

    payments::PaymentData<F>: ConstructFlowSpecificData<F, FData, types::PaymentsResponseData>,
    types::RouterData<F, FData, types::PaymentsResponseData>: Feature<F, FData>,
    dyn api::Connector: services::api::ConnectorIntegration<F, FData, types::PaymentsResponseData>,
{
    metrics::AUTO_RETRY_PAYMENT_COUNT.add(&metrics::CONTEXT, 1, &[]);

    modify_trackers(
        state,
        connector.connector_name.to_string(),
        payment_data,
        key_store,
        merchant_account.storage_scheme,
        router_data,
        is_step_up,
    )
    .await?;

    let (router_data, _mca) = payments::call_connector_service(
        state,
        req_state,
        merchant_account,
        key_store,
        connector,
        operation,
        payment_data,
        customer,
        payments::CallConnectorAction::Trigger,
        validate_result,
        schedule_time,
        api::HeaderPayload::default(),
        frm_suggestion,
        business_profile,
        true,
    )
    .await?;

    Ok(router_data)
}

#[instrument(skip_all)]
pub async fn modify_trackers<F, FData>(
    state: &routes::SessionState,
    connector: String,
    payment_data: &mut payments::PaymentData<F>,
    key_store: &domain::MerchantKeyStore,
    storage_scheme: storage_enums::MerchantStorageScheme,
    router_data: types::RouterData<F, FData, types::PaymentsResponseData>,
    is_step_up: bool,
) -> RouterResult<()>
where
    F: Clone + Send,
    FData: Send,
{
    let new_attempt_count = payment_data.payment_intent.attempt_count + 1;
    let new_payment_attempt = make_new_payment_attempt(
        connector,
        payment_data.payment_attempt.clone(),
        new_attempt_count,
        is_step_up,
    );

    let db = &*state.store;
    let additional_payment_method_data =
        payments::helpers::update_additional_payment_data_with_connector_response_pm_data(
            payment_data.payment_attempt.payment_method_data.clone(),
            router_data
                .connector_response
                .clone()
                .and_then(|connector_response| connector_response.additional_payment_method_data),
        )?;

    match router_data.response {
        Ok(types::PaymentsResponseData::TransactionResponse {
            resource_id,
            connector_metadata,
            redirection_data,
            charge_id,
            ..
        }) => {
            let encoded_data = payment_data.payment_attempt.encoded_data.clone();

            let authentication_data = redirection_data
                .as_ref()
                .map(Encode::encode_to_value)
                .transpose()
                .change_context(errors::ApiErrorResponse::InternalServerError)
                .attach_printable("Could not parse the connector response")?;

            db.update_payment_attempt_with_attempt_id(
                payment_data.payment_attempt.clone(),
                storage::PaymentAttemptUpdate::ResponseUpdate {
                    status: router_data.status,
                    connector: None,
                    connector_transaction_id: match resource_id {
                        types::ResponseId::NoResponseId => None,
                        types::ResponseId::ConnectorTransactionId(id)
                        | types::ResponseId::EncodedData(id) => Some(id),
                    },
                    connector_response_reference_id: payment_data
                        .payment_attempt
                        .connector_response_reference_id
                        .clone(),
                    authentication_type: None,
                    payment_method_id: payment_data.payment_attempt.payment_method_id.clone(),
                    mandate_id: payment_data
                        .mandate_id
                        .clone()
                        .and_then(|mandate| mandate.mandate_id),
                    connector_metadata,
                    payment_token: None,
                    error_code: None,
                    error_message: None,
                    error_reason: None,
                    amount_capturable: if router_data.status.is_terminal_status() {
                        Some(MinorUnit::new(0))
                    } else {
                        None
                    },
                    updated_by: storage_scheme.to_string(),
                    authentication_data,
                    encoded_data,
                    unified_code: None,
                    unified_message: None,
                    payment_method_data: additional_payment_method_data,
                    charge_id,
                },
                storage_scheme,
            )
            .await
            .to_not_found_response(errors::ApiErrorResponse::PaymentNotFound)?;
        }
        Ok(_) => {
            logger::error!("unexpected response: this response was not expected in Retry flow");
            return Ok(());
        }
        Err(ref error_response) => {
            let option_gsm = get_gsm(state, &router_data).await?;
            let auth_update = if Some(router_data.auth_type)
                != payment_data.payment_attempt.authentication_type
            {
                Some(router_data.auth_type)
            } else {
                None
            };

            db.update_payment_attempt_with_attempt_id(
                payment_data.payment_attempt.clone(),
                storage::PaymentAttemptUpdate::ErrorUpdate {
                    connector: None,
                    error_code: Some(Some(error_response.code.clone())),
                    error_message: Some(Some(error_response.message.clone())),
                    status: storage_enums::AttemptStatus::Failure,
                    error_reason: Some(error_response.reason.clone()),
                    amount_capturable: Some(MinorUnit::new(0)),
                    updated_by: storage_scheme.to_string(),
                    unified_code: option_gsm.clone().map(|gsm| gsm.unified_code),
                    unified_message: option_gsm.map(|gsm| gsm.unified_message),
                    connector_transaction_id: error_response.connector_transaction_id.clone(),
                    payment_method_data: additional_payment_method_data,
                    authentication_type: auth_update,
                },
                storage_scheme,
            )
            .await
            .to_not_found_response(errors::ApiErrorResponse::PaymentNotFound)?;
        }
    }

    let payment_attempt = db
        .insert_payment_attempt(new_payment_attempt, storage_scheme)
        .await
        .to_duplicate_response(errors::ApiErrorResponse::DuplicatePayment {
            payment_id: payment_data.payment_intent.payment_id.clone(),
        })?;

    // update payment_attempt, connector_response and payment_intent in payment_data
    payment_data.payment_attempt = payment_attempt;

    payment_data.payment_intent = db
        .update_payment_intent(
            &state.into(),
            payment_data.payment_intent.clone(),
            storage::PaymentIntentUpdate::PaymentAttemptAndAttemptCountUpdate {
                active_attempt_id: payment_data.payment_attempt.attempt_id.clone(),
                attempt_count: new_attempt_count,
                updated_by: storage_scheme.to_string(),
            },
            key_store,
            storage_scheme,
        )
        .await
        .to_not_found_response(errors::ApiErrorResponse::PaymentNotFound)?;

    Ok(())
}

#[instrument(skip_all)]
pub fn make_new_payment_attempt(
    connector: String,
    old_payment_attempt: storage::PaymentAttempt,
    new_attempt_count: i16,
    is_step_up: bool,
) -> storage::PaymentAttemptNew {
    let created_at @ modified_at @ last_synced = Some(common_utils::date_time::now());
    storage::PaymentAttemptNew {
        connector: Some(connector),
        attempt_id: old_payment_attempt
            .payment_id
            .get_attempt_id(new_attempt_count),
        payment_id: old_payment_attempt.payment_id,
        merchant_id: old_payment_attempt.merchant_id,
        status: old_payment_attempt.status,
        amount: old_payment_attempt.amount,
        currency: old_payment_attempt.currency,
        save_to_locker: old_payment_attempt.save_to_locker,
        offer_amount: old_payment_attempt.offer_amount,
        surcharge_amount: old_payment_attempt.surcharge_amount,
        tax_amount: old_payment_attempt.tax_amount,
        payment_method_id: old_payment_attempt.payment_method_id,
        payment_method: old_payment_attempt.payment_method,
        payment_method_type: old_payment_attempt.payment_method_type,
        capture_method: old_payment_attempt.capture_method,
        capture_on: old_payment_attempt.capture_on,
        confirm: old_payment_attempt.confirm,
        authentication_type: if is_step_up {
            Some(storage_enums::AuthenticationType::ThreeDs)
        } else {
            old_payment_attempt.authentication_type
        },
        amount_to_capture: old_payment_attempt.amount_to_capture,
        mandate_id: old_payment_attempt.mandate_id,
        browser_info: old_payment_attempt.browser_info,
        payment_token: old_payment_attempt.payment_token,
        client_source: old_payment_attempt.client_source,
        client_version: old_payment_attempt.client_version,
        created_at,
        modified_at,
        last_synced,
        net_amount: Default::default(),
        error_message: Default::default(),
        cancellation_reason: Default::default(),
        error_code: Default::default(),
        connector_metadata: Default::default(),
        payment_experience: Default::default(),
        payment_method_data: Default::default(),
        business_sub_label: Default::default(),
        straight_through_algorithm: Default::default(),
        preprocessing_step_id: Default::default(),
        mandate_details: Default::default(),
        error_reason: Default::default(),
        connector_response_reference_id: Default::default(),
        multiple_capture_count: Default::default(),
        amount_capturable: Default::default(),
        updated_by: Default::default(),
        authentication_data: Default::default(),
        encoded_data: Default::default(),
        merchant_connector_id: Default::default(),
        unified_code: Default::default(),
        unified_message: Default::default(),
        external_three_ds_authentication_attempted: Default::default(),
        authentication_connector: Default::default(),
        authentication_id: Default::default(),
        mandate_data: Default::default(),
        payment_method_billing_address_id: Default::default(),
        fingerprint_id: Default::default(),
        charge_id: Default::default(),
        customer_acceptance: Default::default(),
        profile_id: old_payment_attempt.profile_id,
        organization_id: old_payment_attempt.organization_id,
    }
}

pub async fn config_should_call_gsm(
    db: &dyn StorageInterface,
    merchant_id: &common_utils::id_type::MerchantId,
) -> bool {
    let config = db
        .find_config_by_key_unwrap_or(
            &merchant_id.get_should_call_gsm_key(),
            Some("false".to_string()),
        )
        .await;
    match config {
        Ok(conf) => conf.config == "true",
        Err(error) => {
            logger::error!(?error);
            false
        }
    }
}

pub trait GsmValidation<F: Send + Clone + Sync, FData: Send + Sync, Resp> {
    // TODO : move this function to appropriate place later.
    fn should_call_gsm(&self) -> bool;
}

impl<F: Send + Clone + Sync, FData: Send + Sync>
    GsmValidation<F, FData, types::PaymentsResponseData>
    for types::RouterData<F, FData, types::PaymentsResponseData>
{
    #[inline(always)]
    fn should_call_gsm(&self) -> bool {
        if self.response.is_err() {
            true
        } else {
            match self.status {
                storage_enums::AttemptStatus::Started
                | storage_enums::AttemptStatus::AuthenticationPending
                | storage_enums::AttemptStatus::AuthenticationSuccessful
                | storage_enums::AttemptStatus::Authorized
                | storage_enums::AttemptStatus::Charged
                | storage_enums::AttemptStatus::Authorizing
                | storage_enums::AttemptStatus::CodInitiated
                | storage_enums::AttemptStatus::Voided
                | storage_enums::AttemptStatus::VoidInitiated
                | storage_enums::AttemptStatus::CaptureInitiated
                | storage_enums::AttemptStatus::RouterDeclined
                | storage_enums::AttemptStatus::VoidFailed
                | storage_enums::AttemptStatus::AutoRefunded
                | storage_enums::AttemptStatus::CaptureFailed
                | storage_enums::AttemptStatus::PartialCharged
                | storage_enums::AttemptStatus::PartialChargedAndChargeable
                | storage_enums::AttemptStatus::Pending
                | storage_enums::AttemptStatus::PaymentMethodAwaited
                | storage_enums::AttemptStatus::ConfirmationAwaited
                | storage_enums::AttemptStatus::Unresolved
                | storage_enums::AttemptStatus::DeviceDataCollectionPending => false,

                storage_enums::AttemptStatus::AuthenticationFailed
                | storage_enums::AttemptStatus::AuthorizationFailed
                | storage_enums::AttemptStatus::Failure => true,
            }
        }
    }
}
