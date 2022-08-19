mod client;

fn main() {
    let mut client = client::ClientBuilder::new()
        .size(10, 40)
        .cmd("tmux")
        .arg("new-session")
        .arg("-A")
        .arg("-s")
        .arg("client-name")
        .build();
    client.process_ms(1000);
    client.send_str("date -Is\r");
    client.to_stdout();
    client.kill();
    client.wait();
    //client.to_stdout();
}
