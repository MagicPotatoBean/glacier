use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_glacier::{
    self,
    operation::{initiate_job::InitiateJobOutput, upload_archive::UploadArchiveOutput},
    primitives::ByteStream,
    types::JobParameters,
    Client,
};
#[tokio::main]
async fn main() {
    let client = get_client().await;
    print_vault_list(&client).await;
    /* let archive_out = upload_archive(
        &client,
        "C:/Users/terra/Downloads/test.txt",
        "testing_vault",
    )
    .await;
    println!("{:#?}", archive_out); */
}
async fn get_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-2");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    Client::new(&config)
}
/// Gets all vaults in a given region, and prints their contents to the console.
///
/// # Panics
/// This will never panic
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
                            vault
                                .last_inventory_date
                                .map(|time_string| time_string.replace("T", " "))
                                .map(|time_string| time_string.replace("Z", ""))
                                .unwrap_or("N/A".to_string())
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
/// Uploads an archive from a filepath, returning the result of the upload
///
/// # Panics
/// This will never panic
async fn upload_archive(
    client: &Client,
    path: &str,
    vault_name: &str,
) -> Result<UploadArchiveOutput, Error> {
    let stream = match ByteStream::from_path(path).await {
        Ok(stream) => stream,
        Err(_) => {
            println!("Failed to get vault");
            return Err(Error::UploadFailed);
        }
    };
    match client
        .upload_archive()
        .archive_description("\"{}\"")
        .body(stream)
        .vault_name(vault_name)
        .send()
        .await
    {
        Ok(archive_output) => return Ok(archive_output),
        Err(err) => {
            println!("Error: {:#?}", err);
            Err(Error::UploadFailed)
        }
    }
}
/// Returns the JobID for the downloading of an archive
///
/// # Panics
/// This will never panic
async fn initiate_download(
    client: &Client,
    archive_id: &str,
    vault_name: &str,
    tier: DownLoadTier,
) -> Result<InitiateJobOutput, Error> {
    let params = JobParameters::builder()
        .archive_id(archive_id)
        .tier(tier.name())
        .description(format!["Getting an archive in \"{}\" tier", tier.name()])
        .build();
    match client
        .initiate_job()
        .account_id("-")
        .job_parameters(params)
        .vault_name(vault_name)
        .send()
        .await
    {
        Ok(job_output) => Ok(job_output),
        Err(_) => Err(Error::InitialiseDownloadFailed),
    }
}
/// Completes the download of an archive from a JobID
///
/// # Panics
/// This will never panic
async fn complete_download(client: &Client, job_id: &str, vault_name: &str) {
    match client
        .get_job_output()
        .job_id("-")
        .job_id(job_id)
        .vault_name(vault_name)
        .send()
        .await
    {
        Ok(job_output_output) => {
            println!("{:#?}", job_output_output)
        }
        Err(_) => todo!(),
    }
}
#[derive(Debug)]
/// Represents any kind of failure during the operation of this program
enum Error {
    UploadFailed,
    InitialiseDownloadFailed,
    CompleteDownloadFailed,
}
/// Represents the three types of download tiers with AWS Glacier
enum DownLoadTier {
    Expedited,
    Standard,
    Bulk,
}
impl DownLoadTier {
    /// Compares the input string to the three download tiers, if it matches, then it returns that tier, otherwise, it returns none.
    /// This is case-insensitive, and whitespace-insensitive.
    ///
    /// # Panics
    /// This will never panic.
    ///
    /// # Examples
    /// ```
    /// let tier_str = "bulk";
    /// assert_eq!(DownloadTier::parse(tier_str), DownloadTier::Bulk);
    /// ```
    fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "expedited" => Some(DownLoadTier::Expedited),
            "standard" => Some(DownLoadTier::Standard),
            "bulk" => Some(DownLoadTier::Bulk),
            _ => None,
        }
    }
    /// Returns the name of the contained type as a [String]
    ///
    /// # Panics
    /// This will never panic.
    ///
    fn name(&self) -> String {
        match self {
            DownLoadTier::Expedited => "Expedited".to_owned(),
            DownLoadTier::Standard => "Standard".to_owned(),
            DownLoadTier::Bulk => "Bulk".to_owned(),
        }
    }
    /// Returns the description of the contained type as a [String]
    ///
    /// # Panics
    /// This will never panic.
    ///
    fn describe(&self) -> String {
        match self {
            DownLoadTier::Expedited => "The fastest tier available".to_owned(),
            DownLoadTier::Standard => "The default tier".to_owned(),
            DownLoadTier::Bulk => "The slowest tier available".to_owned(),
        }
    }
    /// Returns the cost of the contained type as a [String]
    ///
    /// # Panics
    /// This will never panic.
    ///
    fn cost(&self) -> String {
        match self {
            DownLoadTier::Expedited => "£0.0250 / GB + £0.0105".to_owned(),
            DownLoadTier::Standard => "£0.0083 / GB + £0.0000318".to_owned(),
            DownLoadTier::Bulk => "Effectively free (Website says £0)".to_owned(),
        }
    }
}
