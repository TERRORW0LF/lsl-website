--
-- PostgreSQL database dump
--

-- Dumped from database version 16.6
-- Dumped by pg_dump version 16.6

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: Permissions; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public."Permissions" AS ENUM (
    'View',
    'Submit',
    'Trusted',
    'Delete',
    'Verify',
    'ManageRuns',
    'ManageUsers',
    'Administrator'
);


--
-- Name: run_remove(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.run_remove() RETURNS trigger
    LANGUAGE plpgsql
    AS $$DECLARE
	wr record;
	pb record;
BEGIN
	-- Old run was a wr.
	if OLD.is_wr then
		-- Find current wr.
		SELECT id, time, created_at
		INTO wr
		FROM run
		WHERE section_id = OLD.section_id
		ORDER BY time ASC, created_at ASC
		LIMIT 1;

		if NOT found then
			return OLD;
		end if;
		
		-- Set new wr.
		UPDATE run
		SET is_wr = true
		WHERE id = wr.id;

		-- Set new pb for user from deleted run.
		SELECT id, time, created_at
		INTO pb
		FROM run
		WHERE section_id = OLD.section_id
			AND user_id = OLD.user_id
		ORDER BY time ASC, created_at ASC
		LIMIT 1;

		if found then
			UPDATE run
			SET is_pb = true
			WHERE id = pb.id;
		end if;

		-- Calculate points for all pbs.
		UPDATE run
		SET points = GREATEST(100 * ln(1/(((time - wr.time)::real / wr.time::real) + 0.0025)) / 6, 0)
		WHERE section_id = OLD.section_id AND is_pb = true;

		return OLD;
	end if;

	-- Old run was pb.
	if OLD.is_pb then
	-- Find current pb.
		SELECT id, time, created_at
		INTO pb
		FROM run
		WHERE section_id = OLD.section_id
			AND user_id = OLD.user_id
		ORDER BY time ASC, created_at ASC
		LIMIT 1;

		-- Check if old time was pb
		if NOT found then
			return OLD;
		end if;
	
		-- Set current pb as pb.
		UPDATE run
		SET is_pb = true, points = GREATEST(100 * ln(1/(((time - wr.time)::real / wr.time::real) + 0.0025)) / 6, 0)
		WHERE id = pb.id;
	end if;

	return OLD;
END;$$;


--
-- Name: run_submit(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.run_submit() RETURNS trigger
    LANGUAGE plpgsql
    AS $$DECLARE
	wr record;
	pb record;
BEGIN
	SELECT id, time, created_at
	INTO pb
	FROM run
	WHERE section_id = NEW.section_id
		AND user_id = NEW.user_id
		AND is_pb = true
	LIMIT 1;

	if found and pb.time <= NEW.time then
		NEW.is_wr = false;
		NEW.is_pb = false;
		NEW.points = NULL;
		return NEW;
	end if;

	NEW.is_pb = true;
	-- Don't do work if it isn't needed.
	if found then
		UPDATE run
		SET is_pb = false, points = NULL
		WHERE id = pb.id;
	end if;

	SELECT id, time, created_at
	INTO wr
	FROM run
	WHERE section_id = NEW.section_id AND is_wr = true
	LIMIT 1;

	if found and wr.time <= NEW.time then
		NEW.points = GREATEST(100 * ln(1/(((NEW.time - wr.time)::real / wr.time::real) + 0.0025)) / 6, 0);
		NEW.is_wr = false;
		return NEW;
	end if;

	NEW.is_wr = true;
	NEW.points = 100;
	-- Don't do work if it isn't needed.
	if found then
		UPDATE run
		SET is_wr = false
		WHERE id = wr.id;

		UPDATE run
		SET points = GREATEST(100 * ln(1/(((time - NEW.time)::real / NEW.time::real) + 0.0025)) / 6, 0)
		WHERE section_id = NEW.section_id AND is_pb = true;
	end if;

	return NEW;
END;$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: discord; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.discord (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    snowflake character varying(32) NOT NULL,
    name character varying(32) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: discord_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.discord_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: discord_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.discord_id_seq OWNED BY public.discord.id;


--
-- Name: run; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.run (
    id integer NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
)
PARTITION BY RANGE (section_id);


--
-- Name: run_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.run_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: run_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.run_id_seq OWNED BY public.run.id;


--
-- Name: patch_1_00; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.patch_1_00 (
    id integer DEFAULT nextval('public.run_id_seq'::regclass) NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
);


--
-- Name: patch_1_41; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.patch_1_41 (
    id integer DEFAULT nextval('public.run_id_seq'::regclass) NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
);


--
-- Name: patch_1_50; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.patch_1_50 (
    id integer DEFAULT nextval('public.run_id_seq'::regclass) NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
);


--
-- Name: patch_2_00; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.patch_2_00 (
    id integer DEFAULT nextval('public.run_id_seq'::regclass) NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
);


--
-- Name: patch_2_13; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.patch_2_13 (
    id integer DEFAULT nextval('public.run_id_seq'::regclass) NOT NULL,
    section_id integer NOT NULL,
    user_id bigint NOT NULL,
    proof character varying(256) NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    is_pb boolean NOT NULL,
    is_wr boolean NOT NULL,
    points real,
    yt_id character(11),
    "time" numeric(8,3) NOT NULL
);


--
-- Name: permission; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.permission (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    token public."Permissions" NOT NULL
);


--
-- Name: permission_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.permission_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: permission_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.permission_id_seq OWNED BY public.permission.id;


--
-- Name: section; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.section (
    id integer NOT NULL,
    patch character(128) NOT NULL,
    layout character(128) NOT NULL,
    category character varying(128) NOT NULL,
    map character varying(128) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    code character(4) NOT NULL
);


--
-- Name: section_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.section_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: section_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.section_id_seq OWNED BY public.section.id;


--
-- Name: session; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session (
    id character varying(128) NOT NULL,
    expires bigint,
    session text NOT NULL
);


--
-- Name: user; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."user" (
    id bigint NOT NULL,
    name character varying(32) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    password character varying(512) NOT NULL,
    pfp character varying(512) DEFAULT 'default'::character varying NOT NULL,
    bio character varying(2048)
);


--
-- Name: user_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_id_seq OWNED BY public."user".id;


--
-- Name: patch_1_00; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_00 FOR VALUES FROM (1) TO (125);


--
-- Name: patch_1_41; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_41 FOR VALUES FROM (125) TO (373);


--
-- Name: patch_1_50; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_50 FOR VALUES FROM (373) TO (683);


--
-- Name: patch_2_00; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_2_00 FOR VALUES FROM (683) TO (1093);


--
-- Name: patch_2_13; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_2_13 DEFAULT;


--
-- Name: discord id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.discord ALTER COLUMN id SET DEFAULT nextval('public.discord_id_seq'::regclass);


--
-- Name: permission id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permission ALTER COLUMN id SET DEFAULT nextval('public.permission_id_seq'::regclass);


--
-- Name: run id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run ALTER COLUMN id SET DEFAULT nextval('public.run_id_seq'::regclass);


--
-- Name: section id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.section ALTER COLUMN id SET DEFAULT nextval('public.section_id_seq'::regclass);


--
-- Name: user id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."user" ALTER COLUMN id SET DEFAULT nextval('public.user_id_seq'::regclass);


--
-- Name: discord discord_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.discord
    ADD CONSTRAINT discord_pkey PRIMARY KEY (id);


--
-- Name: run id; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.run
    ADD CONSTRAINT id PRIMARY KEY (id, section_id);


--
-- Name: patch_1_00 patch_1_00_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.patch_1_00
    ADD CONSTRAINT patch_1_00_pkey PRIMARY KEY (id, section_id);


--
-- Name: patch_1_41 patch_1_41_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.patch_1_41
    ADD CONSTRAINT patch_1_41_pkey PRIMARY KEY (id, section_id);


--
-- Name: patch_1_50 patch_1_50_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.patch_1_50
    ADD CONSTRAINT patch_1_50_pkey PRIMARY KEY (id, section_id);


--
-- Name: patch_2_00 patch_2_00_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.patch_2_00
    ADD CONSTRAINT patch_2_00_pkey PRIMARY KEY (id, section_id);


--
-- Name: patch_2_13 patch_2_13_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.patch_2_13
    ADD CONSTRAINT patch_2_13_pkey PRIMARY KEY (id, section_id);


--
-- Name: permission permission_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permission
    ADD CONSTRAINT permission_pkey PRIMARY KEY (id);


--
-- Name: section section_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.section
    ADD CONSTRAINT section_pkey PRIMARY KEY (id);


--
-- Name: session session_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session
    ADD CONSTRAINT session_pkey PRIMARY KEY (id);


--
-- Name: user unique_user; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT unique_user UNIQUE (name);


--
-- Name: user user_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT user_pkey PRIMARY KEY (id);


--
-- Name: section_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX section_index ON ONLY public.run USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_1_00_section_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX patch_1_00_section_id_idx ON public.patch_1_00 USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_1_41_section_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX patch_1_41_section_id_idx ON public.patch_1_41 USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_1_50_section_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX patch_1_50_section_id_idx ON public.patch_1_50 USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_2_00_section_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX patch_2_00_section_id_idx ON public.patch_2_00 USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_2_13_section_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX patch_2_13_section_id_idx ON public.patch_2_13 USING btree (section_id) WITH (deduplicate_items='true');


--
-- Name: patch_1_00_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.id ATTACH PARTITION public.patch_1_00_pkey;


--
-- Name: patch_1_00_section_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.section_index ATTACH PARTITION public.patch_1_00_section_id_idx;


--
-- Name: patch_1_41_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.id ATTACH PARTITION public.patch_1_41_pkey;


--
-- Name: patch_1_41_section_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.section_index ATTACH PARTITION public.patch_1_41_section_id_idx;


--
-- Name: patch_1_50_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.id ATTACH PARTITION public.patch_1_50_pkey;


--
-- Name: patch_1_50_section_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.section_index ATTACH PARTITION public.patch_1_50_section_id_idx;


--
-- Name: patch_2_00_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.id ATTACH PARTITION public.patch_2_00_pkey;


--
-- Name: patch_2_00_section_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.section_index ATTACH PARTITION public.patch_2_00_section_id_idx;


--
-- Name: patch_2_13_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.id ATTACH PARTITION public.patch_2_13_pkey;


--
-- Name: patch_2_13_section_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.section_index ATTACH PARTITION public.patch_2_13_section_id_idx;


--
-- Name: run run_delete; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER run_delete AFTER DELETE ON public.run FOR EACH ROW EXECUTE FUNCTION public.run_remove();


--
-- Name: run run_insert; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER run_insert BEFORE INSERT ON public.run FOR EACH ROW EXECUTE FUNCTION public.run_submit();


--
-- Name: run section_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.run
    ADD CONSTRAINT section_id FOREIGN KEY (section_id) REFERENCES public.section(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: permission user_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permission
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: run user_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.run
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: discord user_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.discord
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

