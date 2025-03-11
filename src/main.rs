#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use futures::TryStreamExt;
    use tokio::io::AsyncBufReadExt;

    let initial_value = rand::random_range(0..=100i32);
    println!("{initial_value}");
    let game = guessing_game::Game::new(initial_value);
    let stdin = tokio::io::stdin();
    let stdin = tokio::io::BufReader::new(stdin);
    let stdin = tokio_stream::wrappers::LinesStream::new(stdin.lines());
    let input = stdin
        .map_err(|e| anyhow::Error::new(e).context("Stdin error"))
        .and_then(async |line| {
            let value: i32 = line.parse().context("Failed to parse input")?;
            Ok(value)
        });
    let output = game.try_session(input);
    tokio::pin!(output);
    while let Some(o) = output.try_next().await? {
        use std::cmp::Ordering::{Equal, Greater, Less};
        match o {
            Equal => println!("Correct!"),
            Greater => println!("Too high!"),
            Less => println!("Too low!"),
        }
    }
    Ok(())
}
