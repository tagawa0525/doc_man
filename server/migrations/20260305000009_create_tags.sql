CREATE TABLE tags (
    id   UUID          NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name VARCHAR(50)   NOT NULL,
    CONSTRAINT tags_name_unique UNIQUE (name)
);
