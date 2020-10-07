CREATE FUNCTION edgemorph_core::prepare_call(name: str) -> SET OF schema::`Function` using (
    WITH MODULE schema
    SELECT `Function` {
        name,
        annotations: { name, @value },
        params: {
            kind,
            name,
            num,
            typemod,
            type: { name },
            default,
        },
        return_typemod,
        return_type: { name },
        id }
    FILTER .name = 'edgemorph_core::{{name}}';
);

CREATE ABSTRACT type Ty extending std::BaseObject {
    CREATE REQUIRED PROPERTY builtin -> bool { set default := false ; };
    CREATE REQUIRED PROPERTY is_internal -> bool { set default := false ; };
    CREATE REQUIRED PROPERTY name -> str;
    CREATE OPTIONAL PROPERTY is_ABSTRACT -> bool { set default := false ; };
    CREATE OPTIONAL PROPERTY is_final -> bool { set default := false ; };
    CREATE PROPERTY expr -> std::str;
    CREATE PROPERTY is_from_alias := (EXISTS (.expr)); 
};
