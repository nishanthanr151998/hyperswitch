#[cfg(feature = "payouts")]
use api_models::payouts::PayoutMethodData;
#[cfg(feature = "payouts")]
use common_utils::pii::Email;
#[cfg(feature = "payouts")]
use masking::Secret;
use serde::Deserialize;
#[cfg(feature = "payouts")]
use serde::Serialize;

type Error = error_stack::Report<errors::ConnectorError>;

#[cfg(feature = "payouts")]
use crate::{
    connector::utils::RouterData,
    types::{
        storage::enums::{self as storage_enums, EntityType},
        transformers::ForeignFrom,
    },
};
use crate::{core::errors, types};

pub struct WiseAuthType {
    pub(super) api_key: String,
    #[allow(dead_code)]
    pub(super) profile_id: String,
}

impl TryFrom<&types::ConnectorAuthType> for WiseAuthType {
    type Error = Error;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            types::ConnectorAuthType::BodyKey { api_key, key1 } => Ok(Self {
                api_key: api_key.to_string(),
                profile_id: key1.to_string(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType)?,
        }
    }
}

// Wise error response
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub timestamp: Option<String>,
    pub errors: Option<Vec<SubError>>,
    pub status: Option<i8>,
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub message: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubError {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub field: Option<String>,
    pub arguments: Option<Vec<String>>,
}

// Payouts
#[cfg(feature = "payouts")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseRecipientCreateRequest {
    currency: String,
    #[serde(rename = "type")]
    recipient_type: RecipientType,
    profile: String,
    account_holder_name: Secret<String>,
    details: WiseBankDetails,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum RecipientType {
    Iban,
    SortCode,
    SwiftCode,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseBankDetails {
    legal_type: LegalType,
    address: WiseAddressDetails,
    post_code: Option<String>,
    nationality: Option<String>,
    account_holder_name: Option<Secret<String>>,
    email: Option<Email>,
    account_number: Option<String>,
    city: Option<String>,
    sort_code: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
    transit_number: Option<String>,
    routing_number: Option<String>,
    abartn: Option<String>,
    swift_code: Option<String>,
    payin_reference: Option<String>,
    psp_reference: Option<String>,
    tax_id: Option<String>,
    order_id: Option<String>,
    job: Option<String>,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LegalType {
    Business,
    #[default]
    Private,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseAddressDetails {
    country: Option<storage_enums::CountryAlpha2>,
    country_code: Option<storage_enums::CountryAlpha2>,
    first_line: Option<Secret<String>>,
    post_code: Option<Secret<String>>,
    city: Option<String>,
    state: Option<Secret<String>>,
}

#[allow(dead_code)]
#[cfg(feature = "payouts")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseRecipientCreateResponse {
    id: i64,
    business: Option<i64>,
    profile: Option<i64>,
    account_holder_name: Secret<String>,
    currency: String,
    country: String,
    #[serde(rename = "type")]
    request_type: String,
    details: WiseBankDetails,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WisePayoutQuoteRequest {
    source_currency: String,
    target_currency: String,
    source_amount: Option<i64>,
    target_amount: Option<i64>,
    pay_out: WisePayOutOption,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WisePayOutOption {
    Balance,
    #[default]
    BankTransfer,
    Swift,
    SwiftOur,
    Interac,
}

#[allow(dead_code)]
#[cfg(feature = "payouts")]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WisePayoutQuoteResponse {
    source_amount: f64,
    client_id: String,
    id: String,
    status: WiseStatus,
    profile: i64,
    rate: i8,
    source_currency: String,
    target_currency: String,
    user: i64,
    rate_type: WiseRateType,
    pay_out: WisePayOutOption,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WiseRateType {
    #[default]
    Fixed,
    Floating,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WisePayoutCreateRequest {
    target_account: i64,
    quote_uuid: String,
    customer_transaction_id: String,
    details: WiseTransferDetails,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseTransferDetails {
    transfer_purpose: Option<String>,
    source_of_funds: Option<String>,
    transfer_purpose_sub_transfer_purpose: Option<String>,
}

#[allow(dead_code)]
#[cfg(feature = "payouts")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WisePayoutCreateResponse {
    id: i64,
    user: i64,
    target_account: i64,
    source_account: Option<i64>,
    quote_uuid: String,
    status: WiseStatus,
    reference: String,
    rate: f32,
    business: Option<i64>,
    details: WiseTransferDetails,
    has_active_issues: bool,
    source_currency: String,
    source_value: f64,
    target_currency: String,
    target_value: f64,
    customer_transaction_id: String,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WisePayoutFulfillRequest {
    #[serde(rename = "type")]
    fund_type: FundType,
}

// NOTE - Only balance is allowed as time of incorporating this field - https://api-docs.transferwise.com/api-reference/transfer#fund
#[cfg(feature = "payouts")]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FundType {
    #[default]
    Balance,
}

#[allow(dead_code)]
#[cfg(feature = "payouts")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseFulfillResponse {
    status: WiseStatus,
    error_code: Option<String>,
    error_message: Option<String>,
    balance_transaction_id: Option<i64>,
}

#[cfg(feature = "payouts")]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WiseStatus {
    Completed,
    Pending,
    Rejected,

    #[serde(rename = "processing")]
    #[default]
    Processing,

    #[serde(rename = "incoming_payment_waiting")]
    IncomingPaymentWaiting,
}

#[cfg(feature = "payouts")]
fn get_payout_address_details(
    address: &Option<api_models::payments::Address>,
) -> Option<WiseAddressDetails> {
    address.as_ref().and_then(|add| {
        add.address.as_ref().map(|a| WiseAddressDetails {
            country: a.country,
            country_code: a.country,
            first_line: a.line1.clone(),
            post_code: a.zip.clone(),
            city: a.city.clone(),
            state: a.state.clone(),
        })
    })
}

#[cfg(feature = "payouts")]
fn get_payout_bank_details(
    payout_method_data: PayoutMethodData,
    address: &Option<api_models::payments::Address>,
    entity_type: EntityType,
) -> Result<WiseBankDetails, errors::ConnectorError> {
    let wise_address_details = match get_payout_address_details(address) {
        Some(a) => Ok(a),
        None => Err(errors::ConnectorError::MissingRequiredField {
            field_name: "address",
        }),
    }?;
    match payout_method_data {
        PayoutMethodData::Bank(b) => Ok(WiseBankDetails {
            legal_type: LegalType::foreign_from(entity_type),
            address: wise_address_details,
            account_number: b.bank_account_number.to_owned(),
            sort_code: b.bank_sort_code.to_owned(),
            routing_number: b.bank_routing_number.to_owned(),
            iban: b.iban.to_owned(),
            bic: b.bic.to_owned(),
            transit_number: b.bank_transit_number,
            ..WiseBankDetails::default()
        }),
        _ => Err(errors::ConnectorError::NotSupported {
            message: "Card payout creation is not supported".to_string(),
            connector: "Wise",
            payment_experience: "".to_string(),
        }),
    }
}

// Payouts recipient create request transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<&types::PayoutsRouterData<F>> for WiseRecipientCreateRequest {
    type Error = Error;
    fn try_from(item: &types::PayoutsRouterData<F>) -> Result<Self, Self::Error> {
        let request = item.request.to_owned();
        let customer_details = request.customer_details;
        let payout_method_data = item.get_payout_method_data()?;
        let bank_details = get_payout_bank_details(
            payout_method_data,
            &item.address.billing,
            item.request.entity_type,
        )?;
        let source_id = match item.connector_auth_type.to_owned() {
            types::ConnectorAuthType::BodyKey { api_key: _, key1 } => Ok(key1),
            _ => Err(errors::ConnectorError::MissingRequiredField {
                field_name: "source_id for PayoutRecipient creation",
            }),
        }?;
        match request.payout_type.to_owned() {
            storage_enums::PayoutType::Card => Err(errors::ConnectorError::NotSupported {
                message: "Card payout creation is not supported".to_string(),
                connector: "Wise",
                payment_experience: "".to_string(),
            })?,
            storage_enums::PayoutType::Bank => {
                let account_holder_name = customer_details
                    .ok_or(errors::ConnectorError::MissingRequiredField {
                        field_name: "customer_details for PayoutRecipient creation",
                    })?
                    .name
                    .ok_or(errors::ConnectorError::MissingRequiredField {
                        field_name: "customer_details.name for PayoutRecipient creation",
                    })?;
                Ok(Self {
                    profile: source_id,
                    currency: request.destination_currency.to_string(),
                    recipient_type: RecipientType::SortCode, // TODO: Map it to BankType (added in future commit)
                    account_holder_name,
                    details: bank_details,
                })
            }
        }
    }
}

// Payouts recipient fulfill response transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<types::PayoutsResponseRouterData<F, WiseRecipientCreateResponse>>
    for types::PayoutsRouterData<F>
{
    type Error = Error;
    fn try_from(
        item: types::PayoutsResponseRouterData<F, WiseRecipientCreateResponse>,
    ) -> Result<Self, Self::Error> {
        let response: WiseRecipientCreateResponse = item.response;

        Ok(Self {
            response: Ok(types::PayoutsResponseData {
                status: Some(storage_enums::PayoutStatus::RequiresCreation),
                connector_payout_id: response.id.to_string(),
                payout_eligible: None,
            }),
            ..item.data
        })
    }
}

// Payouts quote request transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<&types::PayoutsRouterData<F>> for WisePayoutQuoteRequest {
    type Error = Error;
    fn try_from(item: &types::PayoutsRouterData<F>) -> Result<Self, Self::Error> {
        let request = item.request.to_owned();
        match request.payout_type.to_owned() {
            storage_enums::PayoutType::Bank => Ok(Self {
                source_amount: Some(request.amount),
                source_currency: request.source_currency.to_string(),
                target_amount: None,
                target_currency: request.destination_currency.to_string(),
                pay_out: WisePayOutOption::default(),
            }),
            storage_enums::PayoutType::Card => Err(errors::ConnectorError::NotSupported {
                message: "Card payout fulfillment is not supported".to_string(),
                connector: "Wise",
                payment_experience: "".to_string(),
            })?,
        }
    }
}

// Payouts quote response transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<types::PayoutsResponseRouterData<F, WisePayoutQuoteResponse>>
    for types::PayoutsRouterData<F>
{
    type Error = Error;
    fn try_from(
        item: types::PayoutsResponseRouterData<F, WisePayoutQuoteResponse>,
    ) -> Result<Self, Self::Error> {
        let response: WisePayoutQuoteResponse = item.response;

        Ok(Self {
            response: Ok(types::PayoutsResponseData {
                status: Some(storage_enums::PayoutStatus::RequiresCreation),
                connector_payout_id: response.id,
                payout_eligible: None,
            }),
            ..item.data
        })
    }
}

// Payouts transfer creation request
#[cfg(feature = "payouts")]
impl<F> TryFrom<&types::PayoutsRouterData<F>> for WisePayoutCreateRequest {
    type Error = Error;
    fn try_from(item: &types::PayoutsRouterData<F>) -> Result<Self, Self::Error> {
        let request = item.request.to_owned();
        match request.payout_type.to_owned() {
            storage_enums::PayoutType::Bank => {
                let connector_customer_id = item.get_connector_customer_id()?;
                let quote_uuid =
                    request
                        .quote_id
                        .ok_or(errors::ConnectorError::MissingRequiredField {
                            field_name: "quote_id",
                        })?;
                let wise_transfer_details = WiseTransferDetails {
                    transfer_purpose: None,
                    source_of_funds: None,
                    transfer_purpose_sub_transfer_purpose: None,
                };
                let target_account: i64 = connector_customer_id.trim().parse().map_err(|_| {
                    errors::ConnectorError::MissingRequiredField {
                        field_name: "profile",
                    }
                })?;
                Ok(Self {
                    target_account,
                    quote_uuid,
                    customer_transaction_id: request.payout_id,
                    details: wise_transfer_details,
                })
            }
            storage_enums::PayoutType::Card => Err(errors::ConnectorError::NotSupported {
                message: "Card payout fulfillment is not supported".to_string(),
                connector: "Wise",
                payment_experience: "".to_string(),
            })?,
        }
    }
}

// Payouts transfer creation response
#[cfg(feature = "payouts")]
impl<F> TryFrom<types::PayoutsResponseRouterData<F, WisePayoutCreateResponse>>
    for types::PayoutsRouterData<F>
{
    type Error = Error;
    fn try_from(
        item: types::PayoutsResponseRouterData<F, WisePayoutCreateResponse>,
    ) -> Result<Self, Self::Error> {
        let response: WisePayoutCreateResponse = item.response;

        Ok(Self {
            response: Ok(types::PayoutsResponseData {
                status: Some(storage_enums::PayoutStatus::RequiresFulfillment),
                connector_payout_id: response.id.to_string(),
                payout_eligible: None,
            }),
            ..item.data
        })
    }
}

// Payouts fulfill request transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<&types::PayoutsRouterData<F>> for WisePayoutFulfillRequest {
    type Error = Error;
    fn try_from(item: &types::PayoutsRouterData<F>) -> Result<Self, Self::Error> {
        let request = item.request.to_owned();
        match request.payout_type.to_owned() {
            storage_enums::PayoutType::Bank => Ok(Self {
                fund_type: FundType::default(),
            }),
            storage_enums::PayoutType::Card => Err(errors::ConnectorError::NotSupported {
                message: "Card payout fulfillment is not supported".to_string(),
                connector: "Wise",
                payment_experience: "".to_string(),
            })?,
        }
    }
}

// Payouts fulfill response transform
#[cfg(feature = "payouts")]
impl<F> TryFrom<types::PayoutsResponseRouterData<F, WiseFulfillResponse>>
    for types::PayoutsRouterData<F>
{
    type Error = Error;
    fn try_from(
        item: types::PayoutsResponseRouterData<F, WiseFulfillResponse>,
    ) -> Result<Self, Self::Error> {
        let response: WiseFulfillResponse = item.response;

        Ok(Self {
            response: Ok(types::PayoutsResponseData {
                status: Some(storage_enums::PayoutStatus::foreign_from(response.status)),
                connector_payout_id: "".to_string(),
                payout_eligible: None,
            }),
            ..item.data
        })
    }
}

#[cfg(feature = "payouts")]
impl ForeignFrom<WiseStatus> for storage_enums::PayoutStatus {
    fn foreign_from(wise_status: WiseStatus) -> Self {
        match wise_status {
            WiseStatus::Completed => Self::Success,
            WiseStatus::Rejected => Self::Failed,
            WiseStatus::Pending | WiseStatus::Processing | WiseStatus::IncomingPaymentWaiting => {
                Self::Pending
            }
        }
    }
}

#[cfg(feature = "payouts")]
impl ForeignFrom<EntityType> for LegalType {
    fn foreign_from(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Individual | EntityType::Personal | EntityType::NonProfit => Self::Private,
            EntityType::Company | EntityType::PublicSector | EntityType::Business => Self::Business,
        }
    }
}
