-- This file contains all new columns being added as part of v2 refactoring.
-- The new columns added should work with both v1 and v2 applications.
ALTER TABLE customers
ADD COLUMN IF NOT EXISTS merchant_reference_id VARCHAR(64),
    ADD COLUMN IF NOT EXISTS default_billing_address BYTEA DEFAULT NULL,
    ADD COLUMN IF NOT EXISTS default_shipping_address BYTEA DEFAULT NULL,
    ADD COLUMN IF NOT EXISTS status "DeleteStatus" NOT NULL DEFAULT 'active';
CREATE TYPE "OrderFulfillmentTimeOrigin" AS ENUM ('create', 'confirm');

ALTER TABLE business_profile
ADD COLUMN routing_algorithm_id VARCHAR(64) DEFAULT NULL,
    ADD COLUMN order_fulfillment_time BIGINT DEFAULT NULL,
    ADD COLUMN order_fulfillment_time_origin "OrderFulfillmentTimeOrigin" DEFAULT NULL,
    ADD COLUMN frm_routing_algorithm_id VARCHAR(64) DEFAULT NULL,
    ADD COLUMN payout_routing_algorithm_id VARCHAR(64) DEFAULT NULL,
    ADD COLUMN default_fallback_routing JSONB DEFAULT NULL;
