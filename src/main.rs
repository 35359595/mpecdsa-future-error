use curv::elliptic::curves::Secp256k1;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{Keygen, LocalKey, ProtocolMessage};
use round_based::{AsyncProtocol, Msg};
use tokio::sync::mpsc::{Receiver, Sender};
use anyhow::{Result, Error};
use futures::{TryStreamExt, StreamExt};

async fn drive_keygen(inp: &mut Receiver<Vec<u8>>, out: &Sender<Vec<u8>>) -> Result<LocalKey<Secp256k1>> {
    let keygen = Keygen::new(1, 3, 3)?;
    let stream = async_stream::stream! {
        while let Some(msg) = inp.recv().await {
            if let Ok(de) = serde_cbor::from_slice::<Msg<ProtocolMessage>>(&msg) {
                yield Ok::<_, anyhow::Error>(de)
            } else {
                yield Err(anyhow::Error::msg("Provided data is not serialized ProtocolMessage"))
            }
        }
    };
    let incoming = stream
        .try_filter(move |msg| {
            futures::future::ready(
                msg.sender != 1 && (msg.receiver.is_none() || msg.receiver == Some(1)),
            )
        })
        .fuse();
    tokio::pin!(incoming);

    // converting tokio Sender into Sink
    let outgoing = futures::sink::unfold(
        out,
        |sender, msg: Msg<ProtocolMessage>| async move {
            if let Ok(serialized) = serde_cbor::to_vec(&msg) {
                sender.send(serialized).await?;
                Ok::<_, Error>(sender)
            } else {
                Err(Error::msg("failed to serialize ProtocolMessage"))
            }
        },
    );
    tokio::pin!(outgoing);

    AsyncProtocol::new(keygen, incoming, outgoing)
        .run()
        .await
        .map_err(|e| anyhow::Error::msg(e.to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let (s, mut r) = tokio::sync::mpsc::channel(1);
    // works
    let _key = drive_keygen(&mut r, &s).await?;
    // errors
    let (s, mut r) = tokio::sync::mpsc::channel(1);
    let _h = tokio::spawn(async move {
        let _key = drive_keygen(&mut r, &s).await.unwrap();
    });
    Ok(())
}
