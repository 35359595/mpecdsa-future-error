## Working example of higher-ranked error

`
error: higher-ranked lifetime error
--> src/main.rs:55:14
|
55 |       let _h = tokio::spawn(async move {
|  ______________^
56 | |         let _key = drive_keygen(&mut r, &s).await.unwrap();
57 | |     });
| |______^
|
= note: could not prove [async block@src/main.rs:55:27: 57:6]: std::marker::Send
error: could not compile inner_fut_issue due to previous error
`