use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_glacier::{
    self, operation::upload_archive::UploadArchiveOutput, primitives::ByteStream, Client,
};
#[tokio::main]
async fn main() {
    let client = get_client().await;
    print_vault_list(&client).await;
    let archive_out = upload_archive(
        &client,
        "C:/Users/terra/Downloads/test.txt",
        "testing_vault",
    )
    .await;
    println!("{:#?}", archive_out);
}
async fn get_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    Client::new(&config)
}
async fn print_vault_list(client: &Client) {
    let resp = client.list_vaults().account_id("-").send();
    let _ = match resp.await {
        Ok(vaults) => match vaults.vault_list {
            Some(vaults) => {
                if vaults.len() > 0 {
                    for vault in vaults {
                        println!(
                            "----- {} -----",
                            vault.vault_name.unwrap_or("Failed to fetch".to_string())
                        );
                        println!(
                            " - Created: {}",
                            vault
                                .creation_date
                                .map(|time_string| time_string.replace("T", " "))
                                .map(|time_string| time_string.replace("Z", ""))
                                .unwrap_or("Failed to fetch".to_string())
                        );
                        println!(" - Number of archives: {}", vault.number_of_archives);
                        println!(" - Size: {}B", vault.size_in_bytes);
                        println!(
                            " - Last inventory date: {}",
                            vault.last_inventory_date.unwrap_or("N/A".to_string())
                        );
                        println!(" - ARN: {}", vault.vault_arn.unwrap_or("N/A".to_string()));
                    }
                } else {
                    println!("There are no vaults in this region.")
                }
            }
            None => {
                println!("Failed to fetch vaults");
                todo!();
            }
        },
        Err(_) => {
            println!("Failed to fetch vaults");
            todo!();
        }
    };
}
async fn upload_archive(
    client: &Client,
    path: &str,
    vault_name: &str,
) -> Result<UploadArchiveOutput, Error> {
    let stream = match ByteStream::from_path(path).await {
        Ok(stream) => stream,
        Err(_) => todo!(),
    };
    match client
        .upload_archive()
        .archive_description("Uploading file \"{}\"")
        .body(stream)
        .vault_name(vault_name)
        .send()
        .await
    {
        Ok(archive_output) => return Ok(archive_output),
        Err(err) => {
            println!("Error: {:#?}", err);
            Err(Error::UploadFailed())
        }
    }
}
#[derive(Debug)]
enum Error {
    UploadFailed(),
}
