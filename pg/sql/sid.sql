CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION generate_sid() RETURNS uuid
    AS $$
        SELECT (lpad(to_hex(floor(extract(epoch FROM clock_timestamp()) * 1000)::bigint), 12, '0') || encode(gen_random_bytes(10), 'hex'))::uuid;
    $$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION uuid_to_sid(id uuid) RETURNS text AS $$
DECLARE
  encoding   bytea = '0123456789abcdefghjkmnpqrstvwxyz';
  output     text  = '';
  uuid_bytes bytea = uuid_send(id);
BEGIN
  -- Encode the timestamp
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 0) & 224) >> 5));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 0) & 31)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 1) & 248) >> 3));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 1) & 7) << 2) | ((GET_BYTE(uuid_bytes, 2) & 192) >> 6)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 2) & 62) >> 1));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 2) & 1) << 4) | ((GET_BYTE(uuid_bytes, 3) & 240) >> 4)));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 3) & 15) << 1) | ((GET_BYTE(uuid_bytes, 4) & 128) >> 7)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 4) & 124) >> 2));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 4) & 3) << 3) | ((GET_BYTE(uuid_bytes, 5) & 224) >> 5)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 5) & 31)));

  -- Encode the entropy
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 6) & 248) >> 3));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 6) & 7) << 2) | ((GET_BYTE(uuid_bytes, 7) & 192) >> 6)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 7) & 62) >> 1));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 7) & 1) << 4) | ((GET_BYTE(uuid_bytes, 8) & 240) >> 4)));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 8) & 15) << 1) | ((GET_BYTE(uuid_bytes, 9) & 128) >> 7)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 9) & 124) >> 2));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 9) & 3) << 3) | ((GET_BYTE(uuid_bytes, 10) & 224) >> 5)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 10) & 31)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 11) & 248) >> 3));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 11) & 7) << 2) | ((GET_BYTE(uuid_bytes, 12) & 192) >> 6)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 12) & 62) >> 1));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 12) & 1) << 4) | ((GET_BYTE(uuid_bytes, 13) & 240) >> 4)));
  output = output || '_';
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 13) & 15) << 1) | ((GET_BYTE(uuid_bytes, 14) & 128) >> 7)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 14) & 124) >> 2));
  output = output || CHR(GET_BYTE(encoding, ((GET_BYTE(uuid_bytes, 14) & 3) << 3) | ((GET_BYTE(uuid_bytes, 15) & 224) >> 5)));
  output = output || CHR(GET_BYTE(encoding, (GET_BYTE(uuid_bytes, 15) & 31)));
  RETURN output;
END
$$
LANGUAGE plpgsql
IMMUTABLE;

CREATE OR REPLACE FUNCTION sid_to_uuid(input TEXT)
RETURNS uuid AS $$
DECLARE
    lookup_table CONSTANT INTEGER[] := ARRAY[
        24, 25, 26, 255, 27, 28, 29, 30, 31, 255, 0, 1, 2,
        3, 4, 5, 6, 7, 8, 9, 255, 10, 11, 12, 13, 14,
        15, 16, 17, 255, 18, 19, 255, 20, 21, 255, 22, 23
    ];
    intermediate INTEGER[];
    result BYTEA;
    i INTEGER;
    j INTEGER;
    k INTEGER;
    d0 INTEGER;
    d1 INTEGER;
    d2 INTEGER;
    d3 INTEGER;
    d4 INTEGER;
    d5 INTEGER;
    d6 INTEGER;
    d7 INTEGER;
    d8 INTEGER;
BEGIN
    -- Check input length
    IF length(input) != 27 THEN
        RAISE EXCEPTION 'Invalid input length';
    END IF;

    -- Check separator
    IF substring(input FROM 23 FOR 1) != '_' THEN
        RAISE EXCEPTION 'No separator found';
    END IF;

    result := convert_to(input, 'UTF8');

    -- Build intermediate array
    intermediate := ARRAY[]::INTEGER[];
    FOR i IN 0..21 LOOP
        intermediate := intermediate || lookup_table[get_byte(result, i) % 38 + 1];
    END LOOP;
    FOR i IN 23..26 LOOP
        intermediate := intermediate || lookup_table[get_byte(result, i) % 38 + 1];
    END LOOP;
    -- Check for invalid characters
    IF array_position(intermediate, 255) IS NOT NULL THEN
        RAISE EXCEPTION 'Invalid character in input';
    END IF;

    -- Perform decoding
    FOR i IN 0..2 LOOP
        j := i * 8 + 1;
        k := i * 5 + 1;
        d0 := intermediate[j];
        d1 := intermediate[j + 1];
        d2 := intermediate[j + 2];
        d3 := intermediate[j + 3];
        d4 := intermediate[j + 4];
        d5 := intermediate[j + 5];
        d6 := intermediate[j + 6];
        d7 := intermediate[j + 7];
        d8 := intermediate[j + 8];

        result := set_byte(result, k - 1, (d0 << 5 | d1)::INTEGER);
        result := set_byte(result, k, (d2 << 3 | (d3 >> 2))::INTEGER);
        result := set_byte(result, k + 1, (d3 << 6 | (d4 << 1) | (d5 >> 4))::INTEGER);
        result := set_byte(result, k + 2, (d5 << 4 | (d6 >> 1))::INTEGER);
        result := set_byte(result, k + 3, (d6 << 7 | (d7 << 2) | (d8 >> 3))::INTEGER);
    END LOOP;

    result := set_byte(result, 15, (intermediate[25] << 5 | intermediate[26])::INTEGER);

    -- Convert BYTEA to UUID
    RETURN CAST(ENCODE(substring(result from 1 for 16), 'hex') AS UUID);
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

CREATE OR REPLACE FUNCTION generate_sid() RETURNS uuid
    AS $$
        SELECT generate_ulid();
    $$ LANGUAGE SQL;