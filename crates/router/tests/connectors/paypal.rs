use masking::Secret;
use regex::Regex;
use router::types::{self, api, storage::enums, AccessToken, ConnectorAuthType};
use serde_json::json;

// use storage_models::schema::payment_attempt::connector_metadata;
use crate::{
    connector_auth,
    utils::{self, Connector, ConnectorActions},
};

struct PaypalTest;
impl ConnectorActions for PaypalTest {}
impl Connector for PaypalTest {
    fn get_data(&self) -> types::api::ConnectorData {
        use router::connector::Paypal;
        types::api::ConnectorData {
            connector: Box::new(&Paypal),
            connector_name: types::Connector::Paypal,
            get_token: types::api::GetToken::Connector,
        }
    }

    fn get_auth_token(&self) -> ConnectorAuthType {
        types::ConnectorAuthType::from(
            connector_auth::ConnectorAuthentication::new()
                .paypal
                .expect("Missing connector authentication configuration"),
        )
    }

    fn get_name(&self) -> String {
        "paypal".to_string()
    }
}
static CONNECTOR: PaypalTest = PaypalTest {};

fn get_access_token() -> Option<AccessToken> {
    let connector = PaypalTest {};

    match connector.get_auth_token() {
        ConnectorAuthType::BodyKey { api_key, key1: _ } => Some(AccessToken {
            token: api_key,
            expires: 18600,
        }),
        _ => None,
    }
}
fn get_default_payment_info() -> Option<utils::PaymentInfo> {
    Some(utils::PaymentInfo {
        access_token: get_access_token(),
        ..Default::default()
    })
}

fn get_payment_data() -> Option<types::PaymentsAuthorizeData> {
    Some(types::PaymentsAuthorizeData {
        payment_method_data: types::api::PaymentMethodData::Card(api::Card {
            card_number: Secret::new(String::from("4000020000000000")),
            ..utils::CCardType::default().0
        }),
        ..utils::PaymentAuthorizeType::default().0
    })
}

// Cards Positive Tests
// Creates a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_only_authorize_payment() {
    let response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    assert_eq!(response.status, enums::AttemptStatus::Authorized);
}

// Captures a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_capture_authorized_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let connector_meta = utils::get_connector_metadata(authorize_response.response);
    let response = CONNECTOR
        .capture_payment(
            txn_id,
            Some(types::PaymentsCaptureData {
                connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    assert_eq!(response.status, enums::AttemptStatus::Charged);
}

// Partially captures a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_partially_capture_authorized_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let connector_meta = utils::get_connector_metadata(authorize_response.response);
    let response = CONNECTOR
        .capture_payment(
            txn_id,
            Some(types::PaymentsCaptureData {
                connector_meta,
                amount_to_capture: Some(50),
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    assert_eq!(response.status, enums::AttemptStatus::Charged);
}

// Synchronizes a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_sync_authorized_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone());
    let response = CONNECTOR
        .psync_retry_till_status_matches(
            enums::AttemptStatus::Authorized,
            Some(types::PaymentsSyncData {
                connector_transaction_id: router::types::ResponseId::ConnectorTransactionId(
                    txn_id.unwrap(),
                ),
                encoded_data: None,
                capture_method: None,
                connector_meta: None,
            }),
            get_default_payment_info(),
        )
        .await
        .expect("PSync response");
    assert_eq!(response.status, enums::AttemptStatus::Authorized,);
}

// Voids a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_void_authorized_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let connector_meta = utils::get_connector_metadata(authorize_response.response);
    let response = CONNECTOR
        .void_payment(
            txn_id,
            Some(types::PaymentsCancelData {
                connector_transaction_id: String::from(""),
                cancellation_reason: Some("requested_by_customer".to_string()),
                connector_meta,
                ..Default::default()
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Void payment response");
    assert_eq!(response.status, enums::AttemptStatus::Voided);
}

// Refunds a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_refund_manually_captured_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Partially refunds a payment using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_partially_refund_manually_captured_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                refund_amount: 50,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Synchronizes a refund using the manual capture flow (Non 3DS).
#[actix_web::test]
async fn should_sync_manually_captured_refund() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let refund_response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    let response = CONNECTOR
        .rsync_retry_till_status_matches(
            enums::RefundStatus::Success,
            refund_response.response.unwrap().connector_refund_id,
            None,
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Creates a payment using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_make_payment() {
    let authorize_response = CONNECTOR
        .make_payment(get_payment_data(), get_default_payment_info())
        .await
        .unwrap();
    assert_eq!(authorize_response.status, enums::AttemptStatus::Charged);
}

// Synchronizes a payment using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_sync_auto_captured_payment() {
    let authorize_response = CONNECTOR
        .make_payment(get_payment_data(), get_default_payment_info())
        .await
        .unwrap();
    assert_eq!(
        authorize_response.status.clone(),
        enums::AttemptStatus::Charged
    );
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone());
    assert_ne!(txn_id, None, "Empty connector transaction id");
    let connector_meta = utils::get_connector_metadata(authorize_response.response);
    let response = CONNECTOR
        .psync_retry_till_status_matches(
            enums::AttemptStatus::Charged,
            Some(types::PaymentsSyncData {
                connector_transaction_id: router::types::ResponseId::ConnectorTransactionId(
                    txn_id.unwrap(),
                ),
                encoded_data: None,
                capture_method: Some(enums::CaptureMethod::Automatic),
                connector_meta,
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(response.status, enums::AttemptStatus::Charged,);
}

// Refunds a payment using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_refund_auto_captured_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Partially refunds a payment using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_partially_refund_succeeded_payment() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let refund_response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                refund_amount: 50,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        refund_response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Creates multiple refunds against a payment using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_refund_succeeded_payment_multiple_times() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    for _x in 0..2 {
        let refund_response = CONNECTOR
            .refund_payment(
                txn_id.clone(),
                Some(types::RefundsData {
                    connector_metadata: refund_connector_metadata.clone(),
                    refund_amount: 50,
                    ..utils::PaymentRefundType::default().0
                }),
                get_default_payment_info(),
            )
            .await
            .unwrap();
        assert_eq!(
            refund_response.response.unwrap().refund_status,
            enums::RefundStatus::Success,
        );
    }
}

// Synchronizes a refund using the automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_sync_refund() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let refund_response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    let response = CONNECTOR
        .rsync_retry_till_status_matches(
            enums::RefundStatus::Success,
            refund_response.response.unwrap().connector_refund_id,
            None,
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap().refund_status,
        enums::RefundStatus::Success,
    );
}

// Cards Negative scenerios
// Creates a payment with incorrect card number.
#[actix_web::test]
async fn should_fail_payment_for_incorrect_card_number() {
    let response = CONNECTOR
        .make_payment(
            Some(types::PaymentsAuthorizeData {
                payment_method_data: types::api::PaymentMethodData::Card(api::Card {
                    card_number: Secret::new("1234567891011".to_string()),
                    ..utils::CCardType::default().0
                }),
                ..utils::PaymentAuthorizeType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap_err().message,
        r#"[{"issue":"UNPROCESSABLE_ENTITY","description":"UNPROCESSABLE_ENTITY"}]"#.to_string(),
    );
}

// Creates a payment with empty card number.
#[actix_web::test]
async fn should_fail_payment_for_empty_card_number() {
    let response = CONNECTOR
        .make_payment(
            Some(types::PaymentsAuthorizeData {
                payment_method_data: types::api::PaymentMethodData::Card(api::Card {
                    card_number: Secret::new(String::from("")),
                    ..utils::CCardType::default().0
                }),
                ..utils::PaymentAuthorizeType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    let x = response.response.unwrap_err();
    assert_eq!(
        x.message,
        r#"[{"field":"/payment_source/card/number","issue":"CARD_NUMBER_REQUIRED","description":"The card number is required when attempting to process payment with card."}]"#,
    );
}

// Creates a payment with incorrect CVC.
#[actix_web::test]
async fn should_fail_payment_for_incorrect_cvc() {
    let response = CONNECTOR
        .make_payment(
            Some(types::PaymentsAuthorizeData {
                payment_method_data: types::api::PaymentMethodData::Card(api::Card {
                    card_cvc: Secret::new("12345".to_string()),
                    ..utils::CCardType::default().0
                }),
                ..utils::PaymentAuthorizeType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap_err().message,
        r#"[{"field":"/payment_source/card/security_code","value":"12345","location":"body","issue":"INVALID_PARAMETER_SYNTAX","description":"The value of a field does not conform to the expected format."}]"#.to_string(),
    );
}

// Creates a payment with incorrect expiry month.
#[actix_web::test]
async fn should_fail_payment_for_invalid_exp_month() {
    let response = CONNECTOR
        .make_payment(
            Some(types::PaymentsAuthorizeData {
                payment_method_data: types::api::PaymentMethodData::Card(api::Card {
                    card_exp_month: Secret::new("20".to_string()),
                    ..utils::CCardType::default().0
                }),
                ..utils::PaymentAuthorizeType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap_err().message,
        r#"[{"field":"/payment_source/card/expiry","value":"2025-20","location":"body","issue":"INVALID_PARAMETER_SYNTAX","description":"The value of a field does not conform to the expected format."}]"#,
    );
}

// Creates a payment with incorrect expiry year.
#[actix_web::test]
async fn should_fail_payment_for_incorrect_expiry_year() {
    let response = CONNECTOR
        .make_payment(
            Some(types::PaymentsAuthorizeData {
                payment_method_data: types::api::PaymentMethodData::Card(api::Card {
                    card_exp_year: Secret::new("2000".to_string()),
                    ..utils::CCardType::default().0
                }),
                ..utils::PaymentAuthorizeType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.response.unwrap_err().message,
        r#"[{"field":"/payment_source/card/expiry","location":"body","issue":"CARD_EXPIRED","description":"The card is expired."}]"#.to_string(),
    );
}

// Voids a payment using automatic capture flow (Non 3DS).
#[actix_web::test]
async fn should_fail_void_payment_for_auto_capture() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id,
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    assert_eq!(capture_response.status, enums::AttemptStatus::Charged);
    let txn_id = utils::get_connector_transaction_id(capture_response.clone().response);
    let connector_meta = utils::get_connector_metadata(capture_response.response);
    assert_ne!(txn_id, None, "Empty connector transaction id");
    let void_response = CONNECTOR
        .void_payment(
            txn_id.unwrap(),
            Some(types::PaymentsCancelData {
                connector_transaction_id: String::from(""),
                cancellation_reason: Some("requested_by_customer".to_string()),
                connector_meta,
                ..Default::default()
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Void payment response");
    let re = Regex::new(r#"[{"description":"Specified resource ID does not exist. Please check the resource ID and try again.","field":"authorization_id","issue":"INVALID_RESOURCE_ID","location":"path","value":".*"}]"#).unwrap();
    assert!(re.is_match(&void_response.response.unwrap_err().message));
}

// Captures a payment using invalid connector payment id.
#[actix_web::test]
async fn should_fail_capture_for_invalid_payment() {
    let connector_meta = Some(json!({
        "txn_id": "56YH8TZ",
    }));
    let capture_response = CONNECTOR
        .capture_payment(
            "123456789".to_string(),
            Some(types::PaymentsCaptureData {
                connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    assert_eq!(
        capture_response.response.unwrap_err().message,
        r#"[{"issue":"INVALID_RESOURCE_ID","description":"Specified resource ID does not exist. Please check the resource ID and try again."}]"#,
    );
}

// Refunds a payment with refund amount higher than payment amount.
#[actix_web::test]
async fn should_fail_for_refund_amount_higher_than_payment_amount() {
    let authorize_response = CONNECTOR
        .authorize_payment(get_payment_data(), get_default_payment_info())
        .await
        .expect("Authorize payment response");
    let txn_id = utils::get_connector_transaction_id(authorize_response.response.clone()).unwrap();
    let capture_connector_meta = utils::get_connector_metadata(authorize_response.response);
    let capture_response = CONNECTOR
        .capture_payment(
            txn_id.clone(),
            Some(types::PaymentsCaptureData {
                connector_meta: capture_connector_meta,
                ..utils::PaymentCaptureType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .expect("Capture payment response");
    let refund_connector_metadata = utils::get_connector_metadata(capture_response.response);
    let response = CONNECTOR
        .refund_payment(
            txn_id,
            Some(types::RefundsData {
                connector_metadata: refund_connector_metadata,
                refund_amount: 150,
                ..utils::PaymentRefundType::default().0
            }),
            get_default_payment_info(),
        )
        .await
        .unwrap();
    let re = Regex::new(r#"[{"description":"Specified resource ID does not exist. Please check the resource ID and try again.","field":"capture_id","issue":"INVALID_RESOURCE_ID","location":"path","value":".*"}]"#).unwrap();
    assert!(re.is_match(&response.response.unwrap_err().message));
}

// Connector dependent test cases goes here

// [#478]: add unit tests for non 3DS, wallets & webhooks in connector tests