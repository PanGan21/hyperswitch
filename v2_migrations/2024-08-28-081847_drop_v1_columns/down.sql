ALTER TABLE ORGANIZATION
ADD COLUMN org_id VARCHAR(32),
    ADD COLUMN org_name TEXT;

ALTER TABLE merchant_account
ADD COLUMN merchant_id VARCHAR(64),
    ADD COLUMN return_url VARCHAR(255),
    ADD COLUMN enable_payment_response_hash BOOLEAN DEFAULT FALSE,
    ADD COLUMN payment_response_hash_key VARCHAR(255),
    ADD COLUMN redirect_to_merchant_with_http_post BOOLEAN DEFAULT FALSE,
    ADD COLUMN sub_merchants_enabled BOOLEAN DEFAULT FALSE,
    ADD COLUMN parent_merchant_id VARCHAR(64),
    ADD COLUMN locker_id VARCHAR(64),
    ADD COLUMN intent_fulfillment_time BIGINT,
    ADD COLUMN default_profile VARCHAR(64),
    ADD COLUMN payment_link_config JSONB NULL,
    ADD COLUMN pm_collect_link_config JSONB NULL,
    ADD COLUMN is_recon_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN webhook_details JSONB NULL,
    ADD COLUMN routing_algorithm JSON,
    ADD COLUMN frm_routing_algorithm JSONB,
    ADD COLUMN payout_routing_algorithm JSONB;

-- The default value is for temporary purpose only
ALTER TABLE merchant_account
ADD COLUMN primary_business_details JSON NOT NULL DEFAULT '[{"country": "US", "business": "default"}]';

ALTER TABLE merchant_account
ALTER COLUMN primary_business_details DROP DEFAULT;

ALTER TABLE business_profile
ADD COLUMN profile_id VARCHAR(64),
    ADD COLUMN routing_algorithm JSON DEFAULT NULL,
    ADD COLUMN intent_fulfillment_time BIGINT DEFAULT NULL,
    ADD COLUMN frm_routing_algorithm JSONB DEFAULT NULL,
    ADD COLUMN payout_routing_algorithm JSONB DEFAULT NULL;

ALTER TABLE merchant_connector_account
ADD COLUMN IF NOT EXISTS business_country "CountryAlpha2",
    ADD COLUMN IF NOT EXISTS business_label VARCHAR(255),
    ADD COLUMN IF NOT EXISTS business_sub_label VARCHAR(64),
    ADD COLUMN IF NOT EXISTS test_mode BOOLEAN,
    ADD COLUMN IF NOT EXISTS frm_configs jsonb,
    ADD COLUMN IF NOT EXISTS merchant_connector_id VARCHAR(32);

ALTER TABLE customers
ADD COLUMN customer_id VARCHAR(64),
    ADD COLUMN address_id VARCHAR(64);
