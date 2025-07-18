PGDMP      )                }           lsl    16.9    16.9 z    �           0    0    ENCODING    ENCODING        SET client_encoding = 'UTF8';
                      false            �           0    0 
   STDSTRINGS 
   STDSTRINGS     (   SET standard_conforming_strings = 'on';
                      false            �           0    0 
   SEARCHPATH 
   SEARCHPATH     8   SELECT pg_catalog.set_config('search_path', '', false);
                      false            �           1262    17176    lsl    DATABASE     o   CREATE DATABASE lsl WITH TEMPLATE = template0 ENCODING = 'UTF8' LOCALE_PROVIDER = libc LOCALE = 'en_US.UTF-8';
    DROP DATABASE lsl;
                postgres    false            �           0    0    DATABASE lsl    ACL     &   GRANT CONNECT ON DATABASE lsl TO lsl;
                   postgres    false    3582                        0    0    SCHEMA public    ACL     #   GRANT ALL ON SCHEMA public TO lsl;
                   pg_database_owner    false    5            d           1247    22307    permissions    TYPE     �   CREATE TYPE public.permissions AS ENUM (
    'View',
    'Submit',
    'Trusted',
    'Delete',
    'Verify',
    'ManageRuns',
    'ManageUsers',
    'Administrator'
);
    DROP TYPE public.permissions;
       public          postgres    false            g           1247    22324    title    TYPE     �   CREATE TYPE public.title AS ENUM (
    'None',
    'Surfer',
    'SuperSurfer',
    'EpicSurfer',
    'LegendarySurfer',
    'MythicSurfer',
    'TopOne'
);
    DROP TYPE public.title;
       public          postgres    false            �            1255    22339    activity_notify()    FUNCTION     �   CREATE FUNCTION public.activity_notify() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	PERFORM pg_notify('activity', NEW.id::text);
	RETURN NULL;
END;$$;
 (   DROP FUNCTION public.activity_notify();
       public          postgres    false            �            1255    22340    activity_rank_update()    FUNCTION     N  CREATE FUNCTION public.activity_rank_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	if OLD.title IS NOT NULL AND NEW.title IS NOT NULL AND OLD.title <> NEW.title then
		INSERT INTO activity (user_id, rank_id, title_old, title_new, created_at)
		VALUES (OLD.user_id, OLD.id, OLD.title, NEW.title, NEW.updated_at);
	end if;
	if OLD.rank IS NOT NULL AND NEW.rank IS NOT NULL AND OLD.rank <> NEW.rank then
		INSERT INTO activity (user_id, rank_id, rank_old, rank_new, created_at)
		VALUES (OLD.user_id, OLD.id, OLD.rank, NEW.rank, NEW.updated_at);
	end if;
	RETURN NULL;
END;$$;
 -   DROP FUNCTION public.activity_rank_update();
       public          postgres    false            �            1255    22341    activity_user_add()    FUNCTION     �   CREATE FUNCTION public.activity_user_add() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	INSERT INTO activity (user_id)
	VALUES (NEW.id);
	RETURN NULL;
END;$$;
 *   DROP FUNCTION public.activity_user_add();
       public          postgres    false            �            1255    22342    discord_notify()    FUNCTION     �   CREATE FUNCTION public.discord_notify() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	PERFORM pg_notify('discord', NEW.id::text);
	RETURN NULL;
END;$$;
 '   DROP FUNCTION public.discord_notify();
       public          postgres    false            �            1255    22344    run_remove()    FUNCTION     �  CREATE FUNCTION public.run_remove() RETURNS trigger
    LANGUAGE plpgsql
    AS $$DECLARE
	submits integer;
	rank_count integer;
	se record;
	ra record;
	wr record;
	pb record;
	perce double precision;
	found_wr boolean = false;
BEGIN
	if NOT OLD.is_pb then
		RETURN OLD;
	end if;
	
	SELECT patch, layout, category FROM section INTO se
	WHERE id = OLD.section_id;

	SELECT COUNT(DISTINCT map) 
	FROM run 
	JOIN section ON section_id = section.id
	INTO submits 
	WHERE patch = se.patch AND layout = se.layout 
		AND category = se.category AND user_id = OLD.user_id;

	-- Update percentage on user rank.
	if submits = 0 then
		DELETE FROM rank
		WHERE patch = se.patch AND layout = se.layout 
		AND category = se.category AND user_id = OLD.user_id;
	else
		UPDATE rank
		SET percentage = submits::double precision / 
			(SELECT COUNT(id) 
				FROM section 
				WHERE patch = se.patch AND layout = se.layout 
				AND category = se.category)::double precision
		WHERE patch = se.patch AND layout = se.layout 
			AND category = se.category AND user_id = OLD.user_id
		RETURNING percentage INTO perce;
	end if;

	SELECT COUNT(id)
	FROM rank
	INTO rank_count
	WHERE patch = se.patch AND user_id = OLD.user_id;

	if rank_count = 1 then
		DELETE FROM rank
		WHERE patch = se.patch AND layout IS NULL 
			AND category IS NULL AND user_id = OLD.user_id;
	else
		UPDATE rank
		SET percentage = (SELECT COUNT(DISTINCT section_id) 
			FROM run r
			JOIN section s ON section_id = s.id
			WHERE patch = se.patch AND user_id = OLD.user_id) / 
			(SELECT COUNT(id) 
				FROM section 
				WHERE patch = se.patch)::double precision
		WHERE patch = se.patch AND layout IS NULL 
			AND category IS NULL AND user_id = OLD.user_id;
	end if;
	
	-- Old run was a wr.
	if OLD.is_wr then
		-- Find current wr.
		SELECT id, time, created_at
		INTO wr
		FROM run
		WHERE section_id = OLD.section_id
		ORDER BY time ASC, created_at ASC
		LIMIT 1;

		if found then
			found_wr = true;
				
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
			SET points = GREATEST(3.0 - 2.0 * time::double precision / wr.time::double precision, 0.0)
			WHERE section_id = OLD.section_id AND is_pb = true;

			WITH ran AS (SELECT percentage AS p, user_id AS u 
				FROM rank
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category),
			perc AS (SELECT AVG(points) AS p, user_id AS u
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category
				GROUP BY user_id)
		UPDATE rank r
		SET updated_at = OLD.created_at,
			rating = (2000 + 8000 
				* LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.15) / 2.03688192726104 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4)) 
			* POW(LN((SELECT p FROM perc WHERE u = r.user_id) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.01) / 4.61512051684126 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4))
		WHERE patch = se.patch AND layout = se.layout AND category = se.category;
		
		-- Update overall ratings.
		WITH ran AS (SELECT percentage AS p, user_id AS u 
				FROM rank
				WHERE patch = se.patch AND layout IS NULL 
					AND category IS NULL),
			perc AS (SELECT AVG(points) AS p, user_id AS u
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch
				GROUP BY user_id)
		UPDATE rank r
		SET updated_at = OLD.created_at,
			rating = (2000 + 8000 
				* LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.15) / 2.03688192726104 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4)) 
			* POW(LN((SELECT p FROM perc WHERE u = r.user_id) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.01) / 4.61512051684126 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4))
		WHERE patch = se.patch AND layout IS NULL AND category IS NULL;
		end if;
	-- Old run was pb.
	else
	-- Find current pb.
		SELECT id, time, created_at
		INTO pb
		FROM run
		WHERE section_id = OLD.section_id
			AND user_id = OLD.user_id
		ORDER BY time ASC, created_at ASC
		LIMIT 1;

		-- Get current wr
		if found then
			SELECT id, time, created_at
			INTO wr
			FROM run
			WHERE section_id = OLD.section_id
				AND is_wr = true;
	
			-- Set current pb as pb.
			UPDATE run
			SET is_pb = true, points = GREATEST(3.0 - 2.0 * time::double precision / wr.time::double precision, 0.0)
			WHERE id = pb.id;
		end if;

		-- Update category rating.
		SELECT id, percentage INTO ra
		FROM rank
		WHERE patch = se.patch AND layout = se.layout 
			AND category = se.category AND user_id = OLD.user_id;

		WITH perc AS (SELECT AVG(points) 
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category AND user_id = OLD.user_id)
		UPDATE rank
		SET updated_at = OLD.created_at,
			rating = (2000 + 8000 * LN(1 + ra.percentage / 0.15) / 2.03688192726104 
				* (1.25 - ra.percentage / 4)) 
			* POW(LN((SELECT * FROM perc) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + ra.percentage / 0.01) / 4.61512051684126 
				* (1.25 - ra.percentage / 4))
		WHERE id = ra.id;

		-- Update overall rating.
		SELECT id, percentage INTO ra
		FROM rank
		WHERE patch = se.patch AND layout IS NULL
			AND category IS NULL AND user_id = OLD.user_id;

		WITH perc AS (SELECT AVG(points) 
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND user_id = OLD.user_id)
		UPDATE rank
		SET updated_at = OLD.created_at,
			rating = (2000 + 8000 * LN(1 + ra.percentage / 0.15) / 2.03688192726104 
				* (1.25 - ra.percentage / 4)) 
			* POW(LN((SELECT * FROM perc) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + ra.percentage / 0.01) / 4.61512051684126
				* (1.25 - ra.percentage / 4))
		WHERE id = ra.id;
	end if;

	CALL update_rank(se.patch, se.layout, se.category);
	CALL update_rank(se.patch, NULL, NULL);

	return OLD;
END;
$$;
 #   DROP FUNCTION public.run_remove();
       public          postgres    false            �            1255    22345    run_submit()    FUNCTION     �  CREATE FUNCTION public.run_submit() RETURNS trigger
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
		NEW.points = GREATEST(3.0 - 2 * NEW.time::double precision / wr.time::double precision, 0.0);
		NEW.is_wr = false;
		return NEW;
	end if;

	NEW.is_wr = true;
	NEW.points = 1.0;
	-- Don't do work if it isn't needed.
	if found then
		UPDATE run
		SET is_wr = false
		WHERE id = wr.id;

		UPDATE run
		SET points = GREATEST(3.0 - 2 * time::double precision / NEW.time::double precision, 0.0)
		WHERE section_id = NEW.section_id AND is_pb = true;
	end if;

	return NEW;
END;$$;
 #   DROP FUNCTION public.run_submit();
       public          postgres    false            �            1255    22346    run_submit_ranks()    FUNCTION     �  CREATE FUNCTION public.run_submit_ranks() RETURNS trigger
    LANGUAGE plpgsql
    AS $$DECLARE
	submits integer;
	ra record;
	se record;
BEGIN
	-- Don't do unneeded work.
	if NOT NEW.is_pb then
		RETURN NULL;
	end if;

	-- Get section of run.
	SELECT patch, layout, category FROM section INTO se
	WHERE id = NEW.section_id;

	-- Add new rank if first submit.
	SELECT COUNT(id) FROM run INTO submits 
	WHERE section_id = NEW.section_id AND user_id = NEW.user_id;
	
	if submits = 1 then
		INSERT INTO rank (user_id, patch, layout, category, title, rank, rating, percentage, created_at, updated_at)
		VALUES (NEW.user_id, se.patch, se.layout, se.category, 'None', (SELECT COUNT(id) 
			FROM rank WHERE patch = se.patch AND layout = se.layout AND category = se.category) + 1,
			0.0, 0.0, NEW.created_at, NEW.created_at),
			(NEW.user_id, se.patch, NULL, NULL, 'None', (SELECT COUNT(id) 
			FROM rank WHERE patch = se.patch AND layout IS NULL AND category IS NULL) + 1,
			0.0, 0.0, NEW.created_at, NEW.created_at)
		ON CONFLICT DO NOTHING;

		-- Update percentage of submitted maps.
		UPDATE rank
		SET percentage = (SELECT COUNT(DISTINCT section_id)
			FROM run r
			JOIN section s ON section_id = s.id
			WHERE patch = se.patch AND layout = se.layout 
				AND category = se.category 
				AND user_id = NEW.user_id)::double precision / 
			(SELECT COUNT(id) 
				FROM section 
				WHERE patch = se.patch AND layout = se.layout 
				AND category = se.category)::double precision
		WHERE patch = se.patch AND layout = se.layout 
			AND category = se.category AND user_id = NEW.user_id;

		UPDATE rank
		SET percentage = (SELECT COUNT(DISTINCT section_id) 
			FROM run r
			JOIN section s ON section_id = s.id
			WHERE patch = se.patch AND user_id = NEW.user_id) / 
			(SELECT COUNT(id) 
				FROM section 
				WHERE patch = se.patch)::double precision
		WHERE patch = se.patch AND layout IS NULL 
			AND category IS NULL AND user_id = NEW.user_id;
	end if;

	-- When pb update only user rating.
	if NOT NEW.is_wr then
		-- Update category rating.
		SELECT id, percentage INTO ra
		FROM rank
		WHERE patch = se.patch AND layout = se.layout 
			AND category = se.category AND user_id = NEW.user_id;

		WITH perc AS (SELECT AVG(points) 
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category AND user_id = NEW.user_id)
		UPDATE rank
		SET updated_at = NEW.created_at,
			rating = (2000 + 8000 * LN(1 + ra.percentage / 0.15) / 2.03688192726104 
				* (1.25 - ra.percentage / 4)) 
			* POW(LN((SELECT * FROM perc) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + ra.percentage / 0.01) / 4.61512051684126 
				* (1.25 - ra.percentage / 4))
		WHERE id = ra.id;

		-- Update overall rating.
		SELECT id, percentage INTO ra
		FROM rank
		WHERE patch = se.patch AND layout IS NULL
			AND category IS NULL AND user_id = NEW.user_id;

		WITH perc AS (SELECT AVG(points) 
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND user_id = NEW.user_id)
		UPDATE rank
		SET updated_at = NEW.created_at,
			rating = (2000 + 8000 * LN(1 + ra.percentage / 0.15) / 2.03688192726104 
				* (1.25 - ra.percentage / 4)) 
			* POW(LN((SELECT * FROM perc) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + ra.percentage / 0.01) / 4.61512051684126
				* (1.25 - ra.percentage / 4))
		WHERE id = ra.id;
	-- When wr update all ranks.
	else
		-- Update category ratings.
		WITH ran AS (SELECT percentage AS p, user_id AS u 
				FROM rank
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category),
			perc AS (SELECT AVG(points) AS p, user_id AS u
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch AND layout = se.layout 
					AND category = se.category
				GROUP BY user_id)
		UPDATE rank r
		SET updated_at = NEW.created_at,
			rating = (2000 + 8000 
				* LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.15) / 2.03688192726104 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4)) 
			* POW(LN((SELECT p FROM perc WHERE u = r.user_id) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.01) / 4.61512051684126 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4))
		WHERE patch = se.patch AND layout = se.layout AND category = se.category;
		
		-- Update overall ratings.
		WITH ran AS (SELECT percentage AS p, user_id AS u 
				FROM rank
				WHERE patch = se.patch AND layout IS NULL 
					AND category IS NULL),
			perc AS (SELECT AVG(points) AS p, user_id AS u
				FROM run r 
				JOIN section s ON section_id = s.id
				WHERE patch = se.patch
				GROUP BY user_id)
		UPDATE rank r
		SET updated_at = NEW.created_at,
			rating = (2000 + 8000 
				* LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.15) / 2.03688192726104 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4)) 
			* POW(LN((SELECT p FROM perc WHERE u = r.user_id) * (EXP(1) - 1) + 1), 
				50 - 44 * LN(1 + (SELECT p FROM ran WHERE u = r.user_id) / 0.01) / 4.61512051684126 
				* (1.25 - (SELECT p FROM ran WHERE u = r.user_id) / 4))
		WHERE patch = se.patch AND layout IS NULL AND category IS NULL;
	end if;

	-- Update ranks and titles
	CALL update_rank(se.patch, se.layout, se.category);
	CALL update_rank(se.patch, NULL, NULL);

	RETURN NULL;
END;$$;
 )   DROP FUNCTION public.run_submit_ranks();
       public          postgres    false            �            1255    22347    submit_notify()    FUNCTION     �   CREATE FUNCTION public.submit_notify() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	PERFORM pg_notify('submit', NEW.id::text);
	RETURN NULL;
END;$$;
 &   DROP FUNCTION public.submit_notify();
       public          postgres    false            �            1255    22581 D   update_rank(character varying, character varying, character varying) 	   PROCEDURE     �  CREATE PROCEDURE public.update_rank(IN patch character varying, IN layout character varying, IN category character varying)
    LANGUAGE sql
    AS $_$WITH ra AS (SELECT rank() OVER (ORDER BY rating DESC, updated_at ASC) AS rank, id
	FROM rank r2
	WHERE r2.patch IS NOT DISTINCT FROM $1 
		AND r2.layout IS NOT DISTINCT FROM $2 
		AND r2.category IS NOT DISTINCT FROM $3)
UPDATE rank r
SET rank = (SELECT ra.rank FROM ra WHERE ra.id = r.id),
	title = (SELECT CASE
		WHEN (SELECT ra.rank FROM ra WHERE ra.id = r.id) = 1 THEN 'TopOne'::title
		WHEN r.rating < 1500 THEN 'None'::title
		WHEN r.rating < 3000 THEN 'Surfer'::title
		WHEN r.rating < 5000 THEN 'SuperSurfer'::title
		WHEN r.rating < 7500 THEN 'EpicSurfer'::title
		WHEN r.rating < 9000 THEN 'LegendarySurfer'::title
		ELSE 'MythicSurfer'::title 
	END)
WHERE r.patch IS NOT DISTINCT FROM $1 
	AND r.layout IS NOT DISTINCT FROM $2 
	AND r.category IS NOT DISTINCT FROM $3;$_$;
 {   DROP PROCEDURE public.update_rank(IN patch character varying, IN layout character varying, IN category character varying);
       public          postgres    false            �            1259    22348    activity    TABLE     �  CREATE TABLE public.activity (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    rank_id integer,
    title_old public.title,
    title_new public.title,
    rank_old integer,
    rank_new integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT either_of CHECK (((title_old IS NULL) OR (rank_old IS NULL))),
    CONSTRAINT full_rank CHECK (((rank_old IS NULL) = (rank_new IS NULL))),
    CONSTRAINT full_title CHECK (((title_old IS NULL) = (title_new IS NULL)))
);
    DROP TABLE public.activity;
       public         heap    postgres    false    871    871                       0    0    TABLE activity    ACL     C   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.activity TO lsl;
          public          postgres    false    215            �            1259    22355    activity_id_seq    SEQUENCE     �   ALTER TABLE public.activity ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.activity_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    215            �            1259    22356    discord    TABLE     y  CREATE TABLE public.discord (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    snowflake character varying(32) NOT NULL,
    name character varying(32) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    access character varying(2048) NOT NULL,
    refresh character varying(512) NOT NULL,
    expires_at timestamp with time zone NOT NULL
);
    DROP TABLE public.discord;
       public         heap    postgres    false                       0    0    TABLE discord    ACL     B   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.discord TO lsl;
          public          postgres    false    217            �            1259    22362    discord_id_seq    SEQUENCE     �   ALTER TABLE public.discord ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.discord_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    217            �            1259    22363    run    TABLE     �  CREATE TABLE public.run (
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
    DROP TABLE public.run;
       public            postgres    false                       0    0 	   TABLE run    ACL     >   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.run TO lsl;
          public          postgres    false    219            �            1259    22368 
   patch_1_00    TABLE     �  CREATE TABLE public.patch_1_00 (
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
);
    DROP TABLE public.patch_1_00;
       public         heap    postgres    false    219            �            1259    22373 
   patch_1_41    TABLE     �  CREATE TABLE public.patch_1_41 (
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
);
    DROP TABLE public.patch_1_41;
       public         heap    postgres    false    219            �            1259    22378 
   patch_1_50    TABLE     �  CREATE TABLE public.patch_1_50 (
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
);
    DROP TABLE public.patch_1_50;
       public         heap    postgres    false    219            �            1259    22383 
   patch_2_00    TABLE     �  CREATE TABLE public.patch_2_00 (
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
);
    DROP TABLE public.patch_2_00;
       public         heap    postgres    false    219            �            1259    22388 
   patch_2_13    TABLE     �  CREATE TABLE public.patch_2_13 (
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
);
    DROP TABLE public.patch_2_13;
       public         heap    postgres    false    219            �            1259    22393 
   permission    TABLE     �   CREATE TABLE public.permission (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    token public.permissions NOT NULL
);
    DROP TABLE public.permission;
       public         heap    postgres    false    868                       0    0    TABLE permission    ACL     E   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.permission TO lsl;
          public          postgres    false    225            �            1259    22396    permission_id_seq    SEQUENCE     �   ALTER TABLE public.permission ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.permission_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    225            �            1259    22397    rank    TABLE     �  CREATE TABLE public.rank (
    id integer NOT NULL,
    user_id bigint NOT NULL,
    patch character varying(128) NOT NULL,
    layout character varying(128),
    category character varying(128),
    title public.title NOT NULL,
    rank integer NOT NULL,
    rating double precision NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    percentage double precision NOT NULL
);
    DROP TABLE public.rank;
       public         heap    postgres    false    871                       0    0 
   TABLE rank    ACL     ?   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.rank TO lsl;
          public          postgres    false    227            �            1259    22402    rank_id_seq    SEQUENCE     �   ALTER TABLE public.rank ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.rank_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    227            �            1259    22403 
   run_id_seq    SEQUENCE     �   ALTER TABLE public.run ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.run_id_seq
    START WITH 20666
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    219            �            1259    22404    section    TABLE     G  CREATE TABLE public.section (
    id integer NOT NULL,
    patch character varying(128) NOT NULL,
    layout character varying(128) NOT NULL,
    category character varying(128) NOT NULL,
    map character varying(128) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    code character(4) NOT NULL
);
    DROP TABLE public.section;
       public         heap    postgres    false                       0    0    TABLE section    ACL     -   GRANT SELECT ON TABLE public.section TO lsl;
          public          postgres    false    230            �            1259    22410    section_id_seq    SEQUENCE     �   ALTER TABLE public.section ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.section_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    230            �            1259    22411    session    TABLE     w   CREATE TABLE public.session (
    id character varying(128) NOT NULL,
    expires bigint,
    session text NOT NULL
);
    DROP TABLE public.session;
       public         heap    postgres    false                       0    0    TABLE session    ACL     B   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.session TO lsl;
          public          postgres    false    232            �            1259    22416    user    TABLE     =  CREATE TABLE public."user" (
    id bigint NOT NULL,
    name character varying(32) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    password character varying(512) NOT NULL,
    pfp character varying(512) DEFAULT 'default'::character varying NOT NULL,
    bio character varying(2048)
);
    DROP TABLE public."user";
       public         heap    postgres    false                       0    0    TABLE "user"    ACL     A   GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public."user" TO lsl;
          public          postgres    false    233            �            1259    22423    user_id_seq    SEQUENCE     �   ALTER TABLE public."user" ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.user_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);
            public          postgres    false    233            �           0    0 
   patch_1_00    TABLE ATTACH     ]   ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_00 FOR VALUES FROM (1) TO (125);
          public          postgres    false    220    219            �           0    0 
   patch_1_41    TABLE ATTACH     _   ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_41 FOR VALUES FROM (125) TO (373);
          public          postgres    false    221    219            �           0    0 
   patch_1_50    TABLE ATTACH     _   ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_1_50 FOR VALUES FROM (373) TO (683);
          public          postgres    false    222    219            �           0    0 
   patch_2_00    TABLE ATTACH     `   ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_2_00 FOR VALUES FROM (683) TO (1093);
          public          postgres    false    223    219            �           0    0 
   patch_2_13    TABLE ATTACH     H   ALTER TABLE ONLY public.run ATTACH PARTITION public.patch_2_13 DEFAULT;
          public          postgres    false    224    219                       2606    22425    activity activity_pkey 
   CONSTRAINT     T   ALTER TABLE ONLY public.activity
    ADD CONSTRAINT activity_pkey PRIMARY KEY (id);
 @   ALTER TABLE ONLY public.activity DROP CONSTRAINT activity_pkey;
       public            postgres    false    215                       2606    22427    discord discord_pkey 
   CONSTRAINT     R   ALTER TABLE ONLY public.discord
    ADD CONSTRAINT discord_pkey PRIMARY KEY (id);
 >   ALTER TABLE ONLY public.discord DROP CONSTRAINT discord_pkey;
       public            postgres    false    217                       2606    22429    run id 
   CONSTRAINT     P   ALTER TABLE ONLY public.run
    ADD CONSTRAINT id PRIMARY KEY (id, section_id);
 0   ALTER TABLE ONLY public.run DROP CONSTRAINT id;
       public            postgres    false    219    219                       2606    22431    patch_1_00 patch_1_00_pkey 
   CONSTRAINT     d   ALTER TABLE ONLY public.patch_1_00
    ADD CONSTRAINT patch_1_00_pkey PRIMARY KEY (id, section_id);
 D   ALTER TABLE ONLY public.patch_1_00 DROP CONSTRAINT patch_1_00_pkey;
       public            postgres    false    3353    220    220    220            $           2606    22433    patch_1_41 patch_1_41_pkey 
   CONSTRAINT     d   ALTER TABLE ONLY public.patch_1_41
    ADD CONSTRAINT patch_1_41_pkey PRIMARY KEY (id, section_id);
 D   ALTER TABLE ONLY public.patch_1_41 DROP CONSTRAINT patch_1_41_pkey;
       public            postgres    false    3353    221    221    221            )           2606    22435    patch_1_50 patch_1_50_pkey 
   CONSTRAINT     d   ALTER TABLE ONLY public.patch_1_50
    ADD CONSTRAINT patch_1_50_pkey PRIMARY KEY (id, section_id);
 D   ALTER TABLE ONLY public.patch_1_50 DROP CONSTRAINT patch_1_50_pkey;
       public            postgres    false    222    222    222    3353            .           2606    22437    patch_2_00 patch_2_00_pkey 
   CONSTRAINT     d   ALTER TABLE ONLY public.patch_2_00
    ADD CONSTRAINT patch_2_00_pkey PRIMARY KEY (id, section_id);
 D   ALTER TABLE ONLY public.patch_2_00 DROP CONSTRAINT patch_2_00_pkey;
       public            postgres    false    3353    223    223    223            3           2606    22439    patch_2_13 patch_2_13_pkey 
   CONSTRAINT     d   ALTER TABLE ONLY public.patch_2_13
    ADD CONSTRAINT patch_2_13_pkey PRIMARY KEY (id, section_id);
 D   ALTER TABLE ONLY public.patch_2_13 DROP CONSTRAINT patch_2_13_pkey;
       public            postgres    false    224    224    3353    224            7           2606    22441    permission permission_pkey 
   CONSTRAINT     X   ALTER TABLE ONLY public.permission
    ADD CONSTRAINT permission_pkey PRIMARY KEY (id);
 D   ALTER TABLE ONLY public.permission DROP CONSTRAINT permission_pkey;
       public            postgres    false    225            9           2606    22443    rank rank_pkey 
   CONSTRAINT     L   ALTER TABLE ONLY public.rank
    ADD CONSTRAINT rank_pkey PRIMARY KEY (id);
 8   ALTER TABLE ONLY public.rank DROP CONSTRAINT rank_pkey;
       public            postgres    false    227            @           2606    22445    section section_pkey 
   CONSTRAINT     R   ALTER TABLE ONLY public.section
    ADD CONSTRAINT section_pkey PRIMARY KEY (id);
 >   ALTER TABLE ONLY public.section DROP CONSTRAINT section_pkey;
       public            postgres    false    230            B           2606    22447    session session_pkey 
   CONSTRAINT     R   ALTER TABLE ONLY public.session
    ADD CONSTRAINT session_pkey PRIMARY KEY (id);
 >   ALTER TABLE ONLY public.session DROP CONSTRAINT session_pkey;
       public            postgres    false    232            >           2606    22449    rank unique_title 
   CONSTRAINT     {   ALTER TABLE ONLY public.rank
    ADD CONSTRAINT unique_title UNIQUE NULLS NOT DISTINCT (patch, layout, category, user_id);
 ;   ALTER TABLE ONLY public.rank DROP CONSTRAINT unique_title;
       public            postgres    false    227    227    227    227            D           2606    22451    user unique_user 
   CONSTRAINT     M   ALTER TABLE ONLY public."user"
    ADD CONSTRAINT unique_user UNIQUE (name);
 <   ALTER TABLE ONLY public."user" DROP CONSTRAINT unique_user;
       public            postgres    false    233            F           2606    22453    user user_pkey 
   CONSTRAINT     N   ALTER TABLE ONLY public."user"
    ADD CONSTRAINT user_pkey PRIMARY KEY (id);
 :   ALTER TABLE ONLY public."user" DROP CONSTRAINT user_pkey;
       public            postgres    false    233                       1259    22454    activity_created_at_index    INDEX     t   CREATE INDEX activity_created_at_index ON public.activity USING btree (created_at) WITH (deduplicate_items='true');
 -   DROP INDEX public.activity_created_at_index;
       public            postgres    false    215                       1259    22455    run_created_at_index    INDEX     o   CREATE INDEX run_created_at_index ON ONLY public.run USING btree (created_at) WITH (deduplicate_items='true');
 (   DROP INDEX public.run_created_at_index;
       public            postgres    false    219                       1259    22456    patch_1_00_created_at_idx    INDEX     v   CREATE INDEX patch_1_00_created_at_idx ON public.patch_1_00 USING btree (created_at) WITH (deduplicate_items='true');
            public            postgres    false    220    3354    220                       1259    22457    run_section_index    INDEX     l   CREATE INDEX run_section_index ON ONLY public.run USING btree (section_id) WITH (deduplicate_items='true');
 %   DROP INDEX public.run_section_index;
       public            postgres    false    219                        1259    22458    patch_1_00_section_id_idx    INDEX     v   CREATE INDEX patch_1_00_section_id_idx ON public.patch_1_00 USING btree (section_id) WITH (deduplicate_items='true');
            public            postgres    false    220    3355    220                       1259    22459    run_user_id_index    INDEX     i   CREATE INDEX run_user_id_index ON ONLY public.run USING btree (user_id) WITH (deduplicate_items='true');
 %   DROP INDEX public.run_user_id_index;
       public            postgres    false    219            !           1259    22460    patch_1_00_user_id_idx    INDEX     p   CREATE INDEX patch_1_00_user_id_idx ON public.patch_1_00 USING btree (user_id) WITH (deduplicate_items='true');
            public            postgres    false    220    220    3356            "           1259    22461    patch_1_41_created_at_idx    INDEX     v   CREATE INDEX patch_1_41_created_at_idx ON public.patch_1_41 USING btree (created_at) WITH (deduplicate_items='true');
            public            postgres    false    221    221    3354            %           1259    22462    patch_1_41_section_id_idx    INDEX     v   CREATE INDEX patch_1_41_section_id_idx ON public.patch_1_41 USING btree (section_id) WITH (deduplicate_items='true');
            public            postgres    false    3355    221    221            &           1259    22463    patch_1_41_user_id_idx    INDEX     p   CREATE INDEX patch_1_41_user_id_idx ON public.patch_1_41 USING btree (user_id) WITH (deduplicate_items='true');
            public            postgres    false    3356    221    221            '           1259    22464    patch_1_50_created_at_idx    INDEX     v   CREATE INDEX patch_1_50_created_at_idx ON public.patch_1_50 USING btree (created_at) WITH (deduplicate_items='true');
            public            postgres    false    3354    222    222            *           1259    22465    patch_1_50_section_id_idx    INDEX     v   CREATE INDEX patch_1_50_section_id_idx ON public.patch_1_50 USING btree (section_id) WITH (deduplicate_items='true');
            public            postgres    false    222    222    3355            +           1259    22466    patch_1_50_user_id_idx    INDEX     p   CREATE INDEX patch_1_50_user_id_idx ON public.patch_1_50 USING btree (user_id) WITH (deduplicate_items='true');
            public            postgres    false    3356    222    222            ,           1259    22467    patch_2_00_created_at_idx    INDEX     v   CREATE INDEX patch_2_00_created_at_idx ON public.patch_2_00 USING btree (created_at) WITH (deduplicate_items='true');
            public            postgres    false    223    3354    223            /           1259    22468    patch_2_00_section_id_idx    INDEX     v   CREATE INDEX patch_2_00_section_id_idx ON public.patch_2_00 USING btree (section_id) WITH (deduplicate_items='true');
            public            postgres    false    223    223    3355            0           1259    22469    patch_2_00_user_id_idx    INDEX     p   CREATE INDEX patch_2_00_user_id_idx ON public.patch_2_00 USING btree (user_id) WITH (deduplicate_items='true');
            public            postgres    false    3356    223    223            1           1259    22470    patch_2_13_created_at_idx    INDEX     v   CREATE INDEX patch_2_13_created_at_idx ON public.patch_2_13 USING btree (created_at) WITH (deduplicate_items='true');
            public            postgres    false    3354    224    224            4           1259    22471    patch_2_13_section_id_idx    INDEX     v   CREATE INDEX patch_2_13_section_id_idx ON public.patch_2_13 USING btree (section_id) WITH (deduplicate_items='true');
            public            postgres    false    224    224    3355            5           1259    22472    patch_2_13_user_id_idx    INDEX     p   CREATE INDEX patch_2_13_user_id_idx ON public.patch_2_13 USING btree (user_id) WITH (deduplicate_items='true');
            public            postgres    false    3356    224    224            :           1259    22473    rank_rating    INDEX     ^   CREATE INDEX rank_rating ON public.rank USING btree (rating) WITH (deduplicate_items='true');
    DROP INDEX public.rank_rating;
       public            postgres    false    227            ;           1259    22474    rank_section    INDEX     p   CREATE INDEX rank_section ON public.rank USING btree (patch, layout, category) WITH (deduplicate_items='true');
     DROP INDEX public.rank_section;
       public            postgres    false    227    227    227            <           1259    22475    rank_user_id_index    INDEX     f   CREATE INDEX rank_user_id_index ON public.rank USING btree (user_id) WITH (deduplicate_items='true');
 &   DROP INDEX public.rank_user_id_index;
       public            postgres    false    227            G           0    0    patch_1_00_created_at_idx    INDEX ATTACH     [   ALTER INDEX public.run_created_at_index ATTACH PARTITION public.patch_1_00_created_at_idx;
          public          postgres    false    3357    3354    220    219            H           0    0    patch_1_00_pkey    INDEX ATTACH     ?   ALTER INDEX public.id ATTACH PARTITION public.patch_1_00_pkey;
          public          postgres    false    3359    3353    220    3353    220    219            I           0    0    patch_1_00_section_id_idx    INDEX ATTACH     X   ALTER INDEX public.run_section_index ATTACH PARTITION public.patch_1_00_section_id_idx;
          public          postgres    false    3360    3355    220    219            J           0    0    patch_1_00_user_id_idx    INDEX ATTACH     U   ALTER INDEX public.run_user_id_index ATTACH PARTITION public.patch_1_00_user_id_idx;
          public          postgres    false    3361    3356    220    219            K           0    0    patch_1_41_created_at_idx    INDEX ATTACH     [   ALTER INDEX public.run_created_at_index ATTACH PARTITION public.patch_1_41_created_at_idx;
          public          postgres    false    3362    3354    221    219            L           0    0    patch_1_41_pkey    INDEX ATTACH     ?   ALTER INDEX public.id ATTACH PARTITION public.patch_1_41_pkey;
          public          postgres    false    221    3353    3364    3353    221    219            M           0    0    patch_1_41_section_id_idx    INDEX ATTACH     X   ALTER INDEX public.run_section_index ATTACH PARTITION public.patch_1_41_section_id_idx;
          public          postgres    false    3365    3355    221    219            N           0    0    patch_1_41_user_id_idx    INDEX ATTACH     U   ALTER INDEX public.run_user_id_index ATTACH PARTITION public.patch_1_41_user_id_idx;
          public          postgres    false    3366    3356    221    219            O           0    0    patch_1_50_created_at_idx    INDEX ATTACH     [   ALTER INDEX public.run_created_at_index ATTACH PARTITION public.patch_1_50_created_at_idx;
          public          postgres    false    3367    3354    222    219            P           0    0    patch_1_50_pkey    INDEX ATTACH     ?   ALTER INDEX public.id ATTACH PARTITION public.patch_1_50_pkey;
          public          postgres    false    3369    3353    222    3353    222    219            Q           0    0    patch_1_50_section_id_idx    INDEX ATTACH     X   ALTER INDEX public.run_section_index ATTACH PARTITION public.patch_1_50_section_id_idx;
          public          postgres    false    3370    3355    222    219            R           0    0    patch_1_50_user_id_idx    INDEX ATTACH     U   ALTER INDEX public.run_user_id_index ATTACH PARTITION public.patch_1_50_user_id_idx;
          public          postgres    false    3371    3356    222    219            S           0    0    patch_2_00_created_at_idx    INDEX ATTACH     [   ALTER INDEX public.run_created_at_index ATTACH PARTITION public.patch_2_00_created_at_idx;
          public          postgres    false    3372    3354    223    219            T           0    0    patch_2_00_pkey    INDEX ATTACH     ?   ALTER INDEX public.id ATTACH PARTITION public.patch_2_00_pkey;
          public          postgres    false    3353    223    3374    3353    223    219            U           0    0    patch_2_00_section_id_idx    INDEX ATTACH     X   ALTER INDEX public.run_section_index ATTACH PARTITION public.patch_2_00_section_id_idx;
          public          postgres    false    3375    3355    223    219            V           0    0    patch_2_00_user_id_idx    INDEX ATTACH     U   ALTER INDEX public.run_user_id_index ATTACH PARTITION public.patch_2_00_user_id_idx;
          public          postgres    false    3376    3356    223    219            W           0    0    patch_2_13_created_at_idx    INDEX ATTACH     [   ALTER INDEX public.run_created_at_index ATTACH PARTITION public.patch_2_13_created_at_idx;
          public          postgres    false    3377    3354    224    219            X           0    0    patch_2_13_pkey    INDEX ATTACH     ?   ALTER INDEX public.id ATTACH PARTITION public.patch_2_13_pkey;
          public          postgres    false    3379    3353    224    3353    224    219            Y           0    0    patch_2_13_section_id_idx    INDEX ATTACH     X   ALTER INDEX public.run_section_index ATTACH PARTITION public.patch_2_13_section_id_idx;
          public          postgres    false    3380    3355    224    219            Z           0    0    patch_2_13_user_id_idx    INDEX ATTACH     U   ALTER INDEX public.run_user_id_index ATTACH PARTITION public.patch_2_13_user_id_idx;
          public          postgres    false    3381    3356    224    219            b           2620    22476    activity activity_insert    TRIGGER     w   CREATE TRIGGER activity_insert AFTER INSERT ON public.activity FOR EACH ROW EXECUTE FUNCTION public.activity_notify();
 1   DROP TRIGGER activity_insert ON public.activity;
       public          postgres    false    235    215            c           2620    22477    discord discord_insert    TRIGGER     t   CREATE TRIGGER discord_insert AFTER INSERT ON public.discord FOR EACH ROW EXECUTE FUNCTION public.discord_notify();
 /   DROP TRIGGER discord_insert ON public.discord;
       public          postgres    false    217    238            h           2620    22478    rank rank_update    TRIGGER     �   CREATE TRIGGER rank_update AFTER UPDATE OF title, rank ON public.rank FOR EACH ROW EXECUTE FUNCTION public.activity_rank_update();
 )   DROP TRIGGER rank_update ON public.rank;
       public          postgres    false    236    227    227    227            d           2620    22481    run run_delete    TRIGGER     h   CREATE TRIGGER run_delete AFTER DELETE ON public.run FOR EACH ROW EXECUTE FUNCTION public.run_remove();
 '   DROP TRIGGER run_delete ON public.run;
       public          postgres    false    219    254            e           2620    22487    run run_insert    TRIGGER     i   CREATE TRIGGER run_insert BEFORE INSERT ON public.run FOR EACH ROW EXECUTE FUNCTION public.run_submit();
 '   DROP TRIGGER run_insert ON public.run;
       public          postgres    false    251    219            f           2620    22493    run run_insert_notify    TRIGGER     r   CREATE TRIGGER run_insert_notify AFTER INSERT ON public.run FOR EACH ROW EXECUTE FUNCTION public.submit_notify();
 .   DROP TRIGGER run_insert_notify ON public.run;
       public          postgres    false    219    252            g           2620    22499    run run_insert_ranks    TRIGGER     t   CREATE TRIGGER run_insert_ranks AFTER INSERT ON public.run FOR EACH ROW EXECUTE FUNCTION public.run_submit_ranks();
 -   DROP TRIGGER run_insert_ranks ON public.run;
       public          postgres    false    253    219            i           2620    22505    user user_insert    TRIGGER     s   CREATE TRIGGER user_insert AFTER INSERT ON public."user" FOR EACH ROW EXECUTE FUNCTION public.activity_user_add();
 +   DROP TRIGGER user_insert ON public."user";
       public          postgres    false    233    237            [           2606    22506    activity rank_id    FK CONSTRAINT     �   ALTER TABLE ONLY public.activity
    ADD CONSTRAINT rank_id FOREIGN KEY (rank_id) REFERENCES public.rank(id) ON UPDATE CASCADE ON DELETE CASCADE;
 :   ALTER TABLE ONLY public.activity DROP CONSTRAINT rank_id;
       public          postgres    false    227    3385    215            ^           2606    22511    run section_id    FK CONSTRAINT     �   ALTER TABLE public.run
    ADD CONSTRAINT section_id FOREIGN KEY (section_id) REFERENCES public.section(id) ON UPDATE CASCADE ON DELETE CASCADE;
 3   ALTER TABLE public.run DROP CONSTRAINT section_id;
       public          postgres    false    230    219    3392            `           2606    22531    permission user_id    FK CONSTRAINT     �   ALTER TABLE ONLY public.permission
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;
 <   ALTER TABLE ONLY public.permission DROP CONSTRAINT user_id;
       public          postgres    false    3398    225    233            _           2606    22536    run user_id    FK CONSTRAINT     �   ALTER TABLE public.run
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;
 0   ALTER TABLE public.run DROP CONSTRAINT user_id;
       public          postgres    false    3398    219    233            ]           2606    22556    discord user_id    FK CONSTRAINT     �   ALTER TABLE ONLY public.discord
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;
 9   ALTER TABLE ONLY public.discord DROP CONSTRAINT user_id;
       public          postgres    false    233    217    3398            a           2606    22561    rank user_id    FK CONSTRAINT     l   ALTER TABLE ONLY public.rank
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id);
 6   ALTER TABLE ONLY public.rank DROP CONSTRAINT user_id;
       public          postgres    false    3398    227    233            \           2606    22566    activity user_id    FK CONSTRAINT     �   ALTER TABLE ONLY public.activity
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON UPDATE CASCADE ON DELETE CASCADE;
 :   ALTER TABLE ONLY public.activity DROP CONSTRAINT user_id;
       public          postgres    false    3398    233    215           