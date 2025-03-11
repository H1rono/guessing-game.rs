use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Game(Arc<Mutex<i32>>);

impl Game {
    pub fn new(value: i32) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub async fn guess(&self, value: i32) -> std::cmp::Ordering {
        let rh = self.0.lock().await;
        std::cmp::Ord::cmp(&value, &*rh)
    }

    pub fn session<S>(&self, input: S) -> impl futures::Stream<Item = std::cmp::Ordering> + Send
    where
        S: futures::Stream<Item = i32> + Send,
    {
        use futures::StreamExt;

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let tx_fut = async move {
            tokio::pin!(input);
            while let Some(value) = input.next().await {
                let o = self.guess(value).await;
                tx.send(o).await.unwrap();
            }
            Option::<_>::None
        };
        let tx_stream = futures::stream::once(tx_fut).filter_map(std::future::ready);
        let rx_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        futures::stream::select(tx_stream, rx_stream)
    }

    pub fn try_session<S, E>(
        &self,
        input: S,
    ) -> impl futures::Stream<Item = Result<std::cmp::Ordering, E>> + Send
    where
        S: futures::Stream<Item = Result<i32, E>> + Send,
        E: Send,
    {
        use futures::{StreamExt, TryStreamExt};

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let tx_fut = async move {
            tokio::pin!(input);
            while let Some(value) = input.try_next().await? {
                let o = self.guess(value).await;
                tx.send(Ok(o)).await.unwrap();
            }
            Result::<Option<_>, E>::Ok(None)
        };
        let tx_stream =
            futures::stream::once(tx_fut).filter_map(|r| std::future::ready(r.transpose()));
        let rx_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        futures::stream::select(tx_stream, rx_stream)
    }
}
