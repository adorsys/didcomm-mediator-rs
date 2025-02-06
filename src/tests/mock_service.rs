use mockito::{mock, Mock};

pub fn setup_mock_service() -> Mock {
    mock("GET", "/upstream")
        .with_status(503)
        .with_body("Service Unavailable")
        .create()
}
