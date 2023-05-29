fn pay_create_req() -> serde_json::Value {
    serde_json::json!(
        {
            "amount": 6540,
            "currency": "USD",
            "confirm": true,
            "capture_method": "automatic",
            "capture_on": "2022-09-10T10:11:12Z",
            "amount_to_capture": 6540,
            "customer_id": "StripeCustomer",
            "email": "guest@example.com",
            "name": "John Doe",
            "phone": "999999999",
            "phone_country_code": "+1",
            "description": "Its my first payment request",
            "authentication_type": "no_three_ds",
            "return_url": "https://google.com",
            "payment_method": "card",
            "payment_method_type": "credit",
            "payment_method_data": {
              "card": {
                "card_number": "4242424242424242",
                "card_exp_month": "10",
                "card_exp_year": "25",
                "card_holder_name": "joseph Doe",
                "card_cvc": "123"
              }
            },
            "billing": {
              "address": {
                "line1": "1467",
                "line2": "Harrison Street",
                "line3": "Harrison Street",
                "city": "San Fransico",
                "state": "California",
                "zip": "94122",
                "country": "US",
                "first_name": "joseph",
                "last_name": "Doe"
              },
              "phone": {
                "number": "8056594427",
                "country_code": "+91"
              }
            },
            "shipping": {
              "address": {
                "line1": "1467",
                "line2": "Harrison Street",
                "line3": "Harrison Street",
                "city": "San Fransico",
                "state": "California",
                "zip": "94122",
                "country": "US",
                "first_name": "joseph",
                "last_name": "Doe"
              },
              "phone": {
                "number": "8056594427",
                "country_code": "+91"
              }
            },
            "statement_descriptor_name": "joseph",
            "statement_descriptor_suffix": "JS",
            "metadata": {
              "udf1": "value1",
              "new_customer": "true",
              "login_date": "2019-09-10T10:11:12Z"
            },
            "business_label" : "default",
            "business_country" : "US"
          }
    )
}