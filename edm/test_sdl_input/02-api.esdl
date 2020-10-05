module api {
    type Session {
        required single link user -> User;
        required single property allottedDuration -> std::duration {
            default := (WITH
                MODULE api
            SELECT
                <duration>'24 hours'
            );
        };
        required single property createdAt -> std::datetime {
            default := (WITH
                MODULE api
            SELECT
                datetime_current()
            );
        };
        required single property sessionID -> std::str;
        single property token -> std::str;
    };
    function clear_expired() -> SET OF Session using (
            WITH MODULE api
            DELETE Session
            IF
            (
                <datetime>Session.createdAt + <duration>Session.allottedDuration <= datetime_current()
            )
            ELSE <Session>{}
        );

    function create_new_session(EMAIL: std::str, PASS: std::str) -> SET OF Session using (
            WITH MODULE api 
            FOR USER IN {validate_credentials(EMAIL, PASS)}
            UNION (
                INSERT Session {
                    token := <str>(SELECT random_big_str_id()),
                    sessionID := <str>(SELECT random_big_str_id()),
                    user := USER
                }   
            )
         );

    function validate_credentials(EMAIL: std::str, PASS: std::str) -> SET OF User using (
            WITH MODULE api
            SELECT
                User
            FILTER
                .email = EMAIL
                AND
                .password = PASS
            LIMIT 1
        );
}
