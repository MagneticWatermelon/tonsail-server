use tonsail_server::{
    prisma::{organization, user, PrismaClient},
    util::hash::hash_password,
};

pub async fn seed_database() {
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
}
