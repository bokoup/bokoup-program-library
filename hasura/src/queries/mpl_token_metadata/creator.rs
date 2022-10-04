use mpl_token_metadata::state::Creator;
use tokio_postgres::{Client, Error};
use tracing::info;

const DELETE_QUERY: &str = include_str!("creator_delete.sql");

const UPSERT_QUERY: &str = include_str!("creator_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    creators: Option<&Vec<Creator>>,
    slot: i64,
    write_version: i64,
) -> Result<(), Error> {
    let metadata_id = bs58::encode(key).into_string();
    let (metadata, address, verified, share, slots, write_versions) =
        if let Some(creators) = creators {
            creators.iter().fold(
                (
                    Vec::<String>::new(),
                    Vec::<String>::new(),
                    Vec::<bool>::new(),
                    Vec::<i32>::new(),
                    Vec::<i64>::new(),
                    Vec::<i64>::new(),
                ),
                |(mut m, mut a, mut v, mut s, mut sl, mut w), c| {
                    m.push(metadata_id.clone());
                    a.push(c.address.to_string());
                    v.push(c.verified);
                    s.push(c.share as i32);
                    sl.push(slot);
                    w.push(write_version);
                    (m, a, v, s, sl, w)
                },
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };

    let insert = if creators.is_some() {
        client
            .query(
                UPSERT_QUERY,
                &[
                    &metadata,
                    &address,
                    &verified,
                    &share,
                    &slots,
                    &write_versions,
                ],
            )
            .await?
            .into_iter()
            .filter(|r| r.get(0))
            .count()
    } else {
        0
    };

    let delete = client
        .query(DELETE_QUERY, &[&metadata_id, &address])
        .await?
        .len();

    info!(metadata = metadata_id.as_str(), insert, delete);

    Ok(())
}
