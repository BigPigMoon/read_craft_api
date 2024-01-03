-- Add up migration script here
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'language') THEN
    CREATE TYPE language AS ENUM (
       'bg',
       'cs',
       'da',
       'de',
       'el',
       'en',
       'es',
       'et',
       'fi',
       'fr',
       'hu',
       'it',
       'ja',
       'lt',
       'lv',
       'mt',
       'nl',
       'pl',
       'pt',
       'ro',
       'ru',
       'sk',
       'sl',
       'sv',
       'zh'
    );
  END IF;
END $$;
