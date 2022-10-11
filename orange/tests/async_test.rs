#[tokio::test]
async fn test_async() {
	let handle = tokio::spawn(async {
		"return value"
	});

	let out = handle.await.unwrap();
	println!("GOT {}", out);
}
