CREATE TYPE sid AS (
    tag int4,
    bytes uuid
);

CREATE OR REPLACE FUNCTION sid_input(input_string text)
RETURNS sid
AS $$
DECLARE
    result sid;
    tag_string text;
    tag int4;
    bytes uuid;
    len integer;
BEGIN
    len := LENGTH(input_string);
    if len = 27 then
    	tag := 0;
    else
    	tag_string := substring(input_string for (len - 27));
    	tag := ascii(substring(tag_string from 1))
    	     + ascii(substring(tag_string from 2)) << 8
    	     + ascii(substring(tag_string from 3)) << 16
    	     + ascii(substring(tag_string from 4)) << 24;
    	input_string := substring(input_string from (len - 25));
    end if;

    bytes := uuid_nil();

    result := sid(tag, bytes);
    RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sid_output(value sid)
RETURNS text
AS $$
DECLARE
    output_string text;
BEGIN
	output_string := chr(sid.tag)
	             || chr(sid.tag >> 8)
	             || chr(sid.tag >> 16)
	             || chr(sid.tag >> 24);
	IF sid.tag <> 0 THEN
		output_string := output_string || '_';
	END IF;
    RETURN output_string;
END;
$$ LANGUAGE plpgsql;


-- create sid in the database




-- convert uuid to sid



-- convert sid to uuid