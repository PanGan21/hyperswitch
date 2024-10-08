-- Backfill for organization table
UPDATE ORGANIZATION
SET org_id = id
WHERE org_id IS NULL;

ALTER TABLE ORGANIZATION DROP CONSTRAINT organization_pkey_id;

ALTER TABLE ORGANIZATION
ADD CONSTRAINT organization_pkey PRIMARY KEY (org_id);

-- back fill
UPDATE ORGANIZATION
SET org_name = organization_name
WHERE org_name IS NULL
    AND organization_name IS NOT NULL;

-- The new primary key for v2 merchant account will be `id`
ALTER TABLE merchant_account DROP CONSTRAINT merchant_account_pkey;

-- In order to run this query, the merchant_id column should be unique and not null
-- We need to backfill the id, a simple strategy will be to copy the values of id to merchant_id
-- Query to update the merchant_id column with values of id
UPDATE merchant_account
SET merchant_id = id
WHERE merchant_id IS NULL;

-- Note: This command might not run successfully for the existing table
-- This is because there will be some rows ( which are created via v2 application ) which will have id as empty
-- A backfill might be required to run this query
-- However if this is being run on a fresh database, this should succeed
ALTER TABLE merchant_account
ADD PRIMARY KEY (merchant_id);

UPDATE business_profile
SET profile_id = id
WHERE profile_id IS NULL;

ALTER TABLE business_profile DROP COLUMN id;

ALTER TABLE business_profile
ADD PRIMARY KEY (profile_id);

ALTER TABLE merchant_connector_account DROP CONSTRAINT merchant_connector_account_pkey;

UPDATE merchant_connector_account
SET merchant_connector_id = id
WHERE merchant_connector_id IS NULL;

ALTER TABLE merchant_connector_account
ADD PRIMARY KEY (merchant_connector_id);

ALTER TABLE merchant_connector_account
ALTER COLUMN profile_id DROP NOT NULL;

-- Run this query only when V1 is deprecated
ALTER TABLE customers DROP CONSTRAINT customers_pkey;

-- Back filling before making it primary key
UPDATE customers
SET customer_id = id
WHERE customer_id IS NULL;

ALTER TABLE customers
ADD PRIMARY KEY (merchant_id, customer_id);
