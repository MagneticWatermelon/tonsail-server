use prisma_client_rust::raw;
use sqlx::{Connection, PgConnection};
use tonsail_server::{
    prisma::{organization, user, PrismaClient},
    util::hash::hash_password,
};

pub async fn seed_database() {
    // MySQL seeding
    {
        let client = PrismaClient::_builder().build().await.unwrap();
        let new_org = client
            .organization()
            .upsert(
                organization::id::equals("orgid1".to_string()),
                organization::create("orgid1".to_string(), "org 1".to_string(), vec![]),
                vec![],
            )
            .exec()
            .await
            .unwrap();

        let _new_user = client
            .user()
            .upsert(
                user::id::equals("userid1".to_string()),
                user::create(
                    "userid1".to_string(),
                    "graham@bell.com".to_string(),
                    hash_password("Gr@h@mBell69".as_bytes()),
                    "Graham Bell".to_string(),
                    organization::id::equals(new_org.id.clone()),
                    vec![],
                ),
                vec![],
            )
            .exec()
            .await
            .unwrap();

        let _catalog = client
            ._execute_raw(raw!(
            "INSERT INTO tonsail.MetricsCatalog (label,value,`group`,unit,description) VALUES
            ('VUs', 'vus', 'general', 'count', 'The number of virtual users'),
            ('Request rate', 'http_request_rate', 'HTTP', 'rps', 'The rate at which requests are made to a given URL'),
            ('Response time', 'http_response_rate', 'HTTP', 'ms', 'The time spent waiting for the full response'),
            ('Failure rate', 'http_failure_rate', 'HTTP', '%', 'The percentage of requests that resulted in a failure'),
            ('Handshake time', 'http_handshake_time', 'HTTP', 'ms', 'The time it takes to establish a connection with the server'),
            ('Waiting time', 'http_waiting_time', 'HTTP', 'ms', 'The time spent waiting for a response from the server'),
            ('Blocked time', 'http_blocked_time', 'HTTP', 'ms', 'The time spent waiting for an available connection slot'),
            ('Sent data', 'http_sent_bytes', 'HTTP', 'bytes', 'The amount of data sent in the request'),
            ('Received data', 'http_recv_bytes', 'HTTP', 'bytes', 'The amount of data received in the response'),
            ('CPU usage', 'cpu_usage', 'System', '%', 'The percentage of CPU utilization'),
            ('Memory usage', 'mem_usage', 'System', '%', 'The percentage of memory utilization');"
            ))
            .exec()
            .await.unwrap();
    }

    // QuestDB seeding
    {
        let mut conn = PgConnection::connect("postgresql://admin:quest@localhost:8812/qdb")
            .await
            .unwrap();

        sqlx::query("CREATE table metrics (name Symbol, runID String, scenario String, url String, method Symbol, status Symbol, ts Timestamp, value Float)").execute(&mut conn).await.unwrap();
        sqlx::query("INSERT INTO metrics
    SELECT
        'http_request_rate' name,
        '3r2f039ffktv' runID,
        'Scenario 1' scenario,
        'https://api.tonsail.dev/health_check' url,
        'GET' method,
        rnd_str('200', '404') status,
        timestamp_sequence(to_timestamp('2023-03-15T00:00:00', 'yyyy-MM-ddTHH:mm:ss'), rnd_long(1,10,0) * 100000L) ts,
        rnd_double(0)*20 + 90 value
    FROM long_sequence(1000)").execute(&mut conn).await.unwrap();
    }
}
