#[tokio::test]
async fn run_all_tests() {
    crate::rabbitmq::test::test().await;
    crate::data_store::test::test().await;
    crate::communication::test::test().await;
}
