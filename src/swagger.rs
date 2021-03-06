#[test]
fn test_index() {
	use crate::test::get_test_environment;
	use rocket::http::Status;
	let env = get_test_environment("swagger_index.sqlite");
	let client = &env.client;
	let response = client.get("/swagger").dispatch();
	assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_index_with_trailing_slash() {
	use crate::test::get_test_environment;
	use rocket::http::Status;
	let env = get_test_environment("swagger_index_with_trailing_slash.sqlite");
	let client = &env.client;
	let response = client.get("/swagger/").dispatch();
	assert_eq!(response.status(), Status::Ok);
}
